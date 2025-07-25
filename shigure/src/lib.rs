// src/overlay_meter_plugin.rs
// Shigure — Rainmeter plugin embedding Wry/WebView2 as a non-resizable child
// with clean shutdown and dynamic URL updates

use std::{
    env, fs,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
    thread,
};

use mado::events::EventRaiser;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use tao::platform::{
    run_return::EventLoopExtRunReturn,
    windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows},
};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GWL_EXSTYLE, GWL_STYLE, GetWindowLongW, SetWindowLongW, WS_BORDER, WS_CAPTION,
    WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_THICKFRAME,
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{LWA_ALPHA, SetLayeredWindowAttributes},
};

use rainmeter::*;
use softbuffer::{Context as SoftbufferContext, Surface as SoftbufferSurface};
use wry::{WebContext, WebViewBuilder, WebViewBuilderExtWindows};
use wry_cmd::use_wry_cmd_protocol;

mod events;
mod services;

/// Rainmeter context stored globally for services and events.
static RAINMETER_CTX: Lazy<parking_lot::RwLock<Option<Arc<RainmeterContext>>>> =
    Lazy::new(|| parking_lot::RwLock::new(None));
pub fn get_rainmeter() -> Option<Arc<RainmeterContext>> {
    RAINMETER_CTX.read().clone()
}

/// Global command sender for broadcasting `Command::Event` to the WebView thread.
static GLOBAL_CMD_TX: Lazy<Mutex<Option<Sender<Command>>>> = Lazy::new(|| Mutex::new(None));

/// Raise an event from any thread via the global command channel.
pub fn raise_event<T: Serialize>(event: mado::events::Event<T>) {
    if let Ok(json) = serde_json::to_string(&event) {
        if let Some(tx) = GLOBAL_CMD_TX.lock().as_ref() {
            let _ = tx.send(Command::Event(json));
        }
    }
}

/// Helper: set up the global command sender (called during plugin initialization)
fn init_global_cmd_tx(tx: Sender<Command>) {
    *GLOBAL_CMD_TX.lock() = Some(tx);
}

fn make_webview_data_dir(rm: &RainmeterContext) -> PathBuf {
    let dir = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            rm.log(RmLogLevel::LogWarning, "LOCALAPPDATA missing; using CWD");
            env::current_dir().unwrap()
        })
        .join("Rainmeter")
        .join("OverlayMeter");
    let _ = fs::create_dir_all(&dir);
    rm.log(
        RmLogLevel::LogNotice,
        &format!("WebView2 data dir: {:?}", dir),
    );
    dir
}

struct OverlayMeter {
    url: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,

    hwnd_rx: Option<Receiver<isize>>,
    cmd_tx: Option<Sender<Command>>,
    shutdown_tx: Option<Sender<()>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    hwnd: Option<isize>,
}

impl Default for OverlayMeter {
    fn default() -> Self {
        Self {
            url: "https://example.com".into(),
            width: 300,
            height: 200,
            x: 0,
            y: 0,
            hwnd_rx: None,
            cmd_tx: None,
            shutdown_tx: None,
            thread_handle: None,
            hwnd: None,
        }
    }
}

impl OverlayMeter {
    fn load_data(&mut self, rm: &RainmeterContext) {
        self.url = rm.read_string("url", &self.url);
        self.width = rm.read_formula("width", self.width as f64) as u32;
        self.height = rm.read_formula("height", self.height as f64) as u32;
        self.x = rm.read_formula("x", self.x as f64) as i32;
        self.y = rm.read_formula("y", self.y as f64) as i32;
    }

    fn reposition(&self) {
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

enum Command {
    Event(String),     // JSON event payload
    UpdateUrl(String), // URL update command
}

impl EventRaiser for OverlayMeter {
    fn raise_event<T: Serialize>(&self, event: mado::events::Event<T>) {
        // Local instance-based raise_event
        if let Ok(json) = serde_json::to_string(&event) {
            if let Some(tx) = &self.cmd_tx {
                let _ = tx.send(Command::Event(json));
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

        let (hwnd_tx, hwnd_rx) = channel::<isize>();
        let (cmd_tx, cmd_rx) = channel::<Command>();
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        self.hwnd_rx = Some(hwnd_rx);
        self.cmd_tx = Some(cmd_tx.clone());
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Initialize the global sender so other threads can call `raise_event`
        init_global_cmd_tx(cmd_tx.clone());

        let url = self.url.clone();
        let (w, h, x, y) = (self.width, self.height, self.x, self.y);
        let parent = rm.get_skin_window_raw() as isize;
        let thread_ctx = rm.clone();

        let handle = thread::spawn(move || {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();
            }
            let mut event_loop = EventLoopBuilder::new().with_any_thread(true).build();
            let window = WindowBuilder::new()
                .with_decorations(false)
                .with_resizable(false)
                .with_transparent(true)
                .with_parent_window(parent)
                .with_inner_size(LogicalSize::new(w, h))
                .with_position(LogicalPosition::new(x, y))
                .build(&event_loop)
                .expect("Failed to create child window");

            let raw = window.hwnd() as isize;
            hwnd_tx.send(raw).unwrap();
            thread_ctx.log(RmLogLevel::LogDebug, &format!("Child HWND = 0x{:x}", raw));

            let window = Rc::new(window);
            let sb_context =
                SoftbufferContext::new(window.clone()).expect("SoftbufferContext failed");
            let mut sb_surface = SoftbufferSurface::new(&sb_context, window.clone())
                .expect("SoftbufferSurface failed");

            let data_dir = make_webview_data_dir(&thread_ctx);
            let mut webctx = WebContext::new(Some(data_dir));
            let wv = WebViewBuilder::new_with_web_context(&mut webctx)
                .with_transparent(true)
                .with_background_color((0, 0, 0, 0))
                .with_url(&url)
                .with_asynchronous_custom_protocol(
                    "mado".to_string(),
                    use_wry_cmd_protocol!("mado"),
                )
                .with_https_scheme(true)
                .build(&window)
                .expect("Failed to build WebView");

            window.request_redraw();
            event_loop.run_return(move |event, _, control_flow| {
                if shutdown_rx.try_recv().is_ok() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if let Ok(next) = cmd_rx.try_recv() {
                    match next {
                        Command::Event(json) => {
                            let script = format!("if(window.ipcEvent)window.ipcEvent({});", json);
                            let _ = wv.evaluate_script(&script);
                        }
                        Command::UpdateUrl(new_url) => {
                            thread_ctx.log(
                                RmLogLevel::LogNotice,
                                &format!("Updating URL to: {}", new_url),
                            );
                            wv.load_url(&new_url).unwrap();
                        }
                    }
                }
                match event {
                    Event::RedrawRequested(_) => {
                        use std::num::NonZeroU32;
                        let size = window.inner_size();
                        sb_surface
                            .resize(
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            )
                            .unwrap();
                        let mut buffer = sb_surface.buffer_mut().unwrap();
                        buffer.fill(0);
                        buffer.present().unwrap();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => *control_flow = ControlFlow::Poll,
                }
            });
        });
        self.thread_handle = Some(handle);
    }

    fn reload(&mut self, rm: RainmeterContext, _max: &mut f64) {
        let old = self.url.clone();
        self.load_data(&rm);
        self.reposition();
        if self.url != old {
            if let Some(tx) = &self.cmd_tx {
                let _ = tx.send(Command::UpdateUrl(self.url.clone()));
            }
        }
    }

    fn update(&mut self, rm: RainmeterContext) -> f64 {
        *RAINMETER_CTX.write() = Some(Arc::new(rm.clone()));
        if self.hwnd.is_none() {
            if let Some(rx) = &self.hwnd_rx {
                if let Ok(raw) = rx.try_recv() {
                    rm.log(
                        RmLogLevel::LogNotice,
                        &format!("Overlay HWND = 0x{:x}", raw),
                    );
                    self.hwnd = Some(raw);
                }
            }
        }
        self.reposition();
        self.poll_updates(&rm);
        0.0
    }

    fn finalize(&mut self, _rm: RainmeterContext) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    fn get_string(&mut self, _rm: RainmeterContext) -> Option<String> {
        None
    }
    fn execute_bang(&mut self, _rm: RainmeterContext, _args: &str) {}
}

declare_plugin!(crate::OverlayMeter);
