use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::{error, info};
use std::ffi::c_void;
use std::io::Read;

// --- EGL & GLES 3.2 FFI Declarations ---
type EGLDisplay = *mut c_void;
type EGLSurface = *mut c_void;
type EGLContext = *mut c_void;
type EGLConfig = *mut c_void;
type EGLBoolean = u32;
type EGLInt = i32;

const EGL_NO_DISPLAY: EGLDisplay = 0 as EGLDisplay;
const EGL_NO_SURFACE: EGLSurface = 0 as EGLSurface;
const EGL_NO_CONTEXT: EGLContext = 0 as EGLContext;
const EGL_DEFAULT_DISPLAY: *mut c_void = 0 as *mut c_void;

const EGL_RED_SIZE: EGLInt = 0x3024;
const EGL_GREEN_SIZE: EGLInt = 0x3023;
const EGL_BLUE_SIZE: EGLInt = 0x3025;
const EGL_ALPHA_SIZE: EGLInt = 0x3021;
const EGL_DEPTH_SIZE: EGLInt = 0x3025;
const EGL_SURFACE_TYPE: EGLInt = 0x3033;
const EGL_WINDOW_BIT: EGLInt = 0x0004;
const EGL_RENDERABLE_TYPE: EGLInt = 0x3040;
const EGL_OPENGL_ES3_BIT: EGLInt = 0x0040;
const EGL_NONE: EGLInt = 0x3038;
const EGL_CONTEXT_CLIENT_VERSION: EGLInt = 0x3098;

const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;

unsafe extern "C" {
    fn eglGetDisplay(display_id: *mut c_void) -> EGLDisplay;
    fn eglInitialize(dpy: EGLDisplay, major: *mut EGLInt, minor: *mut EGLInt) -> EGLBoolean;
    fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
    fn eglChooseConfig(dpy: EGLDisplay, attrib_list: *const EGLInt, configs: *mut EGLConfig, config_size: EGLInt, num_config: *mut EGLInt) -> EGLBoolean;
    fn eglCreateContext(dpy: EGLDisplay, config: EGLConfig, share_context: EGLContext, attrib_list: *const EGLInt) -> EGLContext;
    fn eglCreateWindowSurface(dpy: EGLDisplay, config: EGLConfig, win: *mut c_void, attrib_list: *const EGLInt) -> EGLSurface;
    fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    fn eglDestroyContext(dpy: EGLDisplay, context: EGLContext) -> EGLBoolean;
    fn eglMakeCurrent(dpy: EGLDisplay, draw: EGLSurface, read: EGLSurface, ctx: EGLContext) -> EGLBoolean;
    fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
}

unsafe extern "C" {
    fn glClearColor(red: f32, green: f32, blue: f32, alpha: f32);
    fn glClear(mask: u32);
}

struct GraphicsContext {
    display: EGLDisplay,
    surface: EGLSurface,
    context: EGLContext,
    _window: ndk::native_window::NativeWindow, // Menyimpan ownership window agar tidak ter-drop
}

impl GraphicsContext {
    unsafe fn init(window: ndk::native_window::NativeWindow) -> Option<Self> {
        unsafe {
            let win_ptr = window.ptr().as_ptr() as *mut c_void;
            let display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
            if display == EGL_NO_DISPLAY {
                error!("Gagal mendapatkan EGL display");
                return None;
            }

            let mut major = 0;
            let mut minor = 0;
            if eglInitialize(display, &mut major, &mut minor) == 0 {
                error!("Gagal inisialisasi EGL");
                return None;
            }
            info!("EGL initialized successfully. Version: {}.{}", major, minor);

            let attribs = [
                EGL_RED_SIZE, 8,
                EGL_GREEN_SIZE, 8,
                EGL_BLUE_SIZE, 8,
                EGL_ALPHA_SIZE, 8,
                EGL_DEPTH_SIZE, 16,
                EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
                EGL_NONE,
            ];

            let mut config: EGLConfig = std::ptr::null_mut();
            let mut num_configs = 0;
            if eglChooseConfig(display, attribs.as_ptr(), &mut config, 1, &mut num_configs) == 0 || num_configs == 0 {
                error!("Gagal memilih EGL config yang cocok untuk GLES 3");
                return None;
            }

            let context_attribs = [
                EGL_CONTEXT_CLIENT_VERSION, 3,
                EGL_NONE,
            ];
            let context = eglCreateContext(display, config, EGL_NO_CONTEXT, context_attribs.as_ptr());
            if context == EGL_NO_CONTEXT {
                error!("Gagal membuat EGL context untuk OpenGL ES 3");
                return None;
            }

            let surface = eglCreateWindowSurface(display, config, win_ptr, std::ptr::null());
            if surface == EGL_NO_SURFACE {
                error!("Gagal membuat EGL window surface");
                eglDestroyContext(display, context);
                return None;
            }

            if eglMakeCurrent(display, surface, surface, context) == 0 {
                error!("Gagal melakukan eglMakeCurrent");
                eglDestroySurface(display, surface);
                eglDestroyContext(display, context);
                return None;
            }

            info!("OpenGL ES 3.2 Context berhasil diaktifkan!");
            Some(Self { display, surface, context, _window: window })
        }
    }
}

impl Drop for GraphicsContext {
    fn drop(&mut self) {
        unsafe {
            if self.display != EGL_NO_DISPLAY {
                eglMakeCurrent(self.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
                if self.surface != EGL_NO_SURFACE {
                    eglDestroySurface(self.display, self.surface);
                }
                if self.context != EGL_NO_CONTEXT {
                    eglDestroyContext(self.display, self.context);
                }
                eglTerminate(self.display);
            }
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    info!("=== APLIKASI NATIVE RUST (PURE GLES 3.2) DIMULAI ===");

    // 1. Load play.png dari Assets
    let asset_manager = app.asset_manager();
    if let Some(mut asset) = asset_manager.open(c"play.png") {
        let mut buffer = Vec::new();
        if asset.read_to_end(&mut buffer).is_ok() {
            match image::load_from_memory(&buffer) {
                Ok(img) => {
                    info!("PNG berhasil didekode! Resolusi: {}x{}", img.width(), img.height());
                }
                Err(e) => {
                    error!("Gagal mendecode PNG: {}", e);
                }
            }
        }
    } else {
        error!("Gagal menemukan play.png di folder assets!");
    }

    let mut animating = false;
    let mut destroy = false;
    let mut gfx: Option<GraphicsContext> = None;

    while !destroy {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(MainEvent::InitWindow { .. }) => {
                    info!("Window Android aktif! Menyiapkan EGL...");
                    if let Some(window) = app.native_window() {
                        gfx = unsafe { GraphicsContext::init(window) };
                        if gfx.is_some() {
                            animating = true;
                        }
                    }
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                    info!("Window Android ditutup!");
                    animating = false;
                    gfx = None;
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
            if let Some(ref g) = gfx {
                unsafe {
                    glClearColor(0.08, 0.08, 0.22, 1.0);
                    glClear(GL_COLOR_BUFFER_BIT);
                    eglSwapBuffers(g.display, g.surface);
                }
            }
        }
    }

    info!("=== APLIKASI KELUAR ===");
}
