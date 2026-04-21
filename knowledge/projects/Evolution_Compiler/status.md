# Evolution Compiler

## 基本資訊
- **口號**: 「能做軟體的軟體」—— 輸入自然語法意圖，輸出完整、可用的軟體
- **願景**: 軟體是 AI 用的工具，輸出要有品質，内部不需要 UI
- **位置**: `~/Desktop/Oren_own/evolution_compiler/`
- **Repo**: git@github.com:Oren2026/funnytest.git

## 核心架構（確立的共識）

### Multi-Agent Routing
每個節點分散決策，不靠單一強節點：
1. **Intent Classifier** → 分析意圖類型（CRUD/遊戲/工具/資料視覺化...）
2. **Schema Inferrer** → 根據類型推斷資料欄位和操作
3. **Skill Router** → 分散式推理（不用 keyword map）
4. **Composer** → 組合技能、解決依賴
5. **QA Checker** → 輸出前完整性、一致性、安全性檢查

### 輸出格式（共識）
- Explicit > Abstract（已實測驗證）
- 抽象 constraint 標籤（如 `no_heap_allocation`）→ AI 遵從率 ~0%
- 顯式禁止（如 `DO NOT use: malloc, calloc, realloc, free`）→ AI 遵從率 ~100%
- 細粒度 skill > 粗粒度（+55-85%）

## seed 格式（共識）
```xml
<meta>
    <use name="array-sort" />
</meta>
<constraint>
    <prohibit names="malloc,calloc,realloc,free" />
</constraint>
<body>
    <sort-array name="arr" />
</body>
```

## 技能分層結構（6+6+4）

### 底層（6個，不可拆解）
array-base, comparison, signal-handler, loop-pattern, memory-static, printf-basic

### 中層（6個，依賴底層）
sort-bubble, sort-insertion, search-linear, search-binary, linked-list-static, stack-static

### 高層（4個，依賴中層）
queue-static, sort-quick, daemon-loop, timer-periodic

Engine 支援遞迴依賴解析：`<use name="queue-static">` → 自動展開依賴鏈。

## 已實作產物

位於 `~/Desktop/Oren_own/evolution_compiler/`：
- `parser.py` — 新格式解析（支援 `<use>`, `<prohibit>`, `<body>` 元件）
- `validator.py` — 顯式禁止清單驗證
- `ai_module.py` — 技能組合引擎（遞迴依賴解析）
- `engine.py` — 協調器
- `watch.py` — 語意指紋漂移控制
- 16 個 .skill 檔案（6+6+4 分層結構）

## 現有技能庫
- **UI skills（9個）**: layout-header, table-data, button-primary/danger, modal-form, toast-notify, search-bar, badge-status, confirm-dialog
- **Themes（4個）**: glass, modern, brutal, soft
- **Section markers**: `[html]/[react]/[style]` 支援多平台輸出

## 尚未完成
- [x] 技能缺口補足 — card-group, form-layout, pagination, tabs, sidebar, loading, empty-state, progress-bar（8/8 完成）
- [x] gap-1. 技能缺口分析完成：原有9個 → 現在17個 UI 技能
- [ ] Semantic intent parsing（脫離 keyword match）
- [ ] 輸出品質驗證機制

## 已完成（2026-04-21）
- [x] ai_module.py 升級（遞迴依賴解析）
- [x] engine.py 整合依賴解析
- [x] 端到端測試：`<use name="queue-static" />` 遞迴展開驗證
- [x] L1 測試流程 test_runner.py（初版完成）
- [x] Intent Classifier / Schema Inferrer / Skill Router / Dependency Resolver / Composer / QA Checker 六節點落地
- [x] 9 UI skills + 4 Theme skills

## 待解決問題（2026-04-21 發現）
- **composer.py bug**: `_compose_html()` 內有大量 hardcoded table-data 邏輯（line 270-320），skill block 拼接方式導致 schema-driven 表單/JS 未正確注入。test_runner.py 測試時會暴露此問題。
- **根本原因**: composer 沒有真正做到「技能組合」，而是 hardcoded 了一堆 layout 邏輯

## 技術決策摘要
- 不要靠單一強節點補償，要各節點高效益運算
- 自然語意用 keyword match 而非 AI 推理（更穩定）
- 軟體本體（software/）和測試產出（demo/）必須分開
- subagent 複雜修改應從乾淨 HEAD 重新實作，不要 incremental patch

## 迭代計劃（2026-04-21 起）

### 兩三天內目標
1. **技能缺口補足** — ✅ 完成（8/8）
2. **L1 測試流程** — ✅ 完成（PASS 0 issues）
3. **composer 重構** — ✅ 完成（skill-based composition）
4. **review-1** — 驗證補足技能後輸出品質變化（next）

### 目前處於
**v0.9** — composer 重構完成，skill-based composition + slot injection，test_runner QA 0 issues
**下一步** — review-1：第一輪回顧，驗證新技能組合輸出品質

### 迭代節奏
- 每個 commit = 一個可運行狀態，不堆code不commit
- commit 後更新 status.md 的「目前進展」與「待辦」
- 兩三天後根據實際輸出品質決定下一步

## 最後更新
2026-04-22
