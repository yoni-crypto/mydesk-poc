mod commands_async;
mod error;
mod ipc;
mod utils;

use std::{cell::RefCell, rc::Rc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebView, WebViewBuilder};

use crate::ipc::IpcRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let commands = Rc::new(commands_async::get_commands());
    let commands_clone = Rc::clone(&commands);

    let webview = WebViewBuilder::new()
        .with_html(html)
        // Initialize IPC and response handler
        .with_initialization_script(r#"
            window.ipc = {
                postMessage: (msg) => window.external.invoke(msg)
            };
            
            // Response handler for async commands
            window.__mydeskHandleResponse = (response) => {
                if (window.fromNative) {
                    window.fromNative(response);
                }
            };
        "#)
        .with_ipc_handler(move |req: Request<String>| {
            let body = req.body();
            println!("IPC received: {}", body);

            // Parse IPC request
            match serde_json::from_str::<IpcRequest>(body) {
                Ok(ipc_req) => {
                    println!("Command: {} (id: {})", ipc_req.method, ipc_req.id);
                    
                    if let Some(cmd) = commands_clone.get(&ipc_req.method) {
                        cmd(ipc_req, Rc::clone(&webview_clone));
                    } else {
                        println!("Unknown command: {}", ipc_req.method);
                        let response = ipc::IpcResponse::error(
                            ipc_req.id,
                            error::MyDeskError::UnknownCommand(ipc_req.method.clone()),
                        );
                        if let Some(webview) = webview_clone.borrow().as_ref() {
                            let _ = webview.evaluate_script(&response.to_js_call());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse IPC request: {}", e);
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
