//! Ollama API 客戶端
//!
//! 封裝與 Ollama API 的 HTTP 通訊。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ollama API 端點
const OLLAMA_BASE_URL: &str = "http://localhost:11434";
const CHAT_COMPLETIONS_URL: &str = "http://localhost:11434/api/chat";
const GENERATE_URL: &str = "http://localhost:11434/api/generate";

/// Ollama 客戶端
#[derive(Debug, Clone)]
pub struct OllamaClient {
    /// 模型名稱（預設：gemma4:e2b）
    model: String,
    /// HTTP 客戶端
    http_client: reqwest::Client,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("gemma4:e2b")
    }
}

impl OllamaClient {
    /// 建立新的 Ollama 客戶端
    ///
    /// # 引數
    /// - `model`: 模型名稱（預設 gemma4:e2b）
    pub fn new(model: impl Into<String>) -> Self {
        OllamaClient {
            model: model.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// 設定模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// 發送聊天完成請求
    ///
    /// # 引數
    /// - `messages`: 對話歷史
    /// - `tools`: 可用的工具定義（可選）
    ///
    /// # 範例
    /// ```ignore
    /// let client = OllamaClient::new("gemma4:e2b");
    /// let messages = vec![
    ///     Message::user("你好"),
    /// ];
    /// let response = client.chat(messages, None).await?;
    /// ```
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse, OllamaError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            tools,
            stream: Some(false),
        };

        let response = self
            .http_client
            .post(CHAT_COMPLETIONS_URL)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OllamaError::ApiError {
                status,
                message: body,
            });
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    /// 檢查 Ollama 服務是否可用
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/tags", OLLAMA_BASE_URL);
        self.http_client.get(&url).send().await.is_ok()
    }
}

// ============ 資料結構 ============

/// 訊息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Message {
    /// 角色：user, assistant, system
    pub role: String,
    /// 訊息內容
    pub content: String,
    /// 工具呼叫（可選）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    /// 建立使用者訊息
    pub fn user(content: impl Into<String>) -> Self {
        Message {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// 建立助理訊息
    pub fn assistant(content: impl Into<String>) -> Self {
        Message {
            role: "assistant".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }

    /// 建立系統訊息
    pub fn system(content: impl Into<String>) -> Self {
        Message {
            role: "system".to_string(),
            content: content.into(),
            tool_calls: None,
        }
    }
}

/// 聊天完成請求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// 聊天完成響應
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// 模型
    pub model: String,
    /// 創建時間戳
    pub created_at: String,
    /// 響應訊息
    pub message: Message,
    /// 是否完成
    #[serde(default)]
    pub done: bool,
    /// 總 context 长度
    #[serde(default)]
    pub total_duration: Option<u64>,
    /// 載入持續時間（纳秒）
    #[serde(default)]
    pub load_duration: Option<u64>,
    /// 提示計數
    #[serde(default)]
    pub prompt_eval_count: Option<i32>,
    /// 評估計數
    #[serde(default)]
    pub eval_count: Option<i32>,
    /// 工具呼叫（如果有）
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 工具定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// 工具類型（總是 "function"）
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函數定義
    pub function: FunctionDefinition,
}

impl Tool {
    /// 建立新的工具
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: ToolParameters) -> Self {
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        }
    }
}

/// 函數定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// 函數名稱
    pub name: String,
    /// 函數描述
    pub description: String,
    /// 參數定義
    pub parameters: ToolParameters,
}

/// 工具參數（JSON Schema 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    /// 類型（總是 "object"）
    #[serde(rename = "type")]
    pub param_type: String,
    /// 屬性定義
    pub properties: HashMap<String, ToolProperty>,
    /// 必要屬性
    pub required: Vec<String>,
}

impl ToolParameters {
    /// 建立新的參數定義
    pub fn new() -> Self {
        ToolParameters {
            param_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    /// 加入字串屬性
    pub fn add_string_prop(&mut self, name: &str, description: &str) {
        self.properties.insert(
            name.to_string(),
            ToolProperty {
                param_type: "string".to_string(),
                description: description.to_string(),
            },
        );
        if !self.required.contains(&name.to_string()) {
            self.required.push(name.to_string());
        }
    }

    /// 加入整數屬性
    pub fn add_integer_prop(&mut self, name: &str, description: &str) {
        self.properties.insert(
            name.to_string(),
            ToolProperty {
                param_type: "integer".to_string(),
                description: description.to_string(),
            },
        );
        if !self.required.contains(&name.to_string()) {
            self.required.push(name.to_string());
        }
    }
}

impl Default for ToolParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具屬性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
}

/// 工具呼叫
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 函數
    pub function: ToolCallFunction,
}

/// 工具呼叫函數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    /// 函數名稱
    pub name: String,
    /// 函數參數（JSON 字串）
    pub arguments: String,
}

/// 工具呼叫回應
#[derive(Debug, Serialize)]
pub struct ToolMessage {
    /// 角色（總是 "tool"）
    pub role: String,
    /// 內容（工具執行結果）
    pub content: String,
    /// 工具呼叫 ID
    pub tool_call_id: String,
}

impl ToolMessage {
    /// 建立工具回應訊息
    pub fn new(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        ToolMessage {
            role: "tool".to_string(),
            content: content.into(),
            tool_call_id: tool_call_id.into(),
        }
    }
}

// ============ 錯誤類型 ============

/// Ollama API 錯誤
#[derive(Debug)]
pub enum OllamaError {
    /// 請求失敗
    RequestError(reqwest::Error),
    /// API 錯誤
    ApiError {
        status: reqwest::StatusCode,
        message: String,
    },
    /// JSON 解析錯誤
    JsonError(serde_json::Error),
    /// 其他錯誤
    Other(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::RequestError(e) => write!(f, "請求錯誤: {}", e),
            OllamaError::ApiError { status, message } => {
                write!(f, "API 錯誤 ({}): {}", status, message)
            }
            OllamaError::JsonError(e) => write!(f, "JSON 解析錯誤: {}", e),
            OllamaError::Other(msg) => write!(f, "錯誤: {}", msg),
        }
    }
}

impl std::error::Error for OllamaError {}

impl From<reqwest::Error> for OllamaError {
    fn from(err: reqwest::Error) -> Self {
        OllamaError::RequestError(err)
    }
}

impl From<serde_json::Error> for OllamaError {
    fn from(err: serde_json::Error) -> Self {
        OllamaError::JsonError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "Hi there");
    }

    #[test]
    fn test_message_system() {
        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "You are helpful");
    }

    #[test]
    fn test_tool_new() {
        let mut params = ToolParameters::new();
        params.add_string_prop("node_id", "節點 ID");
        params.add_integer_prop("count", "數量");

        let tool = Tool::new("diverge", "發散生成子節點", params);

        assert_eq!(tool.function.name, "diverge");
        assert_eq!(tool.function.parameters.properties.len(), 2);
    }

    #[test]
    fn test_tool_parameters() {
        let mut params = ToolParameters::new();
        params.add_string_prop("name", "名稱");
        params.add_integer_prop("age", "年齡");

        assert_eq!(params.required.len(), 2);
        assert!(params.required.contains(&"name".to_string()));
        assert!(params.required.contains(&"age".to_string()));
    }

    #[test]
    fn test_tool_message() {
        let msg = ToolMessage::new("call_123", "結果內容");
        assert_eq!(msg.role, "tool");
        assert_eq!(msg.content, "結果內容");
        assert_eq!(msg.tool_call_id, "call_123");
    }

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        assert_eq!(client.model, "gemma4:e2b");
    }

    #[test]
    fn test_ollama_client_with_model() {
        let client = OllamaClient::new("llama2");
        assert_eq!(client.model, "llama2");

        let client2 = client.with_model("mixtral");
        assert_eq!(client2.model, "mixtral");
    }
}
