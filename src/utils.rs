use chrono::prelude::*;
use std::path::PathBuf;
use tokio::fs;

use crate::error::{MyDeskError, Result};

pub fn get_os() -> &'static str {
    std::env::consts::OS
}

pub fn get_arch() -> &'static str {
    std::env::consts::ARCH
}

pub fn get_time() -> String {
    Utc::now().to_rfc3339()
}

// Validate and sanitize file paths
pub fn validate_path(path: &str) -> Result<PathBuf> {
    // Remove any path traversal attempts
    if path.contains("..") {
        return Err(MyDeskError::InvalidPath(
            "Path traversal not allowed".to_string(),
        ));
    }

    // Expand home directory
    let expanded = if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            home.join(&path[2..])
        } else {
            return Err(MyDeskError::InvalidPath("Cannot resolve home directory".to_string()));
        }
    } else {
        PathBuf::from(path)
    };

    Ok(expanded)
}

pub async fn read_file(path: &str) -> Result<String> {
    let path = validate_path(path)?;
    let content = fs::read_to_string(&path).await?;
    Ok(content)
}

pub async fn write_file(path: &str, content: &str) -> Result<()> {
    let path = validate_path(path)?;
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    fs::write(&path, content).await?;
    Ok(())
}

pub async fn file_exists(path: &str) -> Result<bool> {
    let path = validate_path(path)?;
    Ok(path.exists())
}

pub async fn delete_file(path: &str) -> Result<()> {
    let path = validate_path(path)?;
    fs::remove_file(&path).await?;
    Ok(())
}
