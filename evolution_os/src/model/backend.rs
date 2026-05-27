//! OllamaBackend — 本地 Ollama 的 ModelDispatcher 實作
//!
//! 連接 `http://localhost:11434`，使用 `/api/generate` endpoint。

use super::{DispatchError, ModelDispatcher, ModelRequest, ModelResponse};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Ollama 後端實作
pub struct OllamaBackend {
    base_url: String,
    timeout_secs: u64,
}

impl OllamaBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(url: &str) -> Self {
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            timeout_secs: 60,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    fn send_request(&self, req: &ModelRequest) -> Result<String, DispatchError> {
        let host = self.base_url.replace("http://", "").replace("https://", "");
        let mut parts = host.split(':');
        let host = parts.next().unwrap_or("localhost");
        let port: u16 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(11434);

        // 嘗試 TCP 連線
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect_timeout(&addr.parse().map_err(|_| {
            DispatchError::ConnectionError(format!("invalid address: {}", addr))
        })?, Duration::from_secs(self.timeout_secs))
            .map_err(|e| DispatchError::ConnectionError(e.to_string()))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(self.timeout_secs)))
            .ok();
        stream
            .set_write_timeout(Some(Duration::from_secs(self.timeout_secs)))
            .ok();

        // 建立 Ollama API 請求
        let mut request_body = serde_json::json!({
            "model": req.model,
            "prompt": req.prompt,
            "stream": false,
            "options": {
                "temperature": req.temperature,
            }
        });

        if let Some(ref system) = req.system_prompt {
            request_body["system"] = serde_json::json!(system);
        }
        if let Some(tokens) = req.max_tokens {
            request_body["options"]["num_predict"] = serde_json::json!(tokens);
        }

        let body_str = serde_json::to_string(&request_body)
            .map_err(|e| DispatchError::BackendError(format!("failed to serialize: {}", e)))?;

        let request = format!(
            "POST /api/generate HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            host,
            port,
            body_str.len(),
            body_str
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| DispatchError::BackendError(format!("failed to send: {}", e)))?;

        let mut reading_body = false;
        let mut body = Vec::new();

        // 讀取 HTTP 回應（簡單的行式讀取）
        let mut buf = [0u8; 4096];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    for line in chunk.lines() {
                        if reading_body {
                            body.extend_from_slice(line.as_bytes());
                        } else if line.is_empty() {
                            reading_body = true;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Err(DispatchError::Timeout);
                }
                Err(e) => {
                    return Err(DispatchError::BackendError(format!("read error: {}", e)));
                }
            }
        }

        // 嘗試解析 Ollama 的 JSON 回應
        let json: serde_json::Value =
            serde_json::from_slice(&body).map_err(|e| {
                DispatchError::BackendError(format!("failed to parse response: {} (first 200 bytes: {:?})", e, String::from_utf8_lossy(&body[..body.len().min(200)])))
            })?;

        if let Some(err_msg) = json.get("error").and_then(|e| e.as_str()) {
            if err_msg.contains("model not found") || err_msg.contains("no such file") {
                return Err(DispatchError::ModelNotFound(req.model.clone()));
            }
            return Err(DispatchError::BackendError(err_msg.to_string()));
        }

        let content = json
            .get("response")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let model = json
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&req.model)
            .to_string();

        // tokens_used 可能在 total_duration 或其他欄位估算
        let tokens_used = json
            .get("eval_count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as u32;

        Ok(serde_json::to_string(&serde_json::json!({
            "content": content,
            "model": model,
            "tokens_used": tokens_used,
        })).unwrap())
    }
}

impl Default for OllamaBackend {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 60,
        }
    }
}

impl ModelDispatcher for OllamaBackend {
    fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
        let json_str = self.send_request(&req)?;
        let resp: ModelResponse =
            serde_json::from_str(&json_str).map_err(|e| {
                DispatchError::BackendError(format!("failed to parse response struct: {}", e))
            })?;
        Ok(resp)
    }

    fn available_models(&self) -> Vec<String> {
        // 嘗試讀取 /api/tags
        let host = self
            .base_url
            .replace("http://", "")
            .replace("https://", "");
        let mut parts = host.split(':');
        let host_part = parts.next().unwrap_or("localhost");
        let port: u16 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(11434);

        let addr = format!("{}:{}", host_part, port);
        let Ok(addr_parsed) = addr.parse() else {
            return vec![];
        };
        let Ok(mut stream) = TcpStream::connect_timeout(&addr_parsed, Duration::from_secs(2)) else {
            return vec![];
        };

        let request = format!(
            "GET /api/tags HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             Connection: close\r\n\
             \r\n",
            host_part, port
        );

        if stream.write_all(request.as_bytes()).is_err() {
            return vec![];
        }

        let mut body = Vec::new();
        let mut reading_body = false;
        let mut buf = [0u8; 1024];

        let timeout = std::time::Duration::from_secs(2);
        let _ = stream.set_read_timeout(Some(timeout));

        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    for line in chunk.lines() {
                        if reading_body {
                            body.extend_from_slice(line.as_bytes());
                        } else if line.is_empty() {
                            reading_body = true;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        let json: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        json.get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockOllama;

    impl ModelDispatcher for MockOllama {
        fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
            Ok(ModelResponse {
                content: format!("Echo: {}", req.prompt),
                model: req.model,
                tokens_used: req.prompt.len() as u32 / 4,
            })
        }

        fn available_models(&self) -> Vec<String> {
            vec!["llama3".into(), "gemma4:2b".into(), "mistral:7b".into()]
        }
    }

    #[test]
    fn test_mock_ollama_dispatch() {
        let backend = MockOllama;
        let req = ModelRequest::new("llama3", "Hello!");
        let resp = backend.dispatch(req).unwrap();
        assert!(resp.content.contains("Hello!"));
    }

    #[test]
    fn test_mock_available_models() {
        let backend = MockOllama;
        let models = backend.available_models();
        assert!(models.contains(&"llama3".into()));
    }

    #[test]
    fn test_ollama_backend_default_url() {
        let backend = OllamaBackend::new();
        assert_eq!(backend.available_models().len(), 0); // 無法連線時回空vec，不 error
    }
}