use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebViewBuilder, WebView};
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
</script>
</body>
</html>"#;

    // Build webview
    let _webview: WebView = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |request: Request<String>| {
            let payload = request.body();
            println!("⟲ [native] Got IPC: {}", payload);

            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                if let Some(obj) = value.as_object() {
                    match obj.get("type").and_then(|v| v.as_str()) {
                        Some("request") => {
                            if obj.get("method") == Some(&Value::String("app.version".into())) {
                                println!("App version requested, would reply with 0.1.0");
                                // To actually reply, you need a WebView clone outside this closure
                            }
                        }
                        Some("log") => {
                            println!("log: {:?}", obj.get("msg"));
                        }
                        _ => {}
                    }
                }
            }
        })
        .build(&window)?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::CloseRequested = event {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
