use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use serde_json::Value;
use wry::WebView;

pub type CommandFn =
    Box<dyn Fn(Value, Rc<RefCell<Option<WebView>>>) + 'static>;

pub fn get_commands() -> HashMap<String, CommandFn> {
    let mut commands: HashMap<String, CommandFn> = HashMap::new();

    // app.version
    commands.insert(
        "app.version".to_string(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let _ = webview.evaluate_script(
                    r#"window.fromNative({ version: "0.1.0" });"#
                );
            }
            println!("App version requested");
        }),
    );

    // get.os
    commands.insert(
        "get.os".to_string(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let os = crate::utils::get_os();
                let js = format!(r#"window.fromNative({{ os: "{}" }});"#, os);
                let _ = webview.evaluate_script(&js);
            }
            println!("OS requested");
        }),
    );

    // get.arch
    commands.insert(
        "get.arch".to_string(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let arch = crate::utils::get_arch();
                let js = format!(r#"window.fromNative({{ arch: "{}" }});"#, arch);
                let _ = webview.evaluate_script(&js);
            }
            println!("CPU architecture requested");
        }),
    );

    // get.time
    commands.insert(
        "get.time".to_string(),
        Box::new(|_req, webview_rc| {
            if let Some(webview) = webview_rc.borrow().as_ref() {
                let time = crate::utils::get_time();
                let js = format!(r#"window.fromNative({{ time: "{}" }});"#, time);
                let _ = webview.evaluate_script(&js);
            }
            println!("Time requested");
        }),
    );

    // fs.readFile
    commands.insert(
        "fs.readFile".to_string(),
        Box::new(|req, webview_rc| {
            if let Some(path) = req.get("path").and_then(|v| v.as_str()) {
                let content = crate::utils::read_file(path);

                if let Some(webview) = webview_rc.borrow().as_ref() {
                    let js = format!(
                        r#"window.fromNative({{ readFile: "{}" }});"#,
                        content
                    );
                    let _ = webview.evaluate_script(&js);
                }
            }
            println!("fs.readFile requested");
        }),
    );

    // fs.writeFile
    commands.insert(
        "fs.writeFile".to_string(),
        Box::new(|req, webview_rc| {
            if let Some(path) = req.get("path").and_then(|v| v.as_str()) {
                if let Some(content) = req.get("content").and_then(|v| v.as_str()) {
                    crate::utils::write_file(path, content);
                }
            }

            if let Some(webview) = webview_rc.borrow().as_ref() {
                let _ = webview.evaluate_script(
                    r#"window.fromNative({ writeFile: "ok" });"#
                );
            }

            println!("fs.writeFile requested");
        }),
    );

    // log
    commands.insert(
        "log".to_string(),
        Box::new(|req, _| {
            if let Some(msg) = req.get("msg") {
                println!("log: {:?}", msg);
            }
        }),
    );

    commands
}
