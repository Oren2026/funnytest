//! 執行與回饋模型
//!
//! v0.7 新增：Execution Feedback Loop 的核心資料結構。

use chrono::{DateTime, Utc};

/// 執行結果
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// 執行成功，回傳結果
    Success(String),
    /// 執行失敗，回傳錯誤
    Failure(String),
    /// 執行逾時
    Timeout,
    /// 執行中
    Pending,
}

impl ExecutionResult {
    /// 檢查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success(_))
    }

    /// 檢查是否失敗
    pub fn is_failure(&self) -> bool {
        matches!(self, ExecutionResult::Failure(_))
    }

    /// 檢查是否進行中
    pub fn is_pending(&self) -> bool {
        matches!(self, ExecutionResult::Pending)
    }

    /// 取得訊息（如果有）
    pub fn message(&self) -> Option<&str> {
        match self {
            ExecutionResult::Success(msg) => Some(msg),
            ExecutionResult::Failure(msg) => Some(msg),
            _ => None,
        }
    }
}

impl std::fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Success(msg) => write!(f, "Success: {}", msg),
            ExecutionResult::Failure(msg) => write!(f, "Failure: {}", msg),
            ExecutionResult::Timeout => write!(f, "Timeout"),
            ExecutionResult::Pending => write!(f, "Pending"),
        }
    }
}

/// 執行回饋項目
#[derive(Debug, Clone)]
pub struct FeedbackItem {
    /// 關聯的節點 ID
    pub node_id: String,
    /// 執行結果
    pub result: ExecutionResult,
    /// 時間戳
    pub timestamp: DateTime<Utc>,
    /// 重試次數
    pub retry_count: usize,
}

impl FeedbackItem {
    /// 建立新的回饋項目
    pub fn new(node_id: String, result: ExecutionResult) -> Self {
        FeedbackItem {
            node_id,
            result,
            timestamp: Utc::now(),
            retry_count: 0,
        }
    }

    /// 建立成功的回饋
    pub fn success(node_id: String, message: String) -> Self {
        Self::new(node_id, ExecutionResult::Success(message))
    }

    /// 建立失敗的回饋
    pub fn failure(node_id: String, message: String) -> Self {
        Self::new(node_id, ExecutionResult::Failure(message))
    }

    /// 建立逾時的回饋
    pub fn timeout(node_id: String) -> Self {
        Self::new(node_id, ExecutionResult::Timeout)
    }

    /// 建立執行中的回饋
    pub fn pending(node_id: String) -> Self {
        Self::new(node_id, ExecutionResult::Pending)
    }

    /// 增加重試次數
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// 執行任務描述
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    /// 任務 ID
    pub id: String,
    /// 關聯的節點 ID
    pub node_id: String,
    /// 執行的命令
    pub command: String,
    /// 預期結果（可選）
    pub expected_result: Option<String>,
    /// 逾時時間（秒）
    pub timeout_secs: u64,
    /// 建立時間
    pub created_at: DateTime<Utc>,
}

impl ExecutionTask {
    /// 建立新的執行任務
    pub fn new(node_id: String, command: String) -> Self {
        ExecutionTask {
            id: uuid::Uuid::new_v4().to_string(),
            node_id,
            command,
            expected_result: None,
            timeout_secs: 30,
            created_at: Utc::now(),
        }
    }

    /// 設定逾時時間
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 設定預期結果
    pub fn with_expected_result(mut self, result: String) -> Self {
        self.expected_result = Some(result);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_is_success() {
        let success = ExecutionResult::Success("done".to_string());
        let failure = ExecutionResult::Failure("error".to_string());
        let timeout = ExecutionResult::Timeout;

        assert!(success.is_success());
        assert!(!failure.is_success());
        assert!(!timeout.is_success());
    }

    #[test]
    fn test_feedback_item_new() {
        let feedback = FeedbackItem::success("node-1".to_string(), "ok".to_string());
        assert_eq!(feedback.node_id, "node-1");
        assert!(feedback.result.is_success());
        assert_eq!(feedback.retry_count, 0);
    }

    #[test]
    fn test_execution_task() {
        let task = ExecutionTask::new("node-1".to_string(), "ls -la".to_string())
            .with_timeout(60)
            .with_expected_result("files".to_string());

        assert_eq!(task.node_id, "node-1");
        assert_eq!(task.command, "ls -la");
        assert_eq!(task.timeout_secs, 60);
        assert!(task.expected_result.is_some());
    }
}
