//! Ollama 整合模組
//!
//! 負責與 Ollama gemma4 模型通訊。
//!
//! 主要功能：
//! - 發送 chat completion 請求
//! - 解析 tool call 响应
//! - 管理對話歷史

pub mod client;

pub use client::OllamaClient;
