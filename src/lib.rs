use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::{error, info};

struct WgpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    _window: ndk::native_window::NativeWindow,
}

impl WgpuState {
    async fn new(window: ndk::native_window::NativeWindow) -> Result<Self, String> {
        let width = window.width() as u32;
        let height = window.height() as u32;

        let instance = wgpu::Instance::default();

        let win_ptr = window.ptr().as_ptr();
        let raw_display = raw_window_handle::RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::new());
        let raw_window = raw_window_handle::RawWindowHandle::AndroidNdk(
            raw_window_handle::AndroidNdkWindowHandle::new(
                std::ptr::NonNull::new(win_ptr as *mut std::ffi::c_void).unwrap()
            )
        );

        let unsafe_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(raw_display),
            raw_window_handle: raw_window,
        };

        let surface = unsafe {
            instance.create_surface_unsafe(unsafe_target)
                .map_err(|e| format!("Gagal membuat surface: {:?}", e))?
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("Gagal menemukan adapter grafis: {:?}", e))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| format!("Gagal meminta device wgpu: {:?}", e))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: if width > 0 { width } else { 1 },
            height: if height > 0 { height } else { 1 },
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        info!("wgpu v30 berhasil diinisialisasi! Resolusi: {}x{}", width, height);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: (width, height),
            _window: window,
        })
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.size = (new_width, new_height);
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), String> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            other => return Err(format!("Gagal mengambil surface texture: {:?}", other)),
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Warna latar belakang ungu gelap khas Geometry Dash
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.22,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        // Di wgpu v30, surface_texture otomatis dipresentasikan saat di-drop di akhir fungsi ini

        Ok(())
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    info!("=== APLIKASI WGPU v30 ANDROID DIMULAI ===");

    let mut animating = false;
    let mut destroy = false;
    let mut wgpu_state: Option<WgpuState> = None;

    while !destroy {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(MainEvent::InitWindow { .. }) => {
                    info!("Window Android aktif! Menyiapkan wgpu surface...");
                    if let Some(window) = app.native_window() {
                        match pollster::block_on(WgpuState::new(window)) {
                            Ok(state) => {
                                wgpu_state = Some(state);
                                animating = true;
                            }
                            Err(e) => {
                                error!("Gagal menginisialisasi WgpuState: {}", e);
                            }
                        }
                    }
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                    info!("Window Android ditutup.");
                    animating = false;
                    wgpu_state = None;
                }
                PollEvent::Main(MainEvent::GainedFocus) => {
                    info!("Aplikasi fokus.");
                }
                PollEvent::Main(MainEvent::LostFocus) => {
                    info!("Aplikasi kehilangan fokus.");
                }
                PollEvent::Main(MainEvent::Destroy) => {
                    info!("Aplikasi dihancurkan.");
                    destroy = true;
                }
                _ => {}
            }
        });

        if animating {
            if let Some(ref mut state) = wgpu_state {
                if let Err(_) = state.render() {
                    state.resize(state.config.width, state.config.height);
                }
            }
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
