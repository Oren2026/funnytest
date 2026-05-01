# Evolution Compiler

## 基本資訊
- **口號**: 「能做軟體的軟體」—— 輸入自然語法意圖，輸出完整、可用的軟體
- **願景**: 軟體是 AI 用的工具，輸出要有品質，内部不需要 UI
- **位置**: `~/Desktop/funnytest/`（Hermes 獨立工作區）
- **Repo**: git@github.com:Oren2026/funnytest.git
- **當前版本**: v2 — Spec-aware（Phase 1 完成）

## 核心架構（共識）

### Multi-Agent Routing
每個節點分散決策，不靠單一強節點：
1. **Intent Classifier** → 分析意圖類型（CRUD/遊戲/工具/資料視覺化...）
2. **Schema Inferrer** → 根據類型推斷資料欄位和操作
3. **Skill Router** → 分散式推理（不用 keyword map）
4. **Composer** → **Spec-aware 技能組合**，SkillRegistry 動態解析 slot
5. **QA Checker** → 輸出前完整性、一致性、安全性檢查

### 核心進化（v2）
從「技能拼接」（Skill Stitching）升級為「Spec 驅動組合」（Spec-driven Composition）。

---

## Phase 1 完成（2026-04-30）

### Spec 格式標準 ✅
- 檔案：`software/skills/_SPEC_FORMAT.md`
- 定義五區塊：Contract / Dependencies / Slots / Boundaries / Examples
- 取代舊格式：`# depends:` + `[html]` 區塊

### table-data.skill Pilot ✅
- 第一個完成 Spec 化的 Skill
- Commit: `a089bcd`

### Composer Spec-aware 改寫 ✅
- 新增 `SkillSpec` dataclass：解析完整 Spec 五區塊
- 新增 `SkillRegistry`：slot → skill 反冊索引
- `_compose_html()` 用 Registry 解析 slot，fallback 有 Warning
- Commit: `4048ab1`

### L1 測試 ✅
- PASSED — 0 issue(s)
- Composer Warning 精確指出哪些 Skill 尚未 Spec 化

---

## Spec 化進度

| 維度 | 舊版 | Spec 版 |
|------|------|---------|
| AI 組裝時 | 知道「怎麼拼」 | 知道「什麼情境用這個」 |
| 失敗時 | 不清楚哪個 Skill 的問題 | 有 failure signal，可以定位 |
| 缺口暴露 | 需要跑 QA 才發現 | Composer Warning 主動指出 |

**進度：1/41**

---

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

---

## 待定義：Phase 3 進階能力

- `find_skill_for_slot()` 語意匹配（不只是名字包含）
- `Contract.failure_signals` 驅動 QA 定位
- `Boundaries` 驅動組裝時驗證
- `Examples` 驅動 Spec-level 單元測試

## 待解決問題

- [ ] 大多數 Skill 尚未 Spec 化（1/41）
- [ ] Phase 3 Data Flow Validation 未完成（`validate_data_flow()` 是 stub）
- [ ] React 輸出不完整
- [ ] engine.py 仍是 C 代碼生成導向，與 L1 測試流程脫鉤
- [ ] Skill Router 的 SKILL_INDEX 為靜態字典
- [ ] Schema Inferrer 無 LLM

## 最後更新
2026-04-30（Phase 1 完成，Spec-aware v2）
