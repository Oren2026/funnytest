//! 階段轉換日誌（Phase Transition Log）
//!
//! 記錄 QuestionPhase 的變化：Exploration <-> Development <-> Mature。

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use crate::controller::gemma_controller::QuestionPhase;

/// 階段轉換記錄
#[derive(Debug, Clone)]
pub struct PhaseTransition {
    /// 發生時間
    pub timestamp: String,
    /// 轉換前階段
    pub from: QuestionPhase,
    /// 轉換後階段
    pub to: QuestionPhase,
    /// 觸發原因
    pub reason: String,
    /// 當時節點數
    pub node_count: usize,
}

/// Phase Transition Logger
///
/// 負責寫入階段轉換日誌。
/// 檔案位置：`workspace/logs/phase_transitions.md`
#[derive(Debug, Clone)]
pub struct PhaseTransitionLogger {
    /// 日誌檔案路徑
    log_path: PathBuf,
    /// 是否已初始化（已有標頭）
    initialized: bool,
}

impl PhaseTransitionLogger {
    /// 建立新的 PhaseTransitionLogger
    ///
    /// 如果檔案已存在，會附加到現有檔案。
    pub fn new(logs_dir: &PathBuf) -> std::io::Result<Self> {
        let filename = "phase_transitions.md";
        let log_path = logs_dir.join(filename);

        let initialized = log_path.exists();

        // 如果檔案不存在，建立並寫入標頭
        if !initialized {
            let mut file = File::create(&log_path)?;
            writeln!(file, "# 階段轉換記錄\n")?;
            writeln!(file, "| 時間 | 從 | 到 | 觸發原因 | 節點數 |")?;
            writeln!(file, "|------|----|----|---------|-------|")?;
        }

        Ok(PhaseTransitionLogger {
            log_path,
            initialized: true,
        })
    }

    /// 記錄一次階段轉換
    pub fn log_transition(&self, transition: &PhaseTransition) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(
            file,
            "| {} | {} | {} | {} | {} |",
            transition.timestamp,
            phase_name(&transition.from),
            phase_name(&transition.to),
            transition.reason,
            transition.node_count
        )?;

        Ok(())
    }

    /// 取得日誌檔案路徑
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

/// 取得階段名稱（中文）
fn phase_name(phase: &QuestionPhase) -> &'static str {
    phase.name()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_name() {
        assert_eq!(phase_name(&QuestionPhase::Exploration), "探索期");
        assert_eq!(phase_name(&QuestionPhase::Development), "發展期");
        assert_eq!(phase_name(&QuestionPhase::Mature), "成熟期");
    }

    #[test]
    fn test_phase_transition() {
        let t = PhaseTransition {
            timestamp: "2026-05-07 10:30:00".to_string(),
            from: QuestionPhase::Exploration,
            to: QuestionPhase::Development,
            reason: "節點數達到 3".to_string(),
            node_count: 3,
        };
        assert_eq!(t.from, QuestionPhase::Exploration);
        assert_eq!(t.to, QuestionPhase::Development);
    }
}
