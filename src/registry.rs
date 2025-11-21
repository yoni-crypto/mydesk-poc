use std::{rc::Rc, cell::RefCell, env, fs};
use wry::WebView;
use serde_json::Value;
use std::collections::HashMap;

pub type CommandFn = Box<dyn Fn(Value, Rc<RefCell<Option<WebView>>>)>;


pub fn build_registry() -> HashMap<String, CommandFn> {
    let mut registry: HashMap<String, CommandFn> = HashMap::new();

    // App version
    registry.insert(
        "app.version".to_string(), // explicitly String
        Box::new(|_req: Value, webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let js = r#"window.fromNative({ version: "0.1.0" });"#;
                let _ = webview.evaluate_script(js);
            }
            println!("App version requested");
        }),
    );

    // Get OS
    registry.insert(
        "get.os".to_string(),
        Box::new(|_req: Value, webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let os = env::consts::OS;
                let js = format!(r#"window.fromNative({{ os: "{}" }});"#, os);
                let _ = webview.evaluate_script(&js);
            }
            println!("OS requested");
        }),
    );

    // Get CPU architecture
    registry.insert(
        "get.arch".to_string(),
        Box::new(|_req: Value, webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let arch = env::consts::ARCH;
                let js = format!(r#"window.fromNative({{ arch: "{}" }});"#, arch);
                let _ = webview.evaluate_script(&js);
            }
            println!("CPU architecture requested");
        }),
    );

    // Read file
    registry.insert(
        "fs.readFile".to_string(),
        Box::new(|req: Value, webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                if let Some(path) = req.get("path").and_then(|v| v.as_str()) {
                    let content = fs::read_to_string(path).unwrap_or_default();
                    let js = format!(r#"window.fromNative({{ readFile: "{}" }});"#, content);
                    let _ = webview.evaluate_script(&js);
                }
            }
            println!("fs.readFile requested");
        }),
    );

    // Write file
    registry.insert(
        "fs.writeFile".to_string(),
        Box::new(|req: Value, webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                if let Some(path) = req.get("path").and_then(|v| v.as_str()) {
                    if let Some(content) = req.get("content").and_then(|v| v.as_str()) {
                        let _ = fs::write(path, content);
                        let js = r#"window.fromNative({ writeFile: "ok" });"#;
                        let _ = webview.evaluate_script(js);
                    }
                }
            }
            println!("fs.writeFile requested");
        }),
    );

    // Log messages
    registry.insert(
        "log".to_string(),
        Box::new(|req: Value, _webview_rc: Rc<RefCell<Option<WebView>>>| {
            if let Some(msg) = req.get("msg") {
                println!("log: {:?}", msg);
            }
        }),
    );

    registry
}
