use serde_json::Value;
use std::env;
use chrono::prelude::*;
use std::fs;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{http::Request, WebViewBuilder};

type CommandFn = Box<dyn Fn(Request<String>, Rc<RefCell<Option<wry::WebView>>>)>;

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
<button id="btn3">Get OS</button>
<button id="btn4">Get Arch</button>
<button id="btn5">Get Time</button>
<button id="btn6">Read File</button>
<button id="btn7">Write File</button>
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
    window.ipc.postMessage(JSON.stringify({ type: 'request', method: 'get.os', id: 2 }));
  };
  document.getElementById('btn4').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ type: 'request', method: 'get.arch', id: 3 }));
  };
  document.getElementById('btn5').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ type: 'request', method: 'get.time', id: 4 }));
  };
  document.getElementById('btn6').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ 
      type: 'request', 
      method: 'fs.readFile', 
      id: 5,
      path: 'test.txt'
    }));
  };
  document.getElementById('btn7').onclick = () => {
    window.ipc.postMessage(JSON.stringify({ 
      type: 'request', 
      method: 'fs.writeFile', 
      id: 6,
      path: 'test.txt',
      content: 'Hello from JS!'
    }));
  };
</script>
</body>
</html>"#;

    // Rc<RefCell> to share WebView inside closure
    let webview_rc: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
    let webview_clone = Rc::clone(&webview_rc);

    // Command registry
    let mut commands: HashMap<String, CommandFn> = HashMap::new();

    commands.insert(
        "app.version".into(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let js = r#"window.fromNative({ version: "0.1.0" });"#;
                let _ = webview.evaluate_script(js);
            }
            println!("App version requested, replied with 0.1.0");
        }),
    );

    commands.insert(
        "get.os".into(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let os = env::consts::OS;
                let js = format!(r#"window.fromNative({{ os: "{}" }});"#, os);
                let _ = webview.evaluate_script(&js);
            }
            println!("OS requested");
        }),
    );

    commands.insert(
        "get.arch".into(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let arch = env::consts::ARCH;
                let js = format!(r#"window.fromNative({{ arch: "{}" }});"#, arch);
                let _ = webview.evaluate_script(&js);
            }
            println!("CPU architecture requested");
        }),
    );

    commands.insert(
        "get.time".into(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let now = Utc::now();
                let js = format!(r#"window.fromNative({{ time: "{}" }});"#, now.to_rfc3339());
                let _ = webview.evaluate_script(&js);
            }
            println!("Time requested");
        }),
    );

    commands.insert(
        "fs.readFile".into(),
        Box::new(|req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                if let Ok(value) = serde_json::from_str::<Value>(req.body()) {
                    if let Some(path) = value.get("path").and_then(|v| v.as_str()) {
                        let content = fs::read_to_string(path).unwrap_or_else(|_| "".into());
                        let js = format!(r#"window.fromNative({{ readFile: "{}" }});"#, content);
                        let _ = webview.evaluate_script(&js);
                    }
                }
            }
            println!("fs.readFile requested");
        }),
    );

    commands.insert(
        "fs.writeFile".into(),
        Box::new(|req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                if let Ok(value) = serde_json::from_str::<Value>(req.body()) {
                    if let (Some(path), Some(content)) = (
                        value.get("path").and_then(|v| v.as_str()),
                        value.get("content").and_then(|v| v.as_str()),
                    ) {
                        let _ = fs::write(path, content);
                        let js = r#"window.fromNative({ writeFile: "ok" });"#;
                        let _ = webview.evaluate_script(js);
                    }
                }
            }
            println!("fs.writeFile requested");
        }),
    );

    commands.insert(
        "log".into(),
        Box::new(|req, _| {
            if let Ok(value) = serde_json::from_str::<Value>(req.body()) {
                if let Some(msg) = value.get("msg") {
                    println!("log: {:?}", msg);
                }
            }
        }),
    );

    let webview = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |req: Request<String>| {
            if let Ok(value) = serde_json::from_str::<Value>(req.body()) {
                if let Some(obj) = value.as_object() {
                    if let Some(method) = obj.get("method").and_then(|v| v.as_str()) {
                        if let Some(cmd) = commands.get(method) {
                            cmd(req.clone(), Rc::clone(&webview_clone));
                        }
                    } else if let Some(typ) = obj.get("type").and_then(|v| v.as_str()) {
                        if let Some(cmd) = commands.get(typ) {
                            cmd(req.clone(), Rc::clone(&webview_clone));
                        }
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
