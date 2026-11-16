use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::{error, info};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

static ENTERED: AtomicBool = AtomicBool::new(false);

const SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 3.0,  1.0),
    );
    return vec4<f32>(pos[idx], 0.0, 1.0);
}
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(frag_coord.x / 1080.0, 1.0 - frag_coord.y / 1920.0);
    return textureSample(tex, samp, uv);
}
"#;

struct WgpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    _window: ndk::native_window::NativeWindow,
}

impl WgpuState {
    async fn new(window: ndk::native_window::NativeWindow, png_bytes: &[u8]) -> Result<Self, String> {
        info!("[wgpu] start new");
        let width = window.width() as u32;
        let height = window.height() as u32;
        let safe_width = if width > 0 { width } else { 1080 };
        let safe_height = if height > 0 { height } else { 1920 };
        info!("[wgpu] size {}x{}", safe_width, safe_height);

        let instance = wgpu::Instance::default();
        let win_ptr = window.ptr().as_ptr();
        info!("[wgpu] win_ptr = {:?}", win_ptr);
        if win_ptr.is_null() {
            return Err("win_ptr null".into());
        }

        let raw_display = raw_window_handle::RawDisplayHandle::Android(
            raw_window_handle::AndroidDisplayHandle::new(),
        );
        let raw_window = raw_window_handle::RawWindowHandle::AndroidNdk(
            raw_window_handle::AndroidNdkWindowHandle::new(
                std::ptr::NonNull::new(win_ptr as *mut std::ffi::c_void).unwrap(),
            ),
        );

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(raw_display),
                    raw_window_handle: raw_window,
                })
                .map_err(|e| format!("create_surface: {:?}", e))?
        };
        info!("[wgpu] surface ok");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|e| format!("request_adapter: {:?}", e))?;
        info!("[wgpu] adapter ok");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|e| format!("request_device: {:?}", e))?;
        info!("[wgpu] device ok");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: safe_width,
            height: safe_height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Srgb,
        };
        surface.configure(&device, &config);

        // PNG decode (kalau kosong, fallback ke warna solid biar tidak error)
        let (view, bind_group_layout) = if png_bytes.is_empty() {
            info!("[wgpu] png empty, make 1x1 red texture");
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fallback"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255u8, 0, 0, 255],
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
                wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            );
            let v = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let l = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            (v, l)
        } else {
            let img = image::load_from_memory(png_bytes)
                .map_err(|e| format!("decode png: {}", e))?
                .to_rgba8();
            let (tw, th) = img.dimensions();
            info!("[wgpu] png {}x{}", tw, th);
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("png"),
                size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * tw), rows_per_image: Some(th) },
                wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
            );
            let v = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let l = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
            (v, l)
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipe"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        info!("[wgpu] pipeline ok");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            _window: window,
        })
    }

    fn render(&mut self) -> Result<(), String> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            other => return Err(format!("get_current_texture: {:?}", other)),
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rp"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.bind_group, &[]);
            rp.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);
        Ok(())
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    // Guard: kalau android_main dipanggil ulang di proses yang sama, keluar saja
    if ENTERED.swap(true, Ordering::SeqCst) {
        error!("[main] android_main dipanggil ulang di proses yang sama, exit.");
        std::process::exit(0);
    }

    // Panic hook: cetak panic ke logcat
    std::panic::set_hook(Box::new(|info| {
        error!("[panic] {}", info);
    }));

    info!("=== START ===");

    let png_bytes = std::fs::read("assets/play.png").unwrap_or_else(|e| {
        error!("[main] read play.png: {}", e);
        Vec::new()
    });
    info!("[main] png bytes len = {}", png_bytes.len());

    let mut state: Option<WgpuState> = None;
    let mut destroy = false;

    while !destroy {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(MainEvent::InitWindow { .. }) => {
                    info!("[main] InitWindow");
                    if let Some(win) = app.native_window() {
                        match pollster::block_on(WgpuState::new(win, &png_bytes)) {
                            Ok(s) => {
                                info!("[main] WgpuState OK");
                                state = Some(s);
                            }
                            Err(e) => error!("[main] INIT FAIL: {}", e),
                        }
                    } else {
                        error!("[main] InitWindow tapi native_window None");
                    }
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                    info!("[main] TerminateWindow");
                    state = None;
                }
                PollEvent::Main(MainEvent::Destroy) => {
                    info!("[main] Destroy");
                    destroy = true;
                }
                _ => {}
            }
        });

        if let Some(s) = state.as_mut() {
            if let Err(e) = s.render() {
                error!("[main] render: {}", e);
                s.surface.configure(&s.device, &s.config);
            }
        }
    }

    info!("=== END ===");
}
