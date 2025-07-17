//! src/overlay_meter_plugin.rs
//! Updated OverlayMeter Rainmeter plugin using the new Rust‑native API

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{env, fs, thread};

use tao::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowA, FindWindowExA, GetShellWindow, HWND_TOPMOST, MB_OK, MessageBoxA,
    SMTO_NORMAL, SWP_NOACTIVATE, SWP_SHOWWINDOW, SendMessageTimeoutA, SetParent, SetWindowPos,
};
use windows::core::{BOOL, PCSTR, PCWSTR, w};

use wry::{WebContext, WebViewBuilder};

use rainmeter::*;
/// Move a window into the desktop layer
fn set_window_layer(
    window: &tao::window::Window,
    ctx: &RainmeterContext,
) -> windows::core::Result<()> {
    let hwnd = HWND(window.hwnd() as _);
    // Find Progman
    ctx.log(RmLogLevel::LogDebug, "WebNative: Finding Progman...");
    let desktop = unsafe { GetShellWindow() };
    let res = unsafe { SetParent(hwnd, Some(desktop)) };
    if res.is_err() {
        ctx.log(
            RmLogLevel::LogError,
            &format!("WebNative: SetParent (reparent) failed: {:?}", res.err()),
        );
    }
    Ok(())
}

/// Our plugin type
#[derive(Default)]
struct OverlayMeter {
    url: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    hwnd: Option<isize>,
}

impl OverlayMeter {
    /// Move/resize the overlay window
    fn reposition(&self, ctx: &RainmeterContext) {
        if let Some(raw) = self.hwnd {
            let hwnd = HWND(raw as _);
            unsafe {
                let result = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    self.x,
                    self.y,
                    self.width as i32,
                    self.height as i32,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
                if let Err(e) = result {
                    // level 1 == LOG_ERROR
                    ctx.log(
                        RmLogLevel::LogError,
                        &format!("SetWindowPos failed: {:?}", e),
                    );
                }
            }
        }
    }
}

fn make_webview_data_dir(rm: &RainmeterContext) -> PathBuf {
    // 1) Base it on %LOCALAPPDATA%\Rainmeter\OverlayMeter
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            rm.log(
                RmLogLevel::LogWarning,
                "LOCALAPPDATA not found; using current directory for WebView data",
            );
            env::current_dir().expect("Could not get current directory")
        })
        .join("Rainmeter")
        .join("OverlayMeter");

    // 2) Create it (and parents)
    if let Err(e) = fs::create_dir_all(&base) {
        rm.log(
            RmLogLevel::LogError,
            &format!("Failed to create WebView data dir {:?}: {}", base, e),
        );
    }

    // 3) Tell the user where we’ll store it
    rm.log(
        RmLogLevel::LogNotice,
        &format!("WebView2 user data folder: {:?}", base),
    );
    base
}

impl OverlayMeter {
    fn load_data(&mut self, rm: &RainmeterContext) {
        // Read options from the skin
        self.url = rm.read_string("url", &self.url);
        self.width = rm.read_formula("width", self.width as f64) as u32;
        self.height = rm.read_formula("height", self.height as f64) as u32;
        self.x = rm.read_formula("x", self.x as f64) as i32;
        self.y = rm.read_formula("y", self.y as f64) as i32;
    }
}

impl RainmeterPlugin for OverlayMeter {
    fn initialize(&mut self, _rm: RainmeterContext) {
        // Spawn the WebView on a background thread
        let (tx, rx) = channel();
        let url = self.url.clone();
        let w = self.width;
        let h = self.height;
        let x = self.x;
        let y = self.y;
        self.load_data(&_rm);
        _rm.log(
            RmLogLevel::LogNotice,
            &format!(
                "Initializing OverlayMeter plugin with url={}, width={}, height={}, x={}, y={}",
                url, w, h, x, y
            ),
        );

        let ctx = _rm.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                // Prepare the thread for COM
                // 1) Initialize COM for this thread
                /*unsafe {
                    CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                        .ok()
                        .expect("CoInitializeEx failed");
                }*/

                let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

                let window = WindowBuilder::new()
                    .with_decorations(false)
                    .with_transparent(true)
                    //.with_always_on_top(false)
                    //.with_inner_size(LogicalSize::new(w, h))
                    //.with_position(LogicalPosition::new(x, y))
                    .build(&event_loop)
                    .expect("Failed to create window");

                // Parent under desktop layer
                set_window_layer(&window, &ctx).expect("Failed to set window layer");

                // Send back the HWND
                let hwnd = window.hwnd() as isize;
                tx.send(Some(hwnd)).unwrap();
                ctx.log(
                    RmLogLevel::LogNotice,
                    format!("Window Init HWND: {:?}", hwnd).as_str(),
                );

                // Launch WebView
                /*let mut ctx = WebContext::new(Some(PathBuf::from(
                    "C:\\Users\\Kitsune\\Documents\\datadir",
                )));*/
                let data_dir = make_webview_data_dir(&ctx);
                ctx.log(
                    RmLogLevel::LogNotice,
                    &format!("WebView2 user data folder: {:?}", data_dir),
                );
                let mut ctx = WebContext::new(Some(data_dir));
                unsafe {
                    MessageBoxA(
                        None,
                        PCSTR(
                            format!("We are about to build the context. HWND {:?}\0", window)
                                .as_ptr(),
                        ),
                        PCSTR(b"DEBUG\0".as_ptr()),
                        MB_OK,
                    );
                }
                let _wv = WebViewBuilder::new_with_web_context(&mut ctx)
                    .with_url("https://www.example.com") // or any URL/HTML you like
                    .build(&window)
                    .expect("Failed to build WebView");

                event_loop.run(move |event, _, control_flow| {
                    *control_flow = ControlFlow::Poll;
                    if let Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } = event
                    {
                        *control_flow = ControlFlow::Exit;
                    }
                });
            });

            if let Err(err) = result {
                let msg = err
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| err.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non‑string panic>".into());
                ctx.log(RmLogLevel::LogError, &format!("WebEngine panic: {}", msg));
                let _ = tx.send(None);
            }
        });

        if let Ok(raw) = rx.recv() {
            _rm.log(
                RmLogLevel::LogNotice,
                &format!("Using OverlayMeter HWND: {:?}", raw),
            );
            self.hwnd = raw;
        }
    }

    fn reload(&mut self, rm: RainmeterContext, _max_value: &mut f64) {
        rm.log(RmLogLevel::LogNotice, "Reloading OverlayMeter plugin...");
        // Read options from the skin
        self.load_data(&rm);
        rm.log(
            RmLogLevel::LogNotice,
            &format!(
                "OverlayMeter: url={}, width={}, height={}, x={}, y={}",
                self.url, self.width, self.height, self.x, self.y
            ),
        );
        // Reposition the WebView window
        self.reposition(&rm);
    }

    fn update(&mut self, _rm: RainmeterContext) -> f64 {
        //self.reposition();
        0.0
    }

    fn get_string(&mut self, _rm: RainmeterContext) -> Option<String> {
        None
    }

    fn execute_bang(&mut self, _rm: RainmeterContext, _args: &str) {
        // no-op
    }

    fn finalize(&mut self, _rm: RainmeterContext) {
        // Clean-up if needed
    }
}

// Declare the plugin entry points
declare_plugin!(crate::OverlayMeter);
