# 10 — Data Flow（數據流動合約）

## 管道總覽

```
context.md / intent_text
         │
         ▼
   ① Intent Classifier
         │ IntentProfile
         ▼
   ② Schema Inferrer
         │ schema: List[Dict]
         ▼
   ③ Skill Router
         │ skill_chain: [{skill, score}]
         ▼
   ④ Dependency Resolver
         │ ordered_skills: [{skill, depends}]
         ▼
   ⑤ Composer
         │ compiled: {code, warnings, metadata}
         ▼
   ⑥ QA Checker
         │ qa_result: {passed, issues}
         ▼
      產出 HTML
```

## 介面合約

### 合約 A：IntentProfile（①→②③④⑤⑥）

```python
class IntentType(Enum):
    CRUD = "CRUD"
    DASHBOARD = "DASHBOARD"
    GAME = "GAME"
    TOOL = "TOOL"
    API = "API"
    UNKNOWN = "UNKNOWN"

@dataclass
class IntentProfile:
    type: IntentType         # 决定 Schema 推斷分支
    entities: List[str]      # 决定表單/表格欄位命名
    actions: List[str]       # Skill Router 評分用
    context: str             # 主要用於 Theme 推斷
    target: str              # "html" | "react" | "flutter" | "swift"
    theme: str               # "modern" | "glass" | "brutal" | "soft"
```

**契約要點：**
- `type` 為 `UNKNOWN` 時，Schema Inferrer 降級為 CRUD
- `entities` 為空時，Schema Inferrer 用 "項目" 作為 entity fallback
- `theme` 只在 Skill Router 用於 theme skill 的加分，不影響其他邏輯

### 合約 B：Schema（②→③⑤⑥）

```python
# List[Dict]，每個 Dict 是欄位定義
{
    "name": str,       # JS 物件 key，也是 field-id
    "label": str,      # UI 顯示文字
    "type": str,       # text | badge | date | checkbox | action | sortable
    "required": bool,
    "editable": bool,  # False → Composer modal-form 跳過不產 input
    "options": List[str],  # badge/select 專用
    "placeholder": str,
    "default": str,
}
```

**契約要點：**
- `type: "action"` 的欄位 Composer 完全忽略（只用於 thead 顯示）
- `editable: False` 的欄位（如 id, createdAt）→ modal-form 不產生 input
- Composer 的 `_build_form_from_schema()` 只處理 `editable=True` 且 `type not in ("action",)` 的欄位
- Composer 的 JS render() 函式用 `schema[i].name` 作為 `item[key]` 的 key

### 合約 C：SkillChain（③→④）

```python
List[Dict]  # 只需 skill 名稱
[
    {"skill": "table-data", "score": 0.95},
    {"skill": "modal-form", "score": 0.88},
    {"skill": "badge-status", "score": 0.82},
    ...
]
```

**契約要點：**
- Dependency Resolver 只取 `.skill` 名稱，忽略 score
- 少於 8 個也正常處理（可少於 Top-8）

### 合約 D：OrderedSkills（④→⑤）

```python
List[Dict]  # 拓撲排序結果，含依賴宣告
[
    {"skill": "toast-notify", "depends": []},
    {"skill": "layout-header", "depends": []},
    {"skill": "button-primary", "depends": []},
    {"skill": "modal-form", "depends": ["button-primary"]},
    ...
]
```

**契約要點：**
- Composer 只取 `.skill` 名稱構建 `skills_used` 清單
- `depends` 欄位 Composer 目前**未使用**（依賴順序由拓撲排序保證）
- 循環依賴 → `resolve_dependencies()` 拋 `ValueError`

### 合約 E：Compiled（⑤→⑥）

```python
{
    "code": str,   # 完整 HTML 字串（從 <!DOCTYPE 到 </html>）
    "warnings": List[str],
    "metadata": {
        "skills_used": List[str],
        "schema": List[Dict],   # Schema 副本
        "theme": str,
    }
}
```

**契約要點：**
- `code` 必須是完整 HTML，QA Checker 不處理空字串或片段
- `metadata.skills_used` 用於 QA Checker 的 theme 驗證
- `metadata.schema` 用於 QA Checker 的 Schema 覆蓋檢查

### 合約 F：QA Result（⑥ 輸出）

```python
{
    "passed": bool,
    "issues": [
        QAIssue(
            level="error"|"warning"|"info",
            message=str,
            location=str
        )
    ]
}
```

**契約要點：**
- `passed = not any(level == "error" for issue in issues)`
- warning/info 不影響 `passed`
- `passed = True` **不等於輸出正確**，只是沒有被 QA 捕捉到的 error

## 數據流視覺化（一個 CRUD 例子的完整旅程）

### 輸入
```
「顯示待辦列表，包含標題、優先權、截止日期」
```

### ① IntentClassifier 輸出
```python
IntentProfile(
    type=CRUD,
    entities=["任務"],
    actions=["列表"],
    context="顯示待辦列表，包含標題、優先權、截止日期",
    target="html",
    theme="modern"
)
```

### ② SchemaInferrer 輸出
```python
[
    {"name": "id", "label": "ID", "type": "text", "required": False, "editable": False},
    {"name": "title", "label": "任務名稱", "type": "text", "required": True, "editable": True},
    {"name": "priority", "label": "優先權", "type": "badge", "options": ["高","中","低"], "default": "中"},
    {"name": "status", "label": "狀態", "type": "badge", "options": ["進行中","已完成","待處理"], "default": "待處理"},
    {"name": "dueDate", "label": "截止日期", "type": "date", "editable": True},
    {"name": "createdAt", "label": "建立時間", "type": "date", "editable": False},
    {"name": "updatedAt", "label": "更新時間", "type": "date", "editable": False},
    {"name": "actions", "label": "操作", "type": "action"}
]
```

### ③ SkillRouter 輸出
```python
[
    {"skill": "table-data", "score": 0.95},
    {"skill": "search-bar", "score": 0.88},
    {"skill": "modal-form", "score": 0.82},
    {"skill": "toast-notify", "score": 0.80},
    {"skill": "badge-status", "score": 0.75},
    {"skill": "layout-header", "score": 0.73},
    {"skill": "button-primary", "score": 0.60},
    {"skill": "button-danger", "score": 0.55}
]
```

### ④ DependencyResolver 輸出
```python
[
    {"skill": "toast-notify", "depends": []},
    {"skill": "layout-header", "depends": []},
    {"skill": "button-primary", "depends": []},
    {"skill": "badge-status", "depends": []},
    {"skill": "button-danger", "depends": []},
    {"skill": "search-bar", "depends": []},
    {"skill": "table-data", "depends": []},
    {"skill": "modal-form", "depends": ["button-primary"]}
]
```

### ⑤ Composer 注入的 Schema-driven 內容

**表單欄位（從 Schema 生成）：**
- field-title `<input type="text">` — 來自 `name: "title"`
- field-priority `<select>` — 來自 `name: "priority"`, `options: ["高","中","低"]`
- field-dueDate `<input type="date">` — 來自 `name: "dueDate"`, `type: "date"`
- createdAt/updatedAt — `editable: False` → 跳過

**Table thead（從 Schema 生成）：**
- `<th>ID</th><th>任務名稱</th><th>優先權</th><th>狀態</th><th>截止日期</th><th>操作</th>`

**Render switch-case（從 Schema 生成）：**
```javascript
case "title": return `<td>${item.title}</td>`;
case "priority": return _renderBadge(item.priority, item);
case "status": return _renderBadge(item.status, item);
case "dueDate": return `<td class="col-date">${item.dueDate}</td>`;
case "actions": return _renderActions(item);
```

### ⑥ QA Checker 檢查清單

- ✅ DOCTYPE、html、head、body 完整
- ✅ `function openAdd` / `openEdit` / `openDelete` 存在
- ✅ `field-title`, `field-priority`, `field-dueDate` input 存在
- ✅ 無 `eval()`、無危險 `innerHTML`
- ⚠️ `inventory-form` id 存在但 Skill Router 路由 `table-data` → 表單 id 衝突
- ℹ️ 無 `showToast` 但有 toast-notify skill → OK

## 已知 Data Flow 缺口

1. **Phase 3 缺失**：`validate_data_flow()` 需讀取 `## 初始資料` 段落才能驗證 seed data 是否正確出現在 `STATE.items`
2. **Schema 來回拷貝**：Schema 在 ②③④⑤⑥ 都各有副本，無 single source of truth
3. **Skill Router → Composer 只傳技能名**：Composer 無法知道每個技能的 score/置信度，無法做 conditional composition
4. **Dependency Resolver 的 `depends` 未傳給 Composer**：Composer 無法據此做 conditional slot injection
5. **profile.theme 只用於 Skill Router 加分**：Composer 組合 CSS 時 theme injection 是簡單的 `theme-xxx` 名稱匹配，無 fallback 邏輯
