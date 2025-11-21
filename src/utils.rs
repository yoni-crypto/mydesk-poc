use chrono::prelude::*;
use std::env;
use std::fs;

pub fn get_os() -> &'static str {
    env::consts::OS
}

pub fn get_arch() -> &'static str {
    env::consts::ARCH
}

pub fn get_time() -> String {
    Utc::now().to_rfc3339()
}

pub fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| "".into())
}

pub fn write_file(path: &str, content: &str) {
    let _ = fs::write(path, content);
}
