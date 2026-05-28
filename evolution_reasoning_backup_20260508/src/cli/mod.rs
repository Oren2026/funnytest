//! CLI 層（Command Line Interface）
//!
//! 提供互動式命令列介面。

pub mod repl;
pub mod gemma_repl;
pub mod visual;
pub mod server;

pub use repl::run_repl;
pub use gemma_repl::run_gemma_repl;
pub use visual::VisualPanel;
pub use server::{run_api_server, run_stdin_mode};
