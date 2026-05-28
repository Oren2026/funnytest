//! 節點圖快照（Graph Snapshot）
//!
//! 在每個對話回合結束時，儲存當前推理圖的 XML 快照。

use std::fs;
use std::path::PathBuf;
use chrono::Local;
use crate::models::Graph;

/// Graph Snapshot Logger
///
/// 負責寫入節點圖快照。
/// 檔案位置：`workspace/logs/snapshots/snapshot_{round}_{timestamp}.xml`
#[derive(Debug, Clone)]
pub struct SnapshotLogger {
    /// Snapshots 目錄路徑
    snapshots_dir: PathBuf,
}

impl SnapshotLogger {
    /// 建立新的 SnapshotLogger
    pub fn new(snapshots_dir: &PathBuf) -> Self {
        SnapshotLogger {
            snapshots_dir: snapshots_dir.clone(),
        }
    }

    /// 儲存圖快照
    ///
    /// # 引數
    /// - `graph`: 要儲存的圖
    /// - `round`: 當前回合編號
    pub fn save_snapshot(&self, graph: &Graph, round: usize) -> std::io::Result<PathBuf> {
        // 確保目錄存在
        fs::create_dir_all(&self.snapshots_dir)?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("snapshot_{}_{}.xml", round, timestamp);
        let path = self.snapshots_dir.join(&filename);

        let xml = self.graph_to_xml(graph);
        fs::write(&path, &xml)?;

        Ok(path)
    }

    /// 將 Graph 轉換為 XML
    fn graph_to_xml(&self, graph: &Graph) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<graph>\n");

        // Metadata
        xml.push_str("  <meta>\n");
        xml.push_str(&format!(
            "    <timestamp>{}</timestamp>\n",
            Local::now().to_rfc3339()
        ));
        xml.push_str(&format!("    <node_count>{}</node_count>\n", graph.node_count()));
        xml.push_str(&format!("    <edge_count>{}</edge_count>\n", graph.edge_count()));
        xml.push_str(&format!(
            "    <total_complexity>{:.4}</total_complexity>\n",
            graph.total_complexity()
        ));
        xml.push_str("  </meta>\n");

        // Nodes
        xml.push_str("  <nodes>\n");
        for node in graph.get_all_nodes() {
            let status_str = match node.status {
                crate::models::NodeStatus::Draft => "Draft",
                crate::models::NodeStatus::Active => "Active",
                crate::models::NodeStatus::Pruned => "Pruned",
                crate::models::NodeStatus::Locked => "Locked",
            };
            xml.push_str(&format!(
                "    <node id=\"{}\" step=\"{}\" weight=\"{:.4}\" confidence=\"{:.4}\" complexity=\"{:.4}\" status=\"{}\">\n",
                node.id,
                node.step,
                node.weight,
                node.confidence,
                node.complexity,
                status_str
            ));
            xml.push_str(&format!(
                "      <content><![CDATA[{}]]></content>\n",
                node.content
            ));
            xml.push_str("    </node>\n");
        }
        xml.push_str("  </nodes>\n");

        // Edges
        xml.push_str("  <edges>\n");
        for edge in graph.get_all_edges() {
            let edge_type_str = match edge.edge_type {
                crate::models::EdgeType::Reasoning => "Reasoning",
                crate::models::EdgeType::Constraint => "Constraint",
                crate::models::EdgeType::Divergence => "Divergence",
            };
            xml.push_str(&format!(
                "    <edge id=\"{}\" from=\"{}\" to=\"{}\" type=\"{}\" weight=\"{:.4}\" />\n",
                edge.id, edge.from, edge.to, edge_type_str, edge.weight
            ));
        }
        xml.push_str("  </edges>\n");

        xml.push_str("</graph>\n");

        xml
    }

    /// 取得 snapshots 目錄路徑
    pub fn snapshots_dir(&self) -> &PathBuf {
        &self.snapshots_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Node, Edge, EdgeType};

    #[test]
    fn test_snapshot_logger_new() {
        let logger = SnapshotLogger::new(&PathBuf::from("/tmp/snapshots"));
        assert_eq!(logger.snapshots_dir().to_str(), Some("/tmp/snapshots"));
    }

    #[test]
    fn test_graph_to_xml() {
        let logger = SnapshotLogger::new(&PathBuf::from("/tmp/snapshots"));

        let mut graph = Graph::new();
        let node1 = Node::new("節點1".to_string(), 1);
        let node2 = Node::new("節點2".to_string(), 2);

        graph.add_node(node1);
        graph.add_node(node2);

        let xml = logger.graph_to_xml(&graph);

        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<graph>"));
        assert!(xml.contains("</graph>"));
        assert!(xml.contains("<nodes>"));
        assert!(xml.contains("<edges>"));
        assert!(xml.contains("節點1"));
        assert!(xml.contains("節點2"));
    }

    #[test]
    fn test_save_snapshot() {
        let temp_dir = std::env::temp_dir().join("test_snapshots");
        let logger = SnapshotLogger::new(&temp_dir);

        let mut graph = Graph::new();
        let node = Node::new("測試節點".to_string(), 1);
        graph.add_node(node);

        let result = logger.save_snapshot(&graph, 1);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_str().unwrap().contains("snapshot_1_"));

        // cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
