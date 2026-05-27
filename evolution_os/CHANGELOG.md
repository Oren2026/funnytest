# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.1.0] — 2026-05-27

### Added

- **`planner` module** — 任務規劃與分工決策核心
  - `stages.rs`: Stage 枚舉（S1Confirming / S2Analyzing / S3Planning / Complete）
  - `decision.rs`: ComplexityMetrics、DispatchDecision、WorkMode（Fork/Solo）
  - `manifest.rs`: PlannerManifest、Requirement、QuestionItem、EstimatedNode

- **`planner_cli` binary** — 命令行入口
  - 直接模式：`cargo run --bin planner_cli -- "<任務>"` → 終端分析報告 + JSON
  - 互動模式（`--interactive`）：多行輸入

- **整合測試**
  - `tests/planner_decision.rs`: decision.rs 整合測試（6 cases，驗證分工閾值）

- **`VERSION_CONTROL.md`** — 版本控制規劃文件
  - SemVer 命名規則
  - Branch 模型（GitHub Flow）
  - Commit Message 格式（Conventional Commits）
  - CHANGELOG 維護流程
  - 測試覆蓋率目標

- **`CHANGELOG.md`** — 本文件

### Changed

- `src/lib.rs`: 加入 `pub mod planner;` 和 `pub use planner::*;`

- `Cargo.toml`: 新增 `chrono = "0.4"` dependency（manifest timestamp 使用）

- **`planner/decision.rs`**
  - 固定 array 类型标注（`[(&[&str], &str); N]`）修复 Rust 编译器对异质数组的推断
  - `extract_domain_tags`: 补强 keyword 列表（database、frontend、backend、auth、devops、security、performance、testing）
  - `count_domain_diversity`: 改用 `.iter()` 遍历避免 `&&str` 类型错误

### Fixed

- `planner/decision.rs`: `count_domain_diversity` 数组长度不一导致的 type inference 错误
- `planner/decision.rs`: `extract_domain_tags` `&&str` not iterator 错误（改用 `.iter()`）
- `planner/decision.rs`: `tech_terms` 变量名拼写错误（应为 `has_tech_terms`）
- `planner_decision.rs`: 测试中 `frontend` tag 识别失败（新增 "網站/web/頁面" keyword）

### Removed

- （无）

---

## [0.0.0] — 2026-05-?? (Project Initialized)

### Added

- Initial project structure: `node`, `chain`, `skill`, `runtime`, `model`, `storage`, `evo`
- OllamaBackend for model dispatching
- MemoryGraph for call chain tracking
- Executor for sequential node execution
- SkillRegistry and built-in skill nodes
- JsonStorage for graph persistence