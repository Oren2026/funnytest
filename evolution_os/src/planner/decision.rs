//! 分工決策邏輯
//!
//! 判斷依據：
//! - reasoning_branches: 推理分支數（任務需要多少平行思考路徑）
//! - domain_diversity: 領域多樣性（任務涉及多少不同知識領域）
//! - context_complexity: 語境複雜度（輸入資訊的結構化程度）
//!
//! 決策規則：
//! - 單一節點：branches <= 2 且 diversity <= 1 且 context_complexity <= 0.6
//! - 多節點分工：branches > 2 或 diversity > 1 或 context_complexity > 0.6

use serde::{Deserialize, Serialize};

/// 複雜度評估結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// 推理分支數（0-5+）
    pub reasoning_branches: u8,
    /// 領域多樣性（0-3+）
    pub domain_diversity: u8,
    /// 語境複雜度（0.0-1.0）
    pub context_complexity: f32,
}

impl ComplexityMetrics {
    /// 從任務描述估算複雜度指標
    pub fn estimate_from_task(task: &str) -> Self {
        let task_lower = task.to_lowercase();

        let branches = Self::count_reasoning_branches(&task_lower);
        let diversity = Self::count_domain_diversity(&task_lower);
        let complexity = Self::estimate_context_complexity(task);

        Self {
            reasoning_branches: branches,
            domain_diversity: diversity,
            context_complexity: complexity,
        }
    }

    fn count_reasoning_branches(task: &str) -> u8 {
        let keywords = [
            "分析", "比較", "評估", "建構", "整合", "最佳化", "預測", "推薦", "分類", "排序",
            "多方", "各自", "同時", "平行", "分支",
        ];
        let mut count = 0u8;
        for kw in &keywords {
            if task.contains(kw) {
                count += 1;
            }
        }
        count.max(1).min(5)
    }

    fn count_domain_diversity(task: &str) -> u8 {
        let mut domains_found = 0u8;

        // (keywords, score)
        let domain_entries: [(&[&str], u8); 6] = [
            (&["資料庫", "sql", "db", "儲存", "資料", "mysql", "資料庫"], 1),
            (&["前端", "ui", "網頁", "html", "css", "react", "vue", "電子商務", "商城", "電商"], 1),
            (&["後端", "api", "server", "backend", "node", "java"], 1),
            (&["認證", "auth", "登入", "權限", "jwt", "oauth"], 1),
            (&["部署", "docker", "k8s", "ci", "cd"], 1),
            (&["安全", "加密", "資安", "xss", "sql"], 1),
        ];

        for (keywords, _) in domain_entries.iter() {
            for kw in keywords.iter() {
                if task.contains(kw) {
                    domains_found += 1;
                    break;
                }
            }
        }
        domains_found.max(1).min(4)
    }

    fn estimate_context_complexity(task: &str) -> f32 {
        let has_json = task.contains('{') || task.contains('}');
        let has_list = task.contains('\n') || task.contains(',');
        let tech_terms = [
            "module", "function", "class", "struct", "api", "endpoint", "schema", "model", "service",
        ];
        let has_tech_terms = tech_terms.iter().filter(|t| task.contains(*t)).count();

        let structural = if has_json || has_list { 0.2 } else { 0.0 };
        let technical = (has_tech_terms as f32 * 0.15).min(0.5);
        let len_score = (task.len() as f32 / 500.0).min(0.3);

        (structural + technical + len_score).min(1.0)
    }
}

/// 分工模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkMode {
    /// 單一節點獨立處理（CPU 單核模式）
    Solo,
    /// 多節點分工處理（GPU 發散模式）
    Fork,
}

impl WorkMode {
    /// 根據複雜度指標決定分工模式
    pub fn decide(metrics: &ComplexityMetrics) -> Self {
        let threshold_branches = 3;
        let threshold_diversity = 2;
        let threshold_complexity = 0.6;

        if metrics.reasoning_branches >= threshold_branches
            || metrics.domain_diversity >= threshold_diversity
            || metrics.context_complexity >= threshold_complexity
        {
            WorkMode::Fork
        } else {
            WorkMode::Solo
        }
    }
}

/// 分工決策結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDecision {
    /// 最終決策：Solo 或 Fork
    pub mode: WorkMode,
    /// 理由（為何選擇該模式）
    pub rationale: String,
    /// 預估節點數量
    pub estimated_nodes: u8,
    /// 主要領域標籤
    pub domain_tags: Vec<String>,
}

impl DispatchDecision {
    /// 根據複雜度指標和任務內容產生決策
    pub fn from_task(task: &str) -> Self {
        let metrics = ComplexityMetrics::estimate_from_task(task);
        let mode = WorkMode::decide(&metrics);
        let estimated_nodes = Self::estimate_nodes(&metrics, mode);
        let domain_tags = Self::extract_domain_tags(task);

        let rationale = format!(
            "branches={}, diversity={}, complexity={:.2} → {}",
            metrics.reasoning_branches,
            metrics.domain_diversity,
            metrics.context_complexity,
            match mode {
                WorkMode::Solo => "Solo（單一節點）",
                WorkMode::Fork => "Fork（多節點分工）",
            }
        );

        Self {
            mode,
            rationale,
            estimated_nodes,
            domain_tags,
        }
    }

    fn estimate_nodes(metrics: &ComplexityMetrics, mode: WorkMode) -> u8 {
        match mode {
            WorkMode::Solo => 1,
            WorkMode::Fork => (metrics.reasoning_branches as u8 + metrics.domain_diversity as u8 / 2)
                .max(2)
                .min(6),
        }
    }

    fn extract_domain_tags(task: &str) -> Vec<String> {
        // (keywords_slice, tag)
        let domain_map: [(&[&str], &str); 8] = [
            (&["資料庫", "sql", "db", "儲存", "資料", "mysql"], "database"),
            (&["前端", "ui", "html", "css", "react", "vue", "網站", "web", "頁面", "電子商務", "商城", "電商"], "frontend"),
            (&["後端", "api", "server", "backend", "node", "java"], "backend"),
            (&["認證", "auth", "登入", "權限", "jwt", "oauth"], "auth"),
            (&["部署", "docker", "ci", "cd", "k8s", "server"], "devops"),
            (&["安全", "加密", "資安", "xss", "sql"], "security"),
            (&["效能", "優化", "快取", "cache", "load"], "performance"),
            (&["測試", "test", "unit", "integration"], "testing"),
        ];

        let mut tags = Vec::new();
        let task_lower = task.to_lowercase();
        for (keywords, tag) in domain_map.iter() {
            for kw in keywords.iter() {
                if task_lower.contains(kw) {
                    if !tags.contains(&tag.to_string()) {
                        tags.push(tag.to_string());
                    }
                    break;
                }
            }
        }
        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_estimates() {
        let simple = "幫我建一個計數器網頁";
        let m = ComplexityMetrics::estimate_from_task(simple);
        assert_eq!(m.reasoning_branches, 1);
        assert!(m.context_complexity < 0.6);

        let complex = "幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能，並且要支援多方同時操作";
        let m = ComplexityMetrics::estimate_from_task(complex);
        assert!(m.reasoning_branches >= 2);
        assert!(m.domain_diversity >= 2);
    }

    #[test]
    fn test_work_mode_decision() {
        let simple = "幫我建一個計數器";
        let m = ComplexityMetrics::estimate_from_task(simple);
        assert_eq!(WorkMode::decide(&m), WorkMode::Solo);

        let complex = "幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能";
        let m = ComplexityMetrics::estimate_from_task(complex);
        assert_eq!(WorkMode::decide(&m), WorkMode::Fork);
    }

    #[test]
    fn test_dispatch_decision() {
        let d = DispatchDecision::from_task("幫我建一個庫存管理系統");
        assert!(matches!(d.mode, WorkMode::Solo | WorkMode::Fork));
        assert!(!d.rationale.is_empty());
    }

    #[test]
    fn test_domain_tags() {
        let d = DispatchDecision::from_task("幫我建一個有登入功能的電商網站");
        assert!(d.domain_tags.contains(&"frontend".to_string()));
        assert!(d.domain_tags.contains(&"auth".to_string()));
    }
}