use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::error::MyDeskError;
use crate::ipc::{IpcRequest, IpcResponse};
use crate::utils;

pub type CommandFn = Box<dyn Fn(IpcRequest, Sender<String>) + 'static>;

pub fn get_commands() -> HashMap<String, CommandFn> {
    let mut commands: HashMap<String, CommandFn> = HashMap::new();

    // app.version
    commands.insert(
        "app.version".to_string(),
        Box::new(|req, tx| {
            let response = IpcResponse::success(req.id, json!({ "version": "0.1.0" }));
            let _ = tx.send(response.to_js_call());
        }),
    );

    // system.info
    commands.insert(
        "system.info".to_string(),
        Box::new(|req, tx| {
            let info = json!({
                "platform": utils::get_os(),
                "arch": utils::get_arch(),
            });
            let _ = tx.send(IpcResponse::success(req.id, info).to_js_call());
        }),
    );

    // system.time
    commands.insert(
        "system.time".to_string(),
        Box::new(|req, tx| {
            let _ = tx.send(
                IpcResponse::success(req.id, json!({ "time": utils::get_time() })).to_js_call(),
            );
        }),
    );

    // fs.read
    commands.insert(
        "fs.read".to_string(),
        Box::new(|req, tx| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let _ = tx.send(
                        IpcResponse::error(
                            req.id,
                            MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                        )
                        .to_js_call(),
                    );
                    return;
                }
            };

            std::thread::spawn(move || {
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(utils::read_file(&path));
                let response = match result {
                    Ok(content) => IpcResponse::success(req.id, json!({ "content": content })),
                    Err(err) => IpcResponse::error(req.id, err),
                };
                let _ = tx.send(response.to_js_call());
            });
        }),
    );

    // fs.write
    commands.insert(
        "fs.write".to_string(),
        Box::new(|req, tx| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let _ = tx.send(
                        IpcResponse::error(
                            req.id,
                            MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                        )
                        .to_js_call(),
                    );
                    return;
                }
            };

            let content = match req.params.get("content").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => {
                    let _ = tx.send(
                        IpcResponse::error(
                            req.id,
                            MyDeskError::InvalidRequest("Missing 'content' parameter".to_string()),
                        )
                        .to_js_call(),
                    );
                    return;
                }
            };

            std::thread::spawn(move || {
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(utils::write_file(&path, &content));
                let response = match result {
                    Ok(_) => IpcResponse::success(req.id, json!({ "success": true })),
                    Err(err) => IpcResponse::error(req.id, err),
                };
                let _ = tx.send(response.to_js_call());
            });
        }),
    );

    // fs.exists
    commands.insert(
        "fs.exists".to_string(),
        Box::new(|req, tx| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let _ = tx.send(
                        IpcResponse::error(
                            req.id,
                            MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                        )
                        .to_js_call(),
                    );
                    return;
                }
            };

            std::thread::spawn(move || {
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(utils::file_exists(&path));
                let response = match result {
                    Ok(exists) => IpcResponse::success(req.id, json!({ "exists": exists })),
                    Err(err) => IpcResponse::error(req.id, err),
                };
                let _ = tx.send(response.to_js_call());
            });
        }),
    );

    // fs.delete
    commands.insert(
        "fs.delete".to_string(),
        Box::new(|req, tx| {
            let path = match req.params.get("path").and_then(|v| v.as_str()) {
                Some(p) => p.to_string(),
                None => {
                    let _ = tx.send(
                        IpcResponse::error(
                            req.id,
                            MyDeskError::InvalidRequest("Missing 'path' parameter".to_string()),
                        )
                        .to_js_call(),
                    );
                    return;
                }
            };

            std::thread::spawn(move || {
                let result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(utils::delete_file(&path));
                let response = match result {
                    Ok(_) => IpcResponse::success(req.id, json!({ "success": true })),
                    Err(err) => IpcResponse::error(req.id, err),
                };
                let _ = tx.send(response.to_js_call());
            });
        }),
    );

    commands
}
