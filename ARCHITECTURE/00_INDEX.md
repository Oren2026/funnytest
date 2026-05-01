# Evolution Compiler — 系統架構總覽（v2 — Spec-aware）

## 定位

Intent-driven 多平台代碼生成框架，目標是「讓 AI 更容易迭代程式」的語意合約層。

**核心進化**：從「技能拼接」（Skill Stitching）升級為「Spec 驅動組合」（Spec-driven Composition）。

---

## 核心五節點管道

```
輸入文字
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│  ① Intent Classifier   （意圖分類器）                      │
│     software/nodes/intent_classifier.py                  │
│     輸入：自然語言意圖字串                                  │
│     輸出：IntentProfile（type, entities, actions,         │
│                 context, target, theme）                  │
│     控制邏輯：關鍵詞模式匹配 → 6 種 IntentType             │
└────────────────────┬────────────────────────────────────┘
                     │ IntentProfile
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ② Schema Inferrer    （資料結構推斷器）                   │
│     software/nodes/schema_inferrer.py                   │
│     輸入：IntentProfile                                  │
│     輸出：List[Dict] — 欄位定義列表                        │
│     控制邏輯：根據 IntentType 分支推斷欄位                 │
└────────────────────┬────────────────────────────────────┘
                     │ schema: List[Dict]
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ③ Skill Router        （技能路由器）                     │
│     software/nodes/skill_router.py                       │
│     輸入：IntentProfile + Schema                         │
│     輸出：List[Dict] — [{skill, score}] 排序清單         │
│     控制邏輯：加權評分（type 匹配 + keyword + base weight） │
└────────────────────┬────────────────────────────────────┘
                     │ skill_chain: [{skill, score}]
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ④ Dependency Resolver  （依賴解析器）                     │
│     software/nodes/dependency_resolver.py                │
│     輸入：List[str] — 技能名稱列表                        │
│     輸出：List[Dict] — [{skill, depends}] 拓撲排序         │
│     控制邏輯：Kahn 演算法 + 遞迴擴展依賴圖                  │
└────────────────────┬────────────────────────────────────┘
                     │ ordered_skills: [{skill, depends}]
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ⑤ Composer             （組合器）— Spec-aware           │
│     software/nodes/composer.py                          │
│     輸入：ordered_skills + schema + profile + output_type │
│     輸出：Dict — {code, warnings, metadata}               │
│     控制邏輯：                                           │
│     1. SkillRegistry 建立 slot→skill 反向索引             │
│     2. 根據 Spec（Contract/Slots/Boundaries）解析 slot   │
│     3. Fallback 時發 warning，精確指出哪個 slot 缺 Spec   │
│     4. slot injection + schema-driven form/table         │
└────────────────────┬────────────────────────────────────┘
                     │ compiled: {code, warnings, metadata}
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ⑥ QA Checker            （品質檢查器）                   │
│     software/nodes/qa_checker.py                        │
│     輸入：compiled + profile + schema                   │
│     輸出：Dict — {passed, issues: [QAIssue]}             │
│     控制邏輯：結構 / JS 完整性 / 安全性 / Schema 覆蓋     │
└─────────────────────────────────────────────────────────┘
                     │ QA result
                     ▼
                  產出檔案
                (demo/NAME.html)
```

---

## 節點職責地圖

| 節點 | 檔案 | 核心職責 | 決策方式 |
|------|------|----------|----------|
| ① Intent Classifier | `intent_classifier.py` | 把文字分類為 IntentType + 提取實體/動作/主題 | 關鍵詞匹配（優先順序規則） |
| ② Schema Inferrer | `schema_inferrer.py` | 根據 IntentType 推斷資料欄位 | 規則分支 + 正規表達式解析 |
| ③ Skill Router | `skill_router.py` | 從技能庫打分排序候選技能 | 加權評分（5 個維度） |
| ④ Dependency Resolver | `dependency_resolver.py` | 讀取技能 Spec 的依賴宣告並拓撲排序 | Kahn 演算法 |
| ⑤ Composer | `composer.py` | **Spec-aware 技能組合** | SkillRegistry 動態解析 slot |
| ⑥ QA Checker | `qa_checker.py` | 驗證輸出品質（結構/安全/完整性） | 規則檢查（error レベル強制失敗） |

---

## Spec 格式（技能語意合約）

每個 `.skill` 檔案包含五個語意區塊，取代原本的「程式碼區塊 + 依賴宣告」。

格式標準文件：`software/skills/_SPEC_FORMAT.md`

### 格式結構

```markdown
# skill: table-data

## Contract
- **語義承諾**：渲染一個可排序的資料表格，顯示庫存明細
- **輸入資料格式**：`Array<{id, name, category, quantity, status, updatedAt}>`
- **輸出語義**：HTML `<table>` 元素，帶有 thead/tbody slots
- **操作邊界**：
  - ✅ 做：動態渲染、排序、空狀態
  - ❌ 不做：新增/刪除邏輯、API 呼叫、localStorage 操作
- **失敗信號**：`items` 不是陣列 → 渲染 empty-state

## Dependencies
- **依賴**：`badge-status`
- **可選依賴**：`button-primary`, `button-danger`, `modal-form`
- **排斥**：無

## Slots
- **slot:thead**：欄位標題列，由 Schema 注入
- **slot:tbody**：動態資料行，由 Composer 注入
- **slot:empty-state**：空資料時的提示訊息

## Boundaries
- **系統邊界**：Presentation Layer
- **狀態邊界**：Stateless

## Examples
### 基本用法
**輸入**：`{items: [{id:1, name:"螺絲", qty:100}], ...}`
**輸出**：`<table>...</table>`
```

### Spec-aware Composition 的價值

| 維度 | 舊版（程式碼區塊） | Spec 版（語意合約） |
|------|---|---|
| AI 組裝時 | 知道「怎麼拼」 | 知道「什麼情境用這個」 |
| 失敗時 | 不清楚是 Skill 問題還是組裝問題 | 有 failure signal，可以定位 |
| 新增 Skill | 看程式碼片段判斷要不要用 | 看 Contract 描述就知道 |
| 組合順序 | 靠 resolve_dependencies 推斷 | 靠 Boundary 描述決定誰是 parent |
| 缺口暴露 | 需要跑 QA 才發現 | Skill 沒有 Spec 宣告時 Composer 主動發 Warning |

---

## 數據合約（跨節點介面）

### IntentProfile（①→②、①→③、⑤、⑥）

```python
@dataclass
class IntentProfile:
    type: IntentType          # CRUD | DASHBOARD | GAME | TOOL | API | UNKNOWN
    entities: List[str]       # ["任務", "客戶"]
    actions: List[str]        # ["新增", "刪除"]
    context: str              # 原始輸入文字
    target: str               # "html" | "react" | "flutter" | "swift"
    theme: str                # "modern" | "glass" | "brutal" | "soft"
```

### Schema（②→③、②→⑤、②→⑥）

```python
# List[Dict]，每個 Dict 代表一個欄位
{
    "name": str,       # 欄位名（英文駝峰）
    "label": str,      # 顯示標題
    "type": str,       # "text" | "badge" | "date" | "checkbox" | "action" | "sortable"
    "required": bool,
    "editable": bool,
    "options": List[str],   # badge/select 用
    "placeholder": str,
    "default": str,
}
```

### SkillSpec（新增，⑤ 內部使用）

```python
@dataclass
class SkillSpec:
    name: str
    semantic_promise: str     # 語義承諾
    input_format: str         # 輸入格式描述
    output_semantic: str      # 輸出語義
    does: List[str]           # ✅ 做的清單
    does_not: List[str]       # ❌ 不做的清單
    failure_signals: List[str] # 失敗信號
    dependencies: List[str]   # 依賴
    optional_deps: List[str]  # 可選依賴
    excludes: List[str]       # 排斥
    slots_provides: List[str] # 提供這些 slot
    slots_consumes: List[str] # 需要這些 slot
    boundary_layer: str       # 系統邊界
    is_stateful: bool         # 是否保有狀態
    html: str = ""            # 原始 [html] 區塊
    style: str = ""           # 原始 [style] 區塊
    react: str = ""           # 原始 [react] 區塊
```

### SkillChain（③→④、④→⑤）

```python
# ③輸出：打分後排序
[{"skill": "table-data", "score": 0.95}, {"skill": "modal-form", "score": 0.88}, ...]

# ④輸出：拓撲排序 + 依賴宣告
[{"skill": "toast-notify", "depends": []},
 {"skill": "button-primary", "depends": []},
 {"skill": "modal-form", "depends": ["button-primary"]},
 {"skill": "layout-header", "depends": []},
 ...]
```

### Compiled Output（⑤→⑥）

```python
{
    "code": str,          # 最終 HTML 字串
    "warnings": List[str], # Spec-aware fallback 警告
    "metadata": {
        "skills_used": List[str],
        "schema": List[Dict],
        "theme": str,
    }
}
```

### QA Result（⑥ 輸出）

```python
{
    "passed": bool,       # 無 error 即 true
    "issues": [
        QAIssue(level="error"|"warning"|"info",
                message=str,
                location=str)
    ]
}
```

---

## 技能系統（Spec 化進行中）

技能定義在 `software/skills/` 下的 `.skill` 檔案，逐漸從舊格式升級為 Spec 格式。

### Spec 化進度

| Skill | Spec 格式 | Pilot 狀態 |
|-------|-----------|-----------|
| `table-data` | ✅ 完成 | 第一個 pilot |

**進度：1/41** — 其餘 40 個技能仍是舊格式

### Skill 轉 Spec 格式的訊號

當 Composer 的 Warning 出現以下訊息，代表該 Skill 尚未 Spec 化：

```
[Spec-aware] slot 'header': no Spec declaration found, falling back to 'layout-header'
```

當所有 Skill 都完成 Spec 化後，這些 Warning 會完全消失。

### 技能分類目錄

| 目錄 | 內容 |
|------|------|
| `skills/ui/` | UI 元件技能（table-data, modal-form, button-*, badge-*, search-bar...） |
| `skills/styles/` | 主題技能（theme-glass, theme-modern, theme-brutal, theme-soft） |
| `skills/structures/` | 資料結構技能（stack, queue, linked-list） |
| `skills/system/` | 系統技能（timer, daemon-loop） |
| `skills/algorithms/` | 演算法技能（sorting, search） |
| `skills/core/` | C 核心技能（指標、記憶體） |

---

## 入口：test_runner.py（L1 測試流程）

```bash
cd software/
python test_runner.py                          # 跑預設 L1 測試
python test_runner.py --case todo              # 跑指定案例
python test_runner.py --list                   # 列出所有案例
```

測試案例格式：`*-context.md`

---

## 現有技能索引（截至 2026-04-30）

**UI 技能（19個）**：layout-header, layout-dashboard, table-data ✅, modal-form, button-primary, button-danger, toast-notify, search-bar, badge-status, confirm-dialog, sort-control, pagination, card-group, form-layout, empty-state, sidebar, tabs, progress-bar, loading

**主題技能（4個）**：theme-glass, theme-modern, theme-brutal, theme-soft

**圖表技能（3個）**：chart-line, chart-bar, card-stat

**遊戲技能（4個）**：game-canvas, game-loop, score-board, local-storage

**API 技能（2個）**：api-router, auth-jwt

---

## 各節點文件

| 文件 | 節點 |
|------|------|
| `01_IntentClassifier.md` | ① Intent Classifier |
| `02_SchemaInferrer.md` | ② Schema Inferrer |
| `03_SkillRouter.md` | ③ Skill Router |
| `04_DependencyResolver.md` | ④ Dependency Resolver |
| `05_Composer.md` | ⑤ Composer（已重寫，Spec-aware） |
| `06_QAChecker.md` | ⑥ QA Checker |
| `10_DataFlow.md` | 節點間數據流動合約 |
| `v2_FRAMEWORK.md` | 框架雛型（新方向，Spec-driven） |

---

## Spec 化路線圖

### Phase 1：Pilot ✅（2026-04-30）
- [x] 定義 `_SPEC_FORMAT.md`
- [x] `table-data.skill` 轉 Spec 格式
- [x] `composer.py` 新增 `SkillSpec` + `SkillRegistry`
- [x] Composer Spec-aware slot 解析 + fallback warning
- [x] L1 測試 PASS，commit `4048ab1`

### Phase 2：Rollout（進行中）
- [ ] 將剩餘 UI skill 依序轉 Spec（`modal-form`, `search-bar`, `badge-status`...）
- [ ] 逐步消滅 Composer Warning

### Phase 3：進階 Spec 能力（待定義）
- [ ] `find_skill_for_slot()` 語意匹配（不只是名字包含）
- [ ] `Contract.failure_signals` 驅動 QA 定位
- [ ] `Boundaries` 驅動組裝時驗證
- [ ] `Examples` 驅動 Spec-level 單元測試

---

## 已知缺口與待解決問題

- **大多數 Skill 尚未 Spec 化**（1/41）：Composer Warning 精確指出缺口，但尚未補完
- **Phase 3 Data Flow Validation 未完成**：`validate_data_flow()` 是 stub
- **React 輸出**：`composer.py` 有 `_compose_react()` 分支但內容不完整
- **engine.py**：仍是 C 代碼生成導向（舊版），與 L1 測試流程脫鉤
- **Skill Router 的 SKILL_INDEX 為靜態字典**：新技能無法自動被發現
- **Schema Inferrer 無 LLM**：純規則，複雜描述解析能力有限
