//! Evolution Reasoning Tool v0.5
//!
//! 純 Reasoning 工具框架，專注於發散/收斂推理。
//! v0.2 整合 Ollama gemma4 作為控制器。
//! v0.4 提問習慣系統（QuestionPhase）。
//! v0.5 長期記憶系統、視覺化面板、約束條件動態調整。
//!
//! # 模組
//!
//! - `models`: 核心資料結構（Node、Edge、Graph）
//! - `engine`: 發散/收斂引擎、複雜度預算系統、約束條件管理器
//! - `cli`: 命令列介面
//! - `ollama`: Ollama gemma4 API 客戶端
//! - `tools`: 工具介面系統（gemma4 可呼叫的工具）
//! - `workspace`: 狀態持久化（XML 格式）
//! - `controller`: gemma4 控制器整合

#![allow(dead_code, unused)]

pub mod cli;
pub mod controller;
pub mod engine;
pub mod export;
pub mod memory;
pub mod models;
pub mod observability;
pub mod ollama;
pub mod tools;
pub mod workspace;

// 重新匯出常用的型別
pub use models::{Edge, EdgeType, Graph, Node, NodeStatus};
pub use engine::{BacktrackManager, Checkpoint, CheckpointReason, ComplexityBudget, ConvergeEngine, Constraint, ConstraintManager, ConstraintSource, CorrectionHypothesis, DivergeEngine, FailurePattern, FailurePatternType, ThresholdGate};
pub use controller::gemma_controller::{ControllerMode, GemmaController, QuestionPhase};
pub use memory::{MemoryManager, UserProfile, HistoryEntry, ExploredTopic};
pub use tools::{ToolExecutor, ToolRegistry};
pub use export::{ExportFormat, export_graph, export_node, export_backtrack, export_hypotheses, export_memory};
