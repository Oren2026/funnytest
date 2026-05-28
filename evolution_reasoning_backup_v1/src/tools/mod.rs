//! 工具系統模組
//!
//! 定義 Evolution Reasoning Tool 的工具介面，讓 gemma4 可以呼叫 Rust Engine 的功能。
//!
//! 可用工具：
//! - `diverge(node_id, content, count)` - 發散生成子節點
//! - `converge()` - 觸發收斂
//! - `save(name)` - 儲存狀態到 workspace
//! - `load(name)` - 從 workspace 載入狀態
//! - `output(format)` - 產出為 XML/MD 檔案
//! - `status()` - 回傳狀態摘要

pub mod executor;
pub mod registry;

pub use executor::ToolExecutor;
pub use registry::ToolRegistry;
