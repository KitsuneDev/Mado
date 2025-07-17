// src/bin/test_webview.rs

use std::{env, fs, path::PathBuf};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebContext, WebViewBuilder};

fn make_webview_data_dir() -> PathBuf {
    // 1) Base it on %LOCALAPPDATA%\Rainmeter\OverlayMeter
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap()
        .join("Rainmeter")
        .join("OverlayMeter");

    // 2) Create it (and parents)
    if let Err(e) = fs::create_dir_all(&base) {
        eprintln!("Failed to create WebView data dir {:?}: {}", base, e);
        panic!("Dir Err") // Return even if creation fails, so we can log the path
    }

    // 3) Tell the user where we’ll store it

    base
}

fn main() -> wry::Result<()> {
    // 1) Create the event loop & window
    let event_loop = EventLoop::new();
    // let window = WindowBuilder::new()
    //     .with_title("Wry Test")
    //     .with_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0))
    //     .build(&event_loop)
    //     .expect("Failed to create window");
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_transparent(true)
        //.with_always_on_top(false)
        //.with_inner_size(LogicalSize::new(w, h))
        //.with_position(LogicalPosition::new(x, y))
        .build(&event_loop)
        .expect("Failed to create window");

    // 2) Optionally pick a user‑data dir under %LOCALAPPDATA%\TestWebView
    let data_dir = make_webview_data_dir();
    std::fs::create_dir_all(&data_dir).ok();

    // 3) Create a WebContext (this initializes the WebView2 environment)
    let mut web_context = WebContext::new(Some(data_dir));

    // 4) Build the WebView
    let _webview = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_url("https://www.example.com") // or any URL/HTML you like
        .build(&window)
        .expect("Failed to build WebView");

    // 5) Run the event loop until the user closes the window
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
