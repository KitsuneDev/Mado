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
    GWL_EXSTYLE, GetWindowLongW, SetWindowLongW, WS_EX_TRANSPARENT,
};

use rainmeter::*;
use wry::{WebContext, WebViewBuilder};

/// Create a writable user-data folder under %LOCALAPPDATA%\Rainmeter\OverlayMeter
fn make_webview_data_dir(rm: &RainmeterContext) -> PathBuf {
    let dir = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            rm.log(RmLogLevel::LogWarning, "LOCALAPPDATA not found; using CWD");
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
            // recompute skin rect
            let skin = HWND(rm.get_skin_window_raw() as _);
            let mut rect = windows::Win32::Foundation::RECT::default();
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::GetWindowRect(skin, &mut rect);
                windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    HWND(raw as _),
                    Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                    rect.left + self.x,
                    rect.top + self.y,
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
        self.load_data(&rm);
        rm.log(
            RmLogLevel::LogNotice,
            &format!(
                "Overlay init: url={} size={}×{} pos={},{}",
                self.url, self.width, self.height, self.x, self.y
            ),
        );
        // Prepare receiver
        let (tx, rx) = channel();
        self.rx = Some(rx);
        let url = self.url.clone();
        let (w, h, x, y) = (self.width, self.height, self.x, self.y);
        //let parent_hwnd_raw = rm.get_skin_window_raw() as isize;
        let parent_hwnd = rm.get_skin_window_raw() as isize;
        rm.log(
            RmLogLevel::LogDebug,
            &format!("Parent HWND = 0x{:x}", parent_hwnd),
        );
        let thread_ctx = rm.clone();
        thread::spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();
            }
            // 1) Figure out the skin's screen position
            let skin = HWND(parent_hwnd as _);
            let mut rect = windows::Win32::Foundation::RECT::default();
            let _ =
                unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowRect(skin, &mut rect) };

            // 2) Compute our window's absolute screen coords
            let screen_x = rect.left + x;
            let screen_y = rect.top + y;

            // 3) Build a _top-level_ always-on-top window (not a child!)
            let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
            let window = WindowBuilder::new()
                .with_decorations(false)
                .with_transparent(false)
                .with_always_on_top(true)
                .with_skip_taskbar(true)
                .with_inner_size(LogicalSize::new(w, h))
                .with_position(LogicalPosition::new(screen_x, screen_y))
                .build(&event_loop)
                .expect("WindowBuilder failed");

            // Ensure WS_EX_TRANSPARENT is cleared so we receive input
            let hwnd = HWND(window.hwnd() as _);
            unsafe {
                let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                SetWindowLongW(hwnd, GWL_EXSTYLE, style & !(WS_EX_TRANSPARENT.0 as i32));
            }
            tx.send(window.hwnd() as isize).unwrap();
            thread_ctx.log(
                RmLogLevel::LogDebug,
                &format!("Child HWND = 0x{:x}", hwnd.0 as isize),
            );
            // WebView2
            let data_dir = make_webview_data_dir(&thread_ctx);
            let mut webctx = WebContext::new(Some(data_dir));

            let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Red BG</title>
  <style>
    html, body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      background: red;
    }
  </style>
</head>
<body>
  <!-- your content here -->
</body>
</html>
"#;

            WebViewBuilder::new_with_web_context(&mut webctx)
                .with_transparent(false)
                //.with_url("https://www.example.com")
                .with_html(html_content)
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
    }
    fn update(&mut self, rm: RainmeterContext) -> f64 {
        if self.hwnd.is_none() {
            if let Some(rx) = &self.rx {
                if let Ok(hwnd) = rx.try_recv() {
                    rm.log(
                        RmLogLevel::LogNotice,
                        &format!("Overlay HWND = 0x{:x}", hwnd),
                    );
                    self.hwnd = Some(hwnd);
                }
            }
        }
        self.reposition(&rm);
        0.0
    }
    //fn set_option(&mut self, _key: &str, _val: &str) {}
    fn reload(&mut self, rm: RainmeterContext, _max: &mut f64) {
        self.load_data(&rm);
        self.reposition(&rm);
    }
    fn get_string(&mut self, _rm: RainmeterContext) -> Option<String> {
        None
    }
    fn execute_bang(&mut self, _rm: RainmeterContext, _args: &str) {}
    fn finalize(&mut self, _rm: RainmeterContext) {}
}

declare_plugin!(crate::OverlayMeter);
