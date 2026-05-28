//! 引擎層（Engine）
//!
//! 包含發散引擎、收斂引擎、複雜度預算系統。

pub mod backtrack;
pub mod budget;
pub mod converge;
pub mod constraint;
pub mod decision_tree;
pub mod diverge;

pub use backtrack::{BacktrackManager, Checkpoint, CheckpointReason, CorrectionHypothesis, FailurePattern, FailurePatternType};
pub use budget::{ComplexityBudget, ThresholdGate};
pub use converge::ConvergeEngine;
pub use constraint::{Constraint, ConstraintManager, ConstraintSource};
pub use decision_tree::{DecisionNode, DecisionTree, generate_full_report};
pub use diverge::DivergeEngine;
