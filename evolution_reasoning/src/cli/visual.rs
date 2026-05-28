//! 視覺化面板（Visual Panel）
//!
//! v0.5 新增：CLI 彩色輸出，用 ANSI escape code 顯示節點圖。
//!
//! # 顏色定義
//!
//! - 鎖定節點（Locked）：藍色
//! - 高信心度 (> 0.8)：綠色
//! - 中信心度 (0.5-0.8)：黃色
//! - 低信心度 (< 0.5)：紅色
//! - 探索期（Exploration）：白色
//! - 發展期（Development）：青色
//! - 成熟期（Mature）：紫色
//!
//! # 輸出格式
//!
//! ```ignore
//! Root: 人生規劃
//! ├── [探索期] 價值觀確立 (conf: 0.9, locked)
//! │   └── 什麼事情對你最重要？ (conf: 0.8)
//! ├── [探索期] 健康管理 (conf: 0.7)
//! └── [發展期] 職業發展 (conf: 0.6)
//! ```

use crate::controller::gemma_controller::QuestionPhase;
use crate::models::{Graph, Node, NodeStatus, Topic, TopicPhase};

/// ANSI 顏色定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// 預設/白色
    White,
    /// 黑色
    Black,
    /// 紅色
    Red,
    /// 綠色
    Green,
    /// 黃色
    Yellow,
    /// 藍色
    Blue,
    /// 紫色（洋紅）
    Magenta,
    /// 青色
    Cyan,
    /// 深灰色
    DarkGray,
    /// 淺灰色
    LightGray,
}

/// ANSI 樣式
#[derive(Debug, Clone, Copy)]
pub enum Style {
    /// 一般
    Normal,
    /// 粗體
    Bold,
    /// 暗色
    Dim,
}

/// ANSI escape code 前綴
const ANSI_PREFIX: &str = "\x1b[";

/// ANSI 重置
const ANSI_RESET: &str = "\x1b[0m";

impl Color {
    /// 取得前景色 escape code
    fn fg_code(&self) -> &'static str {
        match self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::DarkGray => "90",
            Color::LightGray => "97",
        }
    }

    /// 取得背景色 escape code
    fn bg_code(&self) -> &'static str {
        match self {
            Color::Black => "40",
            Color::Red => "41",
            Color::Green => "42",
            Color::Yellow => "43",
            Color::Blue => "44",
            Color::Magenta => "45",
            Color::Cyan => "46",
            Color::White => "47",
            Color::DarkGray => "100",
            Color::LightGray => "107",
        }
    }
}

/// 格式化 ANSI 文字
fn ansi_str(color: Color, style: Style, text: &str) -> String {
    format!(
        "{}{};{}{}m{}{}",
        ANSI_PREFIX,
        style as u8 + 1,
        color.fg_code(),
        ANSI_PREFIX.trim_end_matches('\x1b'),
        text,
        ANSI_RESET
    )
}

/// 取得節點的顏色（根據狀態和信心度）
fn get_node_color(node: &Node) -> Color {
    // Locked 節點永遠是藍色
    if node.status == NodeStatus::Locked {
        return Color::Blue;
    }

    // 根據信心度決定顏色
    if node.confidence > 0.8 {
        Color::Green
    } else if node.confidence >= 0.5 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// 取得階段的顏色
fn get_phase_color(phase: QuestionPhase) -> Color {
    match phase {
        QuestionPhase::Exploration => Color::White,
        QuestionPhase::Development => Color::Cyan,
        QuestionPhase::Mature => Color::Magenta,
    }
}

/// 取得階段名稱（中文）
fn get_phase_name(phase: QuestionPhase) -> &'static str {
    phase.name()
}

/// 視覺化面板
///
/// 負責用 ANSI 彩色輸出顯示推理圖結構。
#[derive(Debug, Clone)]
pub struct VisualPanel {
    /// 是否啟用顏色輸出
    colored: bool,
}

impl Default for VisualPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualPanel {
    /// 建立新的視覺化面板
    pub fn new() -> Self {
        VisualPanel { colored: true }
    }

    /// 建立停用顏色的視覺化面板
    #[allow(dead_code)]
    pub fn no_color() -> Self {
        VisualPanel { colored: false }
    }

    /// 啟用/停用顏色
    pub fn set_color(&mut self, enabled: bool) {
        self.colored = enabled;
    }

    /// 格式化文字（套用或不套用 ANSI）
    pub fn format(&self, color: Color, style: Style, text: &str) -> String {
        if self.colored {
            ansi_str(color, style, text)
        } else {
            text.to_string()
        }
    }

    /// 顯示完整的推理圖
    pub fn display_graph(&self, graph: &Graph, phase: QuestionPhase) {
        let roots = graph.get_root_nodes();

        if roots.is_empty() {
            println!("{}", self.format(Color::Yellow, Style::Normal, "（圖是空的）"));
            return;
        }

        println!();
        self.display_header(phase, graph.node_count());
        println!();

        for root in roots {
            self.display_node_tree(graph, root, 0, true, phase);
        }

        println!();
        self.display_legend();
    }

    /// 顯示多主題圖（v0.7 新增）
    ///
    /// 當圖中有多個主題時，分別顯示每個主題的節點樹。
    pub fn display_multi_topic_graph(&self, graph: &Graph) {
        let topics = graph.get_topics();

        if topics.is_empty() {
            println!("{}", self.format(Color::Yellow, Style::Normal, "（尚無主題）\n"));
            return;
        }

        println!();
        println!("{}", self.format(Color::White, Style::Bold, &"═".repeat(50)));
        println!(
            "  {} 多主題視圖 | 主題數: {}",
            self.format(Color::Cyan, Style::Bold, "[階段]"),
            topics.len()
        );
        println!("{}", self.format(Color::White, Style::Bold, &"═".repeat(50)));
        println!();

        for topic in topics {
            let node_count = graph.count_topic_nodes(&topic.id);
            let phase = graph.get_topic_phase(&topic.id);
            let is_current = graph.current_topic_id.as_ref() == Some(&topic.id);

            // 主題標題
            let phase_color = match phase {
                TopicPhase::Exploration => Color::White,
                TopicPhase::Development => Color::Cyan,
                TopicPhase::Mature => Color::Magenta,
            };

            let current_marker = if is_current {
                format!("{}", self.format(Color::Green, Style::Bold, " ▶ "))
            } else {
                "   ".to_string()
            };

            println!("{}", self.format(Color::White, Style::Bold, &"─".repeat(40)));
            println!(
                "{} {} {} {}",
                current_marker,
                self.format(Color::Yellow, Style::Bold, &format!("主題：{}", topic.title)),
                self.format(phase_color, Style::Dim, &format!("({})", phase.name())),
                self.format(Color::DarkGray, Style::Normal, &format!("• {} 節點", node_count)),
            );

            // 取得主題根節點
            if let Some(root_node) = graph.get_node(&topic.root_node_id) {
                // 遞迴顯示該主題的節點樹
                // 將 TopicPhase 轉換為 QuestionPhase（兩者階段劃分相同）
                let question_phase = match phase {
                    TopicPhase::Exploration => QuestionPhase::Exploration,
                    TopicPhase::Development => QuestionPhase::Development,
                    TopicPhase::Mature => QuestionPhase::Mature,
                };
                self.display_node_tree(graph, root_node, 0, true, question_phase);
            }

            println!();
        }

        println!("{}", self.format(Color::White, Style::Bold, &"═".repeat(50)));
        self.display_legend();
    }

    /// 顯示圖表標頭
    /// 顯示圖表標頭
    fn display_header(&self, phase: QuestionPhase, node_count: usize) {
        let phase_color = get_phase_color(phase);
        let phase_name = get_phase_name(phase);

        println!("{}", self.format(Color::White, Style::Bold, &"═".repeat(50)));
        println!(
            "  {} {} | 節點數: {}",
            self.format(phase_color, Style::Bold, "[階段]"),
            self.format(Color::White, Style::Normal, phase_name),
            node_count
        );
        println!("{}", self.format(Color::White, Style::Bold, &"═".repeat(50)));
    }

    /// 遞迴顯示節點樹
    fn display_node_tree(&self, graph: &Graph, node: &Node, depth: usize, is_last: bool, phase: QuestionPhase) {
        // 計算縮排
        let prefix = if depth == 0 {
            String::new()
        } else {
            let indent = if is_last { "  " } else { "│ " };
            let branch = if is_last { "└─" } else { "├─" };
            indent.repeat(depth - 1) + branch
        };

        // 格式化節點內容
        let node_str = self.format_node(node, phase);
        println!("{}{}", prefix, node_str);

        // 遞迴顯示子節點
        let children = graph.get_children(&node.id);
        let child_count = children.len();
        for (i, child) in children.iter().enumerate() {
            let is_last_child = i == child_count - 1;
            self.display_node_tree(graph, child, depth + 1, is_last_child, phase);
        }
    }

    /// 格式化單一節點顯示
    fn format_node(&self, node: &Node, phase: QuestionPhase) -> String {
        let color = get_node_color(node);
        let status_marker = match node.status {
            NodeStatus::Draft => self.format(Color::DarkGray, Style::Normal, "[草]"),
            NodeStatus::Active => self.format(Color::White, Style::Normal, "[活]"),
            NodeStatus::Pruned => self.format(Color::Red, Style::Dim, "[刪]"),
            NodeStatus::Locked => self.format(Color::Blue, Style::Bold, "[鎖]"),
            NodeStatus::Failed => self.format(Color::Red, Style::Bold, "[敗]"),
        };

        let confidence_str = format!("{:.2}", node.confidence);
        let confidence_display = if node.confidence > 0.8 {
            self.format(Color::Green, Style::Normal, &confidence_str)
        } else if node.confidence >= 0.5 {
            self.format(Color::Yellow, Style::Normal, &confidence_str)
        } else {
            self.format(Color::Red, Style::Normal, &confidence_str)
        };

        // 階段標記
        let phase_marker = match phase {
            QuestionPhase::Exploration => self.format(Color::White, Style::Dim, "[探索]"),
            QuestionPhase::Development => self.format(Color::Cyan, Style::Dim, "[發展]"),
            QuestionPhase::Mature => self.format(Color::Magenta, Style::Dim, "[成熟]"),
        };

        // 內容預覽（最多 40 字）
        let content_preview = node.content.chars().take(40).collect::<String>();
        let content_display = if node.status == NodeStatus::Locked {
            self.format(Color::Blue, Style::Bold, &content_preview)
        } else {
            self.format(color, Style::Normal, &content_preview)
        };

        format!(
            "{} {} {} conf:{} weight:{:.2} {}",
            phase_marker,
            status_marker,
            content_display,
            confidence_display,
            node.weight,
            if node.status == NodeStatus::Locked {
                self.format(Color::Blue, Style::Bold, "🔒")
            } else {
                String::new()
            }
        )
    }

    /// 顯示圖例
    fn display_legend(&self) {
        println!("{}", self.format(Color::DarkGray, Style::Normal, &"─".repeat(50)));
        println!("{}", self.format(Color::White, Style::Dim, "圖例："));
        println!(
            "  {} {}",
            self.format(Color::Blue, Style::Bold, "[鎖]"),
            self.format(Color::DarkGray, Style::Normal, "鎖定節點")
        );
        println!(
            "  {} {} {} {} {}",
            self.format(Color::Green, Style::Normal, "conf > 0.8"),
            self.format(Color::DarkGray, Style::Normal, "="),
            self.format(Color::Green, Style::Normal, "高信心"),
            self.format(Color::Yellow, Style::Normal, "0.5-0.8"),
            self.format(Color::DarkGray, Style::Normal, "中"),
        );
        println!(
            "  {} {}",
            self.format(Color::Red, Style::Normal, "conf < 0.5"),
            self.format(Color::DarkGray, Style::Normal, "低信心")
        );
        println!(
            "  {} {}  {} {}  {} {}",
            self.format(Color::White, Style::Dim, "[探索]"),
            self.format(Color::DarkGray, Style::Normal, "探索期"),
            self.format(Color::Cyan, Style::Dim, "[發展]"),
            self.format(Color::DarkGray, Style::Normal, "發展期"),
            self.format(Color::Magenta, Style::Dim, "[成熟]"),
            self.format(Color::DarkGray, Style::Normal, "成熟期")
        );
    }

    /// 顯示簡潔的狀態列
    pub fn display_status_bar(&self, graph: &Graph, phase: QuestionPhase) {
        let phase_color = get_phase_color(phase);
        let phase_name = get_phase_name(phase);

        let locked_count = graph.get_all_nodes()
            .iter()
            .filter(|n| n.status == NodeStatus::Locked)
            .count();

        let avg_confidence = if graph.node_count() > 0 {
            let sum: f64 = graph.get_all_nodes()
                .iter()
                .filter(|n| n.status != NodeStatus::Pruned)
                .map(|n| n.confidence)
                .sum();
            let count = graph.get_all_nodes()
                .iter()
                .filter(|n| n.status != NodeStatus::Pruned)
                .count();
            if count > 0 { sum / count as f64 } else { 0.0 }
        } else {
            0.0
        };

        print!("  {} ", self.format(phase_color, Style::Bold, "[階段]"));
        print!("{} | ", self.format(Color::White, Style::Normal, phase_name));
        print!("節點:{} ", graph.node_count());
        print!("鎖定:{} ", locked_count);
        print!("平均信心:{:.2}", avg_confidence);
        println!();
    }

    /// 顯示約束條件列表（彩色）
    pub fn display_constraints(&self, constraints: &[String]) {
        if constraints.is_empty() {
            println!("{}", self.format(Color::DarkGray, Style::Normal, "（無約束條件）"));
            return;
        }

        println!("{}", self.format(Color::White, Style::Bold, "約束條件："));
        for (i, c) in constraints.iter().enumerate() {
            println!("  {}. {}", self.format(Color::Yellow, Style::Normal, &format!("[{}]", i + 1)), c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_panel_new() {
        let panel = VisualPanel::new();
        assert!(panel.colored);
    }

    #[test]
    fn test_no_color_panel() {
        let panel = VisualPanel::no_color();
        assert!(!panel.colored);
    }

    #[test]
    fn test_set_color() {
        let mut panel = VisualPanel::new();
        panel.set_color(false);
        assert!(!panel.colored);
    }

    #[test]
    fn test_format_plain() {
        let panel = VisualPanel::no_color();
        let result = panel.format(Color::Red, Style::Bold, "test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_format_colored() {
        let panel = VisualPanel::new();
        let result = panel.format(Color::Red, Style::Bold, "test");
        // ANSI code 會包含 \x1b[
        assert!(result.contains("test"));
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_get_phase_color() {
        assert_eq!(get_phase_color(QuestionPhase::Exploration), Color::White);
        assert_eq!(get_phase_color(QuestionPhase::Development), Color::Cyan);
        assert_eq!(get_phase_color(QuestionPhase::Mature), Color::Magenta);
    }

    #[test]
    fn test_display_legend() {
        let panel = VisualPanel::no_color();
        // 確認不會 panic
        panel.display_legend();
    }
}
