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
    async fn new(window: ndk::native_window::NativeWindow) -> Option<Self> {
        let width = window.width() as u32;
        let height = window.height() as u32;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Membuat surface secara aman dari NDK NativeWindow menggunakan target unsafe raw handle
        let win_ptr = window.ptr().as_ptr();
        let p_window = raw_window_handle::AndroidDisplayHandle::new();
        let p_display = raw_window_handle::AndroidWindowHandle::new(
            std::num::NonZeroU32::new(win_ptr as u32).unwrap()
        );

        // Menggunakan unsafe raw handle creation untuk NDK Android
        let surface_target = unsafe {
            wgpu::SurfaceTargetUnsafe::from_window(&window).ok()?
        };
        let surface = unsafe { instance.create_surface_unsafe(surface_target).ok()? };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
.ok()?;

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

        Some(Self {
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
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
                        // Warna latar belakang ungu gelap khas GD
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.22,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

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
                        let state = pollster::block_on(WgpuState::new(window));
                        if state.is_some() {
                            wgpu_state = state;
                            animating = true;
                        } else {
                            error!("Gagal menginisialisasi WgpuState!");
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
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.config.width, state.config.height);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        error!("Wgpu out of memory!");
                        destroy = true;
                    }
                    Err(e) => {
                        error!("Surface error: {:?}", e);
                    }
                }
            }
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
