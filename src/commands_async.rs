use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wry::WebView;

use crate::error::{MyDeskError, Result};
use crate::ipc::{IpcRequest, IpcResponse};
use crate::utils;

pub type CommandFn = Box<dyn Fn(IpcRequest, Rc<RefCell<Option<WebView>>>) + 'static>;

pub fn get_commands() -> HashMap<String, CommandFn> {
    let mut commands: HashMap<String, CommandFn> = HashMap::new();

    // app.version
    commands.insert(
        "app.version".to_string(),
        Box::new(|req, webview_rc| {
            let response = IpcResponse::success(req.id, json!({ "version": "0.1.0" }));
            send_response(&webview_rc, response);
        }),
    );

    // system.info
    commands.insert(
        "system.info".to_string(),
        Box::new(|req, webview_rc| {
            let info = json!({
                "platform": utils::get_os(),
                "arch": utils::get_arch(),
                "version": "0.1.0"
            });
            let response = IpcResponse::success(req.id, info);
            send_response(&webview_rc, response);
        }),
    );

    // system.time
    commands.insert(
        "system.time".to_string(),
        Box::new(|req, webview_rc| {
            let time = utils::get_time();
            let response = IpcResponse::success(req.id, json!({ "time": time }));
            send_response(&webview_rc, response);
        }),
    );

    // fs.read
    commands.insert(
        "fs.read".to_string(),
        Box::new(|req, webview_rc| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let response = IpcResponse::error(
                        req.id,
                        MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                    );
                    send_response(&webview_rc, response);
                    return;
                }
            };

            let webview_clone = Rc::clone(&webview_rc);
            let request_id = req.id.clone();

            // Spawn async task
            tokio::spawn(async move {
                let result = utils::read_file(&path).await;
                let response = match result {
                    Ok(content) => IpcResponse::success(request_id, json!({ "content": content })),
                    Err(err) => IpcResponse::error(request_id, err),
                };
                send_response(&webview_clone, response);
            });
        }),
    );

    // fs.write
    commands.insert(
        "fs.write".to_string(),
        Box::new(|req, webview_rc| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let response = IpcResponse::error(
                        req.id,
                        MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                    );
                    send_response(&webview_rc, response);
                    return;
                }
            };

            let content = match req.params.get("content").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => {
                    let response = IpcResponse::error(
                        req.id,
                        MyDeskError::InvalidRequest("Missing 'content' parameter".to_string()),
                    );
                    send_response(&webview_rc, response);
                    return;
                }
            };

            let webview_clone = Rc::clone(&webview_rc);
            let request_id = req.id.clone();

            // Spawn async task
            tokio::spawn(async move {
                let result = utils::write_file(&path, &content).await;
                let response = match result {
                    Ok(_) => IpcResponse::success(request_id, json!({ "success": true })),
                    Err(err) => IpcResponse::error(request_id, err),
                };
                send_response(&webview_clone, response);
            });
        }),
    );

    // fs.exists
    commands.insert(
        "fs.exists".to_string(),
        Box::new(|req, webview_rc| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let response = IpcResponse::error(
                        req.id,
                        MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                    );
                    send_response(&webview_rc, response);
                    return;
                }
            };

            let webview_clone = Rc::clone(&webview_rc);
            let request_id = req.id.clone();

            tokio::spawn(async move {
                let result = utils::file_exists(&path).await;
                let response = match result {
                    Ok(exists) => IpcResponse::success(request_id, json!({ "exists": exists })),
                    Err(err) => IpcResponse::error(request_id, err),
                };
                send_response(&webview_clone, response);
            });
        }),
    );

    // fs.delete
    commands.insert(
        "fs.delete".to_string(),
        Box::new(|req, webview_rc| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let response = IpcResponse::error(
                        req.id,
                        MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                    );
                    send_response(&webview_rc, response);
                    return;
                }
            };

            let webview_clone = Rc::clone(&webview_rc);
            let request_id = req.id.clone();

            tokio::spawn(async move {
                let result = utils::delete_file(&path).await;
                let response = match result {
                    Ok(_) => IpcResponse::success(request_id, json!({ "success": true })),
                    Err(err) => IpcResponse::error(request_id, err),
                };
                send_response(&webview_clone, response);
            });
        }),
    );

    commands
}

fn send_response(webview_rc: &Rc<RefCell<Option<WebView>>>, response: IpcResponse) {
    if let Some(webview) = webview_rc.borrow().as_ref() {
        let js = response.to_js_call();
        let _ = webview.evaluate_script(&js);
    }
}
