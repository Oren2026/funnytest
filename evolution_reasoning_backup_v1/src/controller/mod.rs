//! 控制器模組
//!
//! 整合 Ollama gemma4 模型與工具系統，讓 gemma4 可以操控 Evolution Reasoning Engine。

pub mod gemma_controller;

pub use gemma_controller::{ControllerMode, GemmaController, QuestionPhase};
