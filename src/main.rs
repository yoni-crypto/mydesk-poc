mod utils;
mod commands;

use std::{cell::RefCell, rc::Rc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebView, WebViewBuilder};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("MyDesk POC")
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 640.0))
        .build(&event_loop)?;

    let html = include_str!("frontend.html");

    // WebView storage
    let webview_rc: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));
    let webview_clone = Rc::clone(&webview_rc);

    // Commands storage
    let commands = Rc::new(commands::get_commands());
    let commands_clone = Rc::clone(&commands);

    let webview = WebViewBuilder::new()
        .with_html(html)

        // 🔥 IMPORTANT: Create window.ipc
        .with_initialization_script(r#"
            window.ipc = {
                postMessage: (msg) => window.external.invoke(msg)
            };
        "#)

        .with_ipc_handler(move |req: Request<String>| {
            let body = req.body();

            // Print raw body to debug
            println!("IPC RAW: {}", body);

            if let Ok(value) = serde_json::from_str::<Value>(body) {
                if let Some(method) = value.get("method").and_then(|v| v.as_str()) {
                    if let Some(cmd) = commands_clone.get(method) {
                        cmd(value.clone(), Rc::clone(&webview_clone));
                    }
                }

                // handle simple logs
                if value.get("type").and_then(|v| v.as_str()) == Some("log") {
                    if let Some(msg) = value.get("msg") {
                        println!("js log: {:?}", msg);
                    }
                }
            }
        })
        .build(&window)?;

    *webview_rc.borrow_mut() = Some(webview);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
