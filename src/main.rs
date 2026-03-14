mod commands_async;
mod error;
mod ipc;
mod utils;

use std::sync::mpsc;
use std::{cell::RefCell, rc::Rc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebView, WebViewBuilder};

use crate::ipc::IpcRequest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("MyDesk POC")
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 640.0))
        .build(&event_loop)?;

    let html = include_str!("frontend.html");

    // Channel for sending JS back to the main thread
    let (tx, rx) = mpsc::channel::<String>();

    // WebView storage
    let webview_rc: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));
    let webview_clone = Rc::clone(&webview_rc);

    // Commands storage
    let commands = Rc::new(commands_async::get_commands());
    let commands_clone = Rc::clone(&commands);

    let webview = WebViewBuilder::new()
        .with_html(html)
        .with_initialization_script(r#"
            window.ipc = {
                postMessage: (msg) => window.external.invoke(msg)
            };
            window.__mydeskHandleResponse = (response) => {
                if (window.fromNative) window.fromNative(response);
            };
        "#)
        .with_ipc_handler(move |req: Request<String>| {
            let body = req.body();
            println!("IPC received: {}", body);

            match serde_json::from_str::<IpcRequest>(body) {
                Ok(ipc_req) => {
                    if let Some(cmd) = commands_clone.get(&ipc_req.method) {
                        cmd(ipc_req, tx.clone());
                    } else {
                        let response = ipc::IpcResponse::error(
                            ipc_req.id,
                            error::MyDeskError::UnknownCommand(ipc_req.method.clone()),
                        );
                        let _ = tx.send(response.to_js_call());
                    }
                }
                Err(e) => eprintln!("Failed to parse IPC request: {}", e),
            }
        })
        .build(&window)?;

    *webview_rc.borrow_mut() = Some(webview);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Drain pending JS calls from background threads
        while let Ok(js) = rx.try_recv() {
            if let Some(webview) = webview_clone.borrow().as_ref() {
                let _ = webview.evaluate_script(&js);
            }
        }

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
