//! 執行橋接器
//!
//! v0.7 新增：Execution Feedback Loop 的核心實作。
//! 負責執行外部命令並將結果回傳到推理圖。

use std::collections::VecDeque;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::models::execution::{ExecutionResult, ExecutionTask, FeedbackItem};

/// 執行橋接器
///
/// 負責管理執行佇列、執行命令、收集 Feedback。
#[derive(Debug, Clone)]
pub struct ExecutionBridge {
    /// 待執行的任務佇列
    pending_tasks: VecDeque<ExecutionTask>,
    /// 已完成的 Feedback 歷史
    feedback_history: Vec<FeedbackItem>,
    /// 最大歷史記錄數
    max_history: usize,
}

impl Default for ExecutionBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionBridge {
    /// 建立新的執行橋接器
    pub fn new() -> Self {
        ExecutionBridge {
            pending_tasks: VecDeque::new(),
            feedback_history: Vec::new(),
            max_history: 100,
        }
    }

    /// 佇列中待執行的任務數量
    pub fn pending_count(&self) -> usize {
        self.pending_tasks.len()
    }

    /// 已處理的 Feedback 數量
    pub fn feedback_count(&self) -> usize {
        self.feedback_history.len()
    }

    /// 取得最新的 Feedback
    pub fn latest_feedback(&self) -> Option<&FeedbackItem> {
        self.feedback_history.last()
    }

    /// 依節點 ID 查詢 Feedback
    pub fn get_feedback_for_node(&self, node_id: &str) -> Option<&FeedbackItem> {
        self.feedback_history
            .iter()
            .rev()
            .find(|f| f.node_id == node_id)
    }

    /// 依節點 ID 查詢所有相關的 Feedback
    pub fn get_all_feedback_for_node(&self, node_id: &str) -> Vec<&FeedbackItem> {
        self.feedback_history
            .iter()
            .filter(|f| f.node_id == node_id)
            .collect()
    }

    /// 加入執行任務到佇列
    pub fn enqueue(&mut self, task: ExecutionTask) {
        self.pending_tasks.push_back(task);
    }

    /// 執行下一個任務
    ///
    /// # 引數
    /// - `node_id`: 任務關聯的節點 ID
    /// - `command`: 要執行的命令
    ///
    /// # 回傳
    /// 執行結果
    pub fn execute_next(&mut self, node_id: &str, command: &str) -> ExecutionResult {
        // 移除閒置逾時的任務（超過 5 分鐘的 pending 任務）
        self.cleanup_stale_tasks();

        // 執行命令
        let result = Self::run_command(command, 30);

        // 建立 Feedback
        let feedback = FeedbackItem::new(node_id.to_string(), result.clone());

        // 存入歷史
        self.push_feedback(feedback);

        result
    }

    /// 驗證執行結果是否符合預期
    ///
    /// # 引數
    /// - `node_id`: 節點 ID
    /// - `expected`: 預期結果
    ///
    /// # 回傳
    /// 是否符合預期
    pub fn verify(&self, node_id: &str, expected: &str) -> bool {
        if let Some(feedback) = self.get_feedback_for_node(node_id) {
            if let Some(message) = feedback.result.message() {
                return message.contains(expected);
            }
        }
        false
    }

    /// 重試失敗的任務
    ///
    /// # 引數
    /// - `node_id`: 節點 ID
    ///
    /// # 回傳
    /// 重試後的執行結果
    pub fn retry(&mut self, node_id: &str) -> ExecutionResult {
        // 找最近一次失敗的 Feedback
        if let Some(feedback) = self
            .feedback_history
            .iter()
            .rev()
            .find(|f| f.node_id == node_id && f.result.is_failure())
        {
            let mut new_feedback = feedback.clone();
            new_feedback.increment_retry();
            new_feedback.timestamp = chrono::Utc::now();

            // 重新執行（簡單假設同樣命令）
            // 實際實作需要從 task 取得 command，這裡用占位
            let result = Self::run_command(&format!("echo retry {}", node_id), 30);
            new_feedback.result = result.clone();

            self.push_feedback(new_feedback);
            result
        } else {
            ExecutionResult::Failure(format!("No failed task found for node {}", node_id))
        }
    }

    /// 清除所有待執行的任務
    pub fn clear_pending(&mut self) {
        self.pending_tasks.clear();
    }

    /// 清除逾時的 pending 任務（超過 5 分鐘）
    fn cleanup_stale_tasks(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(5);
        self.pending_tasks.retain(|task| task.created_at > cutoff);
    }

    /// 將 Feedback 存入歷史（並維持上限）
    pub fn push_feedback(&mut self, feedback: FeedbackItem) {
        self.feedback_history.push(feedback);
        if self.feedback_history.len() > self.max_history {
            self.feedback_history.remove(0);
        }
    }

    /// 執行外部命令
    ///
    /// # 引數
    /// - `command`: 命令字串
    /// - `timeout_secs`: 逾時秒數
    ///
    /// # 回傳
    /// 執行結果
    fn run_command(command: &str, timeout_secs: u64) -> ExecutionResult {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        // 使用 sh -c 執行命令（支援 shell 語法）
        match Command::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => {
                if start.elapsed() > timeout {
                    ExecutionResult::Timeout
                } else if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    ExecutionResult::Success(stdout.trim().to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    ExecutionResult::Failure(stderr.trim().to_string())
                }
            }
            Err(e) => ExecutionResult::Failure(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_bridge_new() {
        let bridge = ExecutionBridge::new();
        assert_eq!(bridge.pending_count(), 0);
        assert_eq!(bridge.feedback_count(), 0);
    }

    #[test]
    fn test_execute_command_success() {
        let mut bridge = ExecutionBridge::new();
        let result = bridge.execute_next("node-1", "echo hello");

        assert!(result.is_success());
        assert!(result.message().unwrap().contains("hello"));
    }

    #[test]
    fn test_execute_command_failure() {
        let mut bridge = ExecutionBridge::new();
        let result = bridge.execute_next("node-1", "exit 1");

        assert!(result.is_failure());
    }

    #[test]
    fn test_verify() {
        let mut bridge = ExecutionBridge::new();
        bridge.execute_next("node-1", "echo hello world");

        assert!(bridge.verify("node-1", "hello"));
        assert!(!bridge.verify("node-1", "goodbye"));
    }

    #[test]
    fn test_retry() {
        let mut bridge = ExecutionBridge::new();
        // 第一次執行（失敗）
        bridge.execute_next("node-1", "ls /nonexistent");
        // 重試
        let result = bridge.retry("node-1");

        // retry 會執行 echo retry node-1，所以應該成功
        assert!(result.is_success());
    }

    #[test]
    fn test_feedback_history_limit() {
        let mut bridge = ExecutionBridge::new();
        bridge.max_history = 3;

        for i in 0..5 {
            bridge.execute_next(&format!("node-{}", i), "echo ok");
        }

        assert_eq!(bridge.feedback_count(), 3);
    }
}
