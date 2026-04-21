# Evolution Compiler 多節點架構

## 設計原則

- **每個節點職責單一**：不做超過自己邊界的事
- **節點之間用結構化資料溝通**：不用自然語法，降低推理負擔
- **可獨立測試每個節點**：不需要整個系統才能驗證
- **節點失敗有明確錯誤回報**：不是 exception，是診斷報告

---

## 節點流程圖

```
User Intent (自然語法)
     ↓
┌─────────────────────────────────────────────────────────┐
│  1. Intent Classifier（意圖分類器）                      │
│     輸入：自然語法意圖                                   │
│     輸出：IntentType + IntentProfile                    │
│         IntentType: CRUD | DASHBOARD | GAME | TOOL | API│
│         IntentProfile: {                                │
│           type,       // CRUD/DASHBOARD/GAME/TOOL/API  │
│           entities,   // 名詞：報表、任務、庫存          │
│           actions,    // 動詞：新增、刪除、搜尋、匯出    │
│           context,    // 額外描述                        │
│           target,     // 輸出目標：html/react/flutter    │
│           theme       // 主題傾向：dark/light/minimal    │
│         }                                              │
└─────────────────────────────────────────────────────────┘
     ↓
┌─────────────────────────────────────────────────────────┐
│  2. Schema Inferrer（資料結構推斷器）                    │
│     輸入：IntentProfile                                 │
│     輸出：InferredSchema（欄位列表 + 類型 + 驗證規則）   │
│         [                                              │
│           { name, label, type, required, options },     │
│           ...                                           │
│         ]                                              │
│     規則：每個 IntentType 有自己的 schema 推理邏輯       │
│         CRUD: 新增/編輯/刪除 + 清單視圖                 │
│         DASHBOARD: 圖表 + 統計卡 + 趨勢線              │
│         GAME: 分數/等級/存檔/多人機制                   │
└─────────────────────────────────────────────────────────┘
     ↓
┌─────────────────────────────────────────────────────────┐
│  3. Skill Router（技能路由器）                          │
│     輸入：IntentProfile + InferredSchema                │
│     輸出：SkillChain（已排序的技能列表）                │
│         [                                              │
│           { skill: "layout-header", weight: 0.9 },     │
│           { skill: "table-data", weight: 0.8 },        │
│           { skill: "theme-modern", weight: 0.7 },       │
│           ...                                          │
│         ]                                              │
│     邏輯：根據 entities + actions + schema，             │
│           打分每個 skill 的適用程度，輸出 Top-K          │
└─────────────────────────────────────────────────────────┘
     ↓
┌─────────────────────────────────────────────────────────┐
│  4. Dependency Resolver（依賴解析器）                   │
│     輸入：SkillChain                                    │
│     輸出：OrderedSkills（拓扑排序後的技能列表）          │
│         [                                              │
│           { skill: "base-css", depends: [] },           │
│           { skill: "theme-modern", depends: ["base-css"]},
│           { skill: "layout-header", depends: ["theme-modern"]},
│           ...                                          │
│         ]                                              │
│     邏輯：每個 skill 的 # depends: 宣告 → 拓扑排序      │
│           檢測循環依賴 → 報錯                          │
└─────────────────────────────────────────────────────────┘
     ↓
┌─────────────────────────────────────────────────────────┐
│  5. Composer（組合器）                                  │
│     輸入：OrderedSkills + InferredSchema + IntentProfile│
│     輸出：CompiledOutput（HTML/React/Swift...）         │
│         {                                              │
│           code: "...",                                 │
│           warnings: [...],  // 技能衝突、棄用技能      │
│           metadata: { skills_used, schema, theme }     │
│         }                                              │
│     邏輯：每個 skill 的 [html]/[react] 區塊 → 組合      │
│           schema-driven 動態生成（見下方合成規則）      │
└─────────────────────────────────────────────────────────┘
     ↓
┌─────────────────────────────────────────────────────────┐
│  6. QA Checker（品質檢查器）                            │
│     輸入：CompiledOutput + IntentProfile                │
│     輸出：QAResult                                     │
│         {                                              │
│           passed: true/false,                          │
│           issues: [                                    │
│             { level: "error"|"warning"|"info",        │
│               message: "...",                          │
│               location: "css"|"js"|"html"             │
│             }                                          │
│           ]                                            │
│         }                                              │
│     檢查項目：                                          │
│         - 完整性：必要功能是否都有實作                  │
│         - 一致性：handler 數量和 schema 一致            │
│         - 安全性：innerHTML 用 replace、 無 eval       │
│         - 主題：CSS 變數有正確覆蓋                     │
└─────────────────────────────────────────────────────────┘
```

---

## 合成規則（Composer 層）

### Schema-Driven 動態生成

```
InferredSchema.fields[]
     ↓
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Table Header │ + │  Form Fields  │ + │  Handler JS  │
│  (thead)      │   │  (modal)     │     │  (openAdd/  │
│               │   │              │     │   openEdit/ │
│               │   │              │     │   onSubmit) │
└──────────────┘     └──────────────┘     └──────────────┘
```

每個 field type 映射到對應的 UI 元件和 handler：
- `text` → `<input type="text">` + `field.value`
- `date` → `<input type="date">` + `field.value`
- `badge` → `<select>` + 高/中/低 options
- `checkbox` → `<input type="checkbox">` + `toggleComplete(id)`
- `action` → 按鈕組合 + `openEdit`/`openDelete`
- `sortable` → JS sort handler

---

## IntentType 定義

### CRUD
- **典型意圖**：「做一個代辦事項系統」「庫存管理系統」「客戶管理」
- **需要的技能**：`layout-header`, `table-data`, `modal-form`, `button-*`, `toast-notify`, `search-bar`, `badge-status`, `theme-*`
- **輸出**：單頁 HTML 或 React 组件

### DASHBOARD
- **典型意圖**：「做一個數據儀表板」「後台統計頁面」
- **需要的技能**：`layout-dashboard`, `card-stat`, `chart-*`, `table-data`, `theme-*`
- **輸出**：儀表板 HTML（ECharts 圖表）

### GAME
- **典型意圖**：「做一個小遊戲」「俄羅斯方塊」「射擊遊戲」
- **需要的技能**：`game-canvas`, `game-loop`, `game-physics`, `score-board`, `local-storage`
- **輸出**：單頁 HTML5 Canvas 遊戲

### TOOL
- **典型意圖**：「做一個計時器」「單位換算器」「密碼產生器」
- **需要的技能**：`layout-simple`, `input-*`, `result-display`, `theme-*`
- **輸出**：單頁工具 HTML

### API
- **典型意圖**：「做一個 REST API」「登入認證服務」
- **需要的技能**：`api-router`, `auth-*`, `db-connector`, `middleware-*`
- **輸出**：Python/Node.js API 程式碼

---

## Skill Router 打分邏輯

```python
def score_skill(skill, intent_profile, schema):
    score = 0.0
    
    # 1. 關鍵字匹配（30%）
    for kw in skill.keywords:
        for entity in intent_profile.entities:
            if kw in entity or entity in kw:
                score += 0.3
        for action in intent_profile.actions:
            if kw in action or action in kw:
                score += 0.2
    
    # 2. Schema 類型覆蓋（40%）
    for field in schema:
        if skill.handles_type(field.type):
            score += 0.4 / len(schema)
    
    # 3. IntentType 匹配（30%）
    if intent_profile.type in skill.supported_types:
        score += 0.3
    
    return min(score, 1.0)
```

---

## 錯誤回報格式

```python
class EvolutionError(Exception):
    def __init__(self, node: str, code: str, message: str, detail: str = ""):
        self.node = node          # "IntentClassifier" | "SkillRouter" | ...
        self.code = code          # "E001" | "E002" | ...
        self.message = message    # 人类可读
        self.detail = detail      # 技术细节
```

錯誤代碼：
- `E001`：Intent Classifier 無法分類（意圖太模糊）
- `E002`：Schema Inferrer 推斷失敗（缺少足夠實體資訊）
- `E003`：Skill Router 找不到適用技能（技能庫不足）
- `E004`：循環依賴檢測（技能之間互相依賴）
- `E005`：QA 檢查失敗（輸出品質未達標準）

---

## 檔案結構

```
software/
├── engine.py              # 主入口：synthesize() 協調所有節點
├── nodes/
│   ├── __init__.py
│   ├── intent_classifier.py   # 節點 1
│   ├── schema_inferrer.py     # 節點 2
│   ├── skill_router.py       # 節點 3
│   ├── dependency_resolver.py # 節點 4（重構自現有 topological sort）
│   ├── composer.py           # 節點 5（重構自 html_compiler/react_compiler）
│   └── qa_checker.py         # 節點 6
├── compiler/
│   ├── html_compiler.py      # 保留（Composer 的子模組）
│   └── react_compiler.py     # 保留（Composer 的子模組）
├── skills/
│   ├── ui/                   # UI 技能
│   ├── styles/               # 主題技能
│   ├── core/                 # 核心技能
│   └── ...
└── ARCHITECTURE.md           # 本文件
```

---

## 現有程式碼對應

| 現有檔案 | 對應節點 | 需改動 |
|----------|----------|--------|
| `intent_parser.py` | Intent Classifier + Skill Router | 重寫 keyword map → 分層推理 |
| `html_compiler.py` | Composer（子模組）| 保留介面，內部重構 |
| 現有 topological sort | Dependency Resolver | 抽出成獨立節點 |
| 無 | Schema Inferrer | 新增 |
| 無 | QA Checker | 新增 |

---

## 測試策略

每個節點獨立測試：
- `test_intent_classifier.py`：用已知意圖測試分類正確性
- `test_schema_inferrer.py`：用 CRUD/DASHBOARD/TOOL 類型測試推斷
- `test_skill_router.py`：確認技能分數合理性
- `test_dependency_resolver.py`：確認循環依賴檢測
- `test_qa_checker.py`：確認各類問題能檢出

端到端測試：
- L1（代辦事項）：Intent → 輸出 → QA 全流程
- L2（儀表板）：Intent → 輸出 → QA 全流程
- L3（游戲）：Intent → 輸出 → QA 全流程
