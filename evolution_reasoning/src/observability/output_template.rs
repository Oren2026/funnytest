//! Session 產出模板（Output Template）
//!
//! v0.7 新增：在 Session 結束時自動生成結構化的產出報告。
//!
//! # 輸出位置
//!
//! ```ignore
//! workspace/output/
//! └── discussion_{timestamp}.md
//! ```

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

/// Session 產出資料
#[derive(Debug, Clone)]
pub struct OutputData {
    /// 討論主題
    pub topic: String,
    /// 日期時間
    pub timestamp: String,
    /// 總節點數
    pub total_nodes: usize,
    /// 總邊數
    pub total_edges: usize,
    /// 總複雜度
    pub total_complexity: f64,
    /// 最終階段
    pub final_phase: String,
    /// 核心發現摘要
    pub core_findings: String,
    /// 決策樹內容（Markdown 格式）
    pub decision_tree: String,
    /// 後續行動建議
    pub action_items: Vec<String>,
}

impl OutputData {
    /// 建立新的 OutputData
    pub fn new(topic: &str) -> Self {
        OutputData {
            topic: topic.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_nodes: 0,
            total_edges: 0,
            total_complexity: 0.0,
            final_phase: String::new(),
            core_findings: String::new(),
            decision_tree: String::new(),
            action_items: Vec::new(),
        }
    }

    /// 從 SessionStats 和決策樹產生 OutputData
    pub fn from_stats_and_tree(
        topic: &str,
        node_count: usize,
        edge_count: usize,
        complexity: f64,
        phase: &str,
        decision_tree_markdown: &str,
    ) -> Self {
        OutputData {
            topic: topic.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_nodes: node_count,
            total_edges: edge_count,
            total_complexity: complexity,
            final_phase: phase.to_string(),
            core_findings: Self::generate_core_findings(node_count, complexity, phase),
            decision_tree: decision_tree_markdown.to_string(),
            action_items: Self::generate_action_items(node_count, phase),
        }
    }

    /// 產生核心發現摘要
    fn generate_core_findings(node_count: usize, complexity: f64, phase: &str) -> String {
        let mut findings = Vec::new();

        findings.push(format!("總共建立了 {} 個思考節點", node_count));
        findings.push(format!("總複雜度為 {:.2}", complexity));

        let phase_desc = match phase {
            "探索期" => "目前處於探索階段，主要聚焦於了解問題的本質",
            "發展期" => "目前處於發展階段，已開始形成具體的方向和策略",
            "成熟期" => "目前處於成熟階段，已形成完整的決策框架",
            _ => "階段未知",
        };
        findings.push(phase_desc.to_string());

        findings.join("\n")
    }

    /// 產生後續行動建議
    fn generate_action_items(node_count: usize, phase: &str) -> Vec<String> {
        let mut items = Vec::new();

        match phase {
            "探索期" => {
                items.push("繼續提問以深化對問題的理解".to_string());
                items.push("識別關鍵的約束條件和限制".to_string());
                items.push("列舉可能的方向或選項".to_string());
            }
            "發展期" => {
                items.push("對已識別的方向進行深入分析".to_string());
                items.push("評估每個方向的可行性".to_string());
                items.push("開始建立候選方案".to_string());
            }
            "成熟期" => {
                items.push("收斂到最有希望的少數方案".to_string());
                items.push("制定具體的行動計劃".to_string());
                items.push("識別下一步的具體行動".to_string());
            }
            _ => {
                items.push("繼續探索和發展".to_string());
            }
        }

        // 根據節點數添加建議
        if node_count > 10 {
            items.push("考慮進行收斂操作以簡化圖結構".to_string());
        }
        if node_count < 3 {
            items.push("增加節點數量以獲得更完整的視角".to_string());
        }

        items
    }

    /// 轉換為 Markdown 格式
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# 討論產出：{}\n\n", self.topic));
        output.push_str(&format!("**日期：** {}\n", self.timestamp));
        output.push_str(&format!("**總節點數：** {}\n", self.total_nodes));
        output.push_str(&format!("**總邊數：** {}\n", self.total_edges));
        output.push_str(&format!("**總複雜度：** {:.2}\n", self.total_complexity));
        output.push_str(&format!("**最終階段：** {}\n", self.final_phase));
        output.push('\n');
        output.push_str("---\n\n");

        output.push_str("## 核心發現\n\n");
        output.push_str(&self.core_findings);
        output.push('\n');
        output.push('\n');

        output.push_str("## 決策樹\n\n");
        output.push_str(&self.decision_tree);
        output.push('\n');

        output.push_str("## 後續行動建議\n\n");
        for (i, item) in self.action_items.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, item));
        }
        output.push('\n');

        output.push_str("---\n\n");
        output.push_str("*此報告由 Evolution Reasoning Tool v0.7 自動產生*\n");

        output
    }
}

/// Session Output Logger
///
/// 負責將 Session 產出寫入檔案。
#[derive(Debug, Clone)]
pub struct OutputLogger {
    /// 輸出目錄路徑
    output_dir: PathBuf,
}

impl Default for OutputLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputLogger {
    /// 建立新的 OutputLogger
    pub fn new() -> Self {
        let root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".evolution_reasoning")
            .join("workspace")
            .join("output");

        OutputLogger {
            output_dir: root,
        }
    }

    /// 確保輸出目錄存在
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.output_dir)
    }

    /// 寫入產出檔案
    pub fn write_output(&self, data: &OutputData) -> std::io::Result<PathBuf> {
        self.ensure_dir()?;

        let filename = format!("discussion_{}.md",
            Local::now().format("%Y%m%d_%H%M%S"));
        let path = self.output_dir.join(&filename);

        let mut file = File::create(&path)?;
        file.write_all(data.to_markdown().as_bytes())?;

        Ok(path)
    }

    /// 取得輸出目錄路徑
    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_data_new() {
        let data = OutputData::new("測試主題");
        assert_eq!(data.topic, "測試主題");
        assert!(data.total_nodes == 0);
        assert!(data.core_findings.is_empty());
    }

    #[test]
    fn test_output_data_from_stats() {
        let tree = "## 決策樹\n\n- 方向 A\n- 方向 B\n";
        let data = OutputData::from_stats_and_tree(
            "睡眠品質",
            10,
            15,
            42.5,
            "發展期",
            tree,
        );

        assert_eq!(data.topic, "睡眠品質");
        assert_eq!(data.total_nodes, 10);
        assert_eq!(data.total_edges, 15);
        assert!((data.total_complexity - 42.5).abs() < 0.001);
        assert_eq!(data.final_phase, "發展期");
    }

    #[test]
    fn test_output_data_to_markdown() {
        let mut data = OutputData::new("測試");
        data.action_items.push("行動 1".to_string());
        data.action_items.push("行動 2".to_string());

        let markdown = data.to_markdown();

        assert!(markdown.contains("討論產出：測試"));
        assert!(markdown.contains("行動 1"));
        assert!(markdown.contains("行動 2"));
        assert!(markdown.contains("Evolution Reasoning Tool v0.7"));
    }

    #[test]
    fn test_output_logger() {
        let logger = OutputLogger::new();
        assert!(logger.output_dir().ends_with("output"));

        let data = OutputData::new("測試");
        let result = logger.write_output(&data);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());

        // cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_generate_action_items_exploration() {
        let items = OutputData::generate_action_items(2, "探索期");
        assert!(!items.is_empty());
        assert!(items[0].contains("提問"));
    }

    #[test]
    fn test_generate_action_items_mature() {
        let items = OutputData::generate_action_items(15, "成熟期");
        assert!(!items.is_empty());
        assert!(items[0].contains("收斂") || items[0].contains("行動"));
    }
}