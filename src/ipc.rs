use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct IpcRequest {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::error::ErrorResponse>,
}

impl IpcResponse {
    pub fn success(id: String, data: Value) -> Self {
        Self {
            id,
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(id: String, error: crate::error::MyDeskError) -> Self {
        Self {
            id,
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }

    pub fn to_js_call(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        format!("window.__mydeskHandleResponse({})", json)
    }
}
