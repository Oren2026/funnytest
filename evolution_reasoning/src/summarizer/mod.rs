//! NodeSummarizer - 單節點萃取系統
//!
//! 把 gemma4 回應中的結構化區塊（XML）解析為 NodeUpdate
//! 自動寫入節點的 key_findings / conclusion / relevant_topics

pub mod parser;
pub mod node_update;

pub use parser::NodeUpdateParser;
pub use node_update::NodeUpdate;