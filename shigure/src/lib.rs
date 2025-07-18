// src/overlay_meter_plugin.rs
// Shigure — Rainmeter plugin embedding Wry/WebView2 as a child of the Rainmeter skin window

use std::{
    env, fs,
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
    thread,
};

use tao::platform::windows::{
    EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows,
}; // for `.with_parent_window()`
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongW, SetWindowLongW, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

use rainmeter::*;
use wry::{WebContext, WebViewBuilder};

/// Ensure a writable data folder under %LOCALAPPDATA%\Rainmeter\OverlayMeter
fn make_webview_data_dir(rm: &RainmeterContext) -> PathBuf {
    let dir = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            rm.log(
                RmLogLevel::LogWarning,
                "LOCALAPPDATA not found; using current directory",
            );
            env::current_dir().unwrap()
        })
        .join("Rainmeter")
        .join("OverlayMeter");
    let _ = fs::create_dir_all(&dir);
    rm.log(
        RmLogLevel::LogNotice,
        &format!("WebView2 user data folder: {:?}", dir),
    );
    dir
}

/// Plugin state
#[derive(Default)]
struct OverlayMeter {
    url: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    hwnd: Option<isize>,
    rx: Option<Receiver<isize>>,
}

impl OverlayMeter {
    fn load_data(&mut self, rm: &RainmeterContext) {
        self.url = rm.read_string("url", &self.url);
        self.width = rm.read_formula("width", self.width as f64) as u32;
        self.height = rm.read_formula("height", self.height as f64) as u32;
        self.x = rm.read_formula("x", self.x as f64) as i32;
        self.y = rm.read_formula("y", self.y as f64) as i32;
    }

    fn reposition(&self, rm: &RainmeterContext) {
        if let Some(raw) = self.hwnd {
            let hwnd = HWND(raw as _);
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                    self.x,
                    self.y,
                    self.width as i32,
                    self.height as i32,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE
                        | windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
                )
                .ok();
            }
        }
    }
}

impl RainmeterPlugin for OverlayMeter {
    fn initialize(&mut self, rm: RainmeterContext) {
        // Load initial skin options
        self.load_data(&rm);
        rm.log(
            RmLogLevel::LogNotice,
            &format!(
                "Overlay init: url={} size={}×{} pos={},{}",
                self.url, self.width, self.height, self.x, self.y
            ),
        );

        // Prepare channel for HWND
        let (tx, rx) = channel();
        self.rx = Some(rx);
        let url = self.url.clone();
        let (w, h, x, y) = (self.width, self.height, self.x, self.y);

        // Rainmeter skin window handle to parent our child
        let parent_hwnd = rm.get_skin_window_raw() as isize;
        rm.log(
            RmLogLevel::LogDebug,
            &format!("Parent HWND = 0x{:x}", parent_hwnd),
        );

        let thread_ctx = rm.clone();
        thread::spawn(move || {
            // Initialize COM on this thread for WebView2
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();
            }

            // Build Tao event loop & child window
            let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
            let window = WindowBuilder::new()
                .with_decorations(false)
                .with_transparent(true) // allow per-pixel transparency
                .with_parent_window(parent_hwnd)
                .with_inner_size(LogicalSize::new(w, h))
                .with_position(LogicalPosition::new(x, y))
                .build(&event_loop)
                .expect("Failed to create child window");

            // Fix styles: keep layered, remove hit-test transparent
            let raw = window.hwnd() as isize;
            let hwnd = HWND(raw as _);
            unsafe {
                let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                let new_style = (style | WS_EX_LAYERED.0 as i32) & !(WS_EX_TRANSPARENT.0 as i32);
                SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
            }

            // Send handle back to main thread
            tx.send(raw).unwrap();
            thread_ctx.log(RmLogLevel::LogDebug, &format!("Child HWND = 0x{:x}", raw));

            // WebView2 initialization
            let data_dir = make_webview_data_dir(&thread_ctx);
            let mut webctx = WebContext::new(Some(data_dir));

            // Inline test HTML: red background to verify rendering
            let html =
                r#"<!DOCTYPE html><html><body style='margin:0;background:red;'></body></html>"#;
            let _wv = WebViewBuilder::new_with_web_context(&mut webctx)
                .with_transparent(true)
                .with_html(html)
                .build(&window)
                .expect("Failed to build WebView");

            // Enter event loop
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
    }

    fn reload(&mut self, rm: RainmeterContext, _max: &mut f64) {
        self.load_data(&rm);
        self.reposition(&rm);
    }

    fn update(&mut self, rm: RainmeterContext) -> f64 {
        // Try retrieving HWND once
        if self.hwnd.is_none() {
            if let Some(rx) = &self.rx {
                if let Ok(raw) = rx.try_recv() {
                    rm.log(
                        RmLogLevel::LogNotice,
                        &format!("Overlay HWND = 0x{:x}", raw),
                    );
                    self.hwnd = Some(raw);
                }
            }
        }
        // Move/resize child window
        self.reposition(&rm);
        0.0
    }

    fn get_string(&mut self, _rm: RainmeterContext) -> Option<String> {
        None
    }
    fn execute_bang(&mut self, _rm: RainmeterContext, _args: &str) {}
    fn finalize(&mut self, _rm: RainmeterContext) {}
}

declare_plugin!(crate::OverlayMeter);
