use std::ffi::{CString, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::ptr::null_mut;
use rainmeter::*;
use std::sync::mpsc::channel;

// 1) Import the builder & extension
use tao::{
    event_loop::{EventLoopBuilder, ControlFlow},
    dpi::{LogicalPosition, LogicalSize},
    event::Event,
};
use tao::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use tao::window::WindowBuilder;

use windows::core::{BOOL, PCSTR, PCWSTR};
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_SHOWWINDOW, HWND_TOPMOST, MessageBoxW, MB_OK, FindWindowA, SendMessageTimeoutA, SMTO_NORMAL, FindWindowExA, EnumWindows, SetParent};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

use wry::{WebContext, WebViewBuilder};

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

struct OverlayMeter {
    url:    String,
    width:  u32,
    height: u32,
    x:      i32,
    y:      i32,
    hwnd:   Option<isize>,
}

impl Default for OverlayMeter {
    fn default() -> Self {
        Self {
            url:    "https://openai.com".into(),
            width:  300,
            height: 200,
            x:      0,
            y:      0,
            hwnd:   None,
        }
    }
}

impl OverlayMeter {
    fn reposition(&self) {
        if let Some(raw) = self.hwnd {
            // 3) directly wrap the isize
            let hwnd = HWND(raw as *mut _);
            unsafe {
                SetWindowPos(
                    hwnd,
                    Option::from(HWND_TOPMOST),
                    self.x,
                    self.y,
                    self.width as i32,
                    self.height as i32,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                ).expect("Failed to put the window in the desktop layer");
            }
        }
    }
}

impl RainmeterPlugin for OverlayMeter {
    fn initialize(&mut self) {
        let (tx, rx) = channel();
        let url    = self.url.clone();
        let w      = self.width;
        let h      = self.height;
        let x      = self.x;
        let y      = self.y;

        std::thread::spawn(move || {
            let res = std::panic::catch_unwind(|| {
                // 1) Use the builder + any_thread
                let event_loop = EventLoopBuilder::new()
                    .with_any_thread(true)
                    .build();

                let window = WindowBuilder::new()
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_always_on_top(false)
                    .with_inner_size(LogicalSize::new(w, h))
                    .with_position(LogicalPosition::new(x, y))
                    .build(&event_loop)
                    .expect("Failed to configure Window");

                set_window_layer(&window);

                // 3) grab and send the raw handle value
                let raw_handle = window.hwnd() as isize;
                tx.send(raw_handle).unwrap();

                // 4) current wry API
                let mut webcontext = WebContext::new(Some(PathBuf::from("C:\\Users\\Kitsune\\Documents\\datadir")));
                let _wv = WebViewBuilder::new_with_web_context(&mut webcontext)
                    .with_url(&url)
                    .build(&window)
                    .expect("Failed to build Window");

                // 2) Poll so the WebView actually updates
                event_loop.run(move |event, _, control_flow| {
                    *control_flow = ControlFlow::Poll;
                    if let Event::WindowEvent { event: tao::event::WindowEvent::CloseRequested, .. } = event {
                        *control_flow = ControlFlow::Exit;
                    }
                });
            });
            if let Err(err) = res {
                let msg = err
                    .downcast_ref::<&str>()
                    .map(|s| *s)
                    .or_else(|| err.downcast_ref::<String>().map(|s| &**s))
                    .unwrap_or("<non-string panic>");
                let txt = wide(&format!("Thread panic: {}", msg));
                let cap = wide("Error");
                unsafe {
                    MessageBoxW(
                        None,
                        PCWSTR(txt.as_ptr()),
                        PCWSTR(cap.as_ptr()),
                        MB_OK,
                    );
                }
            }
        });

        if let Ok(raw) = rx.recv() {
            self.hwnd = Some(raw);
        }
    }

    fn update(&mut self) -> f64 {
        self.reposition();
        0.0
    }

    fn set_option(&mut self, key: &str, val: &str) {
        match key.to_lowercase().as_str() {
            "url"    => self.url    = val.to_string(),
            "width"  => self.width  = val.parse().unwrap_or(self.width),
            "height" => self.height = val.parse().unwrap_or(self.height),
            "x"      => self.x      = val.parse().unwrap_or(self.x),
            "y"      => self.y      = val.parse().unwrap_or(self.y),
            _        => {}
        }
        self.reposition();
    }

    fn finalize(&mut self) {
        // nothing else to do here for now
    }
}

fn set_window_layer(window: &tao::window::Window) {
    let progman = unsafe {
        let progman_name = c"Progman";
        FindWindowA(PCSTR(progman_name.as_ptr() as *const u8), None).expect("Could not find ProgMan")
    };
    unsafe {
        let _ = SendMessageTimeoutA(
            progman,
            0x052C,       // WM_SPAWN_WORKER
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            None,
        );
    }





    // 2b) Find the WorkerW window that now hosts the desktop icons
    let mut raw_worker: HWND = HWND(ptr::null_mut());
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // Look for the SHELLDLL_DefView child
        if let Ok(defview) = FindWindowExA(
            Some(hwnd),
            None,
            PCSTR(b"SHELLDLL_DefView\0".as_ptr()),
            PCSTR::null(),
        ) {
            // defview.0 is the raw *mut c_void; non-null means we found it
            if !defview.0.is_null() {
                // The sibling WorkerW is our target
                let worker = FindWindowExA(
                    None,
                    Some(hwnd),
                    PCSTR(b"WorkerW\0".as_ptr()),
                    PCSTR::null(),
                )
                    .expect("FindWindowExA(\"WorkerW\") failed");
                // Write it back into raw_worker
                let out_ptr = lparam.0 as *mut HWND;
                *out_ptr = worker;
                return BOOL(0); // stop enumeration
            }
        }
        BOOL(1) // continue
    }

    
    unsafe {
        // Enumerate top‐level windows
        EnumWindows(
            Some(enum_proc),
            LPARAM(&mut raw_worker as *mut HWND as isize),
        )
            .expect("EnumWindows failed");
    }


    let txt = wide("We have the enumeration");
    let cap = wide("Error");
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(txt.as_ptr()),
            PCWSTR(cap.as_ptr()),
            MB_OK,
        );
    }

    // 2c) Finally, re‐parent *your* window into that WorkerW
    // 4) If raw_worker.0 is non-null, we found it—reparent our window
    if !raw_worker.0.is_null() {
        // Tao’s `hwnd()` gives you the raw handle as `isize`, so cast it back
        let our_raw: *mut c_void = window.hwnd() as *mut c_void;
        let our_hwnd = HWND(our_raw);

        unsafe {
            SetParent(our_hwnd, Some(raw_worker))
                .expect("SetParent failed");
        }
    }
}

declare_plugin!(OverlayMeter);
