use std::{rc::Rc, cell::RefCell};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebViewBuilder};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("MyDesk POC")
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 640.0))
        .build(&event_loop)?;

    let html = r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>MyDesk POC</title></head>
<body style="font-family: sans-serif; display:flex; flex-direction:column; gap:12px; padding:20px;">
<h2>MyDesk POC</h2>
<div id="log">Press the buttons below.</div>
<button id="btn1">Send hello</button>
<button id="btn2">Request app.version</button>
<button id="btn3">Request system.info</button>
<script>
  window.fromNative = (msg) => {
    const log = document.getElementById('log');
    log.innerText = 'Native replied: ' + JSON.stringify(msg);
  };
  document.getElementById('btn1').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ type: 'log', msg: 'hello from web' }));
  };
  document.getElementById('btn2').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ type: 'request', method: 'app.version', id: 1 }));
  };
  document.getElementById('btn3').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ type: 'request', method: 'system.info', id: 2 }));
  };
</script>
</body>
</html>"#;

    // Use Rc + RefCell to allow webview access inside closures
    let webview_rc: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
    let webview_clone = Rc::clone(&webview_rc);

    let webview = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |request: Request<String>| {
            let payload = request.body();
            println!("⟲ [native] Got IPC: {}", payload);

            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                if let Some(obj) = value.as_object() {
                    match obj.get("type").and_then(|v| v.as_str()) {
                        Some("log") => {
                            println!("log: {:?}", obj.get("msg"));
                        }
                        Some("request") => {
                            if let Some(method) = obj.get("method").and_then(|v| v.as_str()) {
                                let response_js = match method {
                                    "app.version" => r#"window.fromNative({ version: "0.1.0" });"#.to_string(),
                                    "system.info" => {
                                        // Example system info
                                        let info = format!(
    r#"window.fromNative({{ os: "{os}", arch: "{arch}" }});"#,
    os = std::env::consts::OS,
    arch = std::env::consts::ARCH
);

                                        info
                                    }
                                    _ => "".to_string(),
                                };

                                if let Some(webview_ref) = webview_clone.borrow().as_ref() {
                                    let _ = webview_ref.evaluate_script(&response_js);
                                }

                                println!("Handled request: {}", method);
                            }
                        }
                        _ => {}
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
