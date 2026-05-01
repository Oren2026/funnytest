# ⑤ Composer（組合器）— Spec-aware Composition

**檔案：** `software/nodes/composer.py`

## 核心願景

讓每個 Skill 不是「程式碼區塊」，而是「有語意合約的構件」。
Composer 在組裝時，能根據合約判斷「什麼情境用這個 Skill」、「誰該負責哪個 slot」，而不是靠技能名字的巧合拼接。

---

## 職責

將經過拓撲排序的技能列表、推斷出的 Schema、以及 IntentProfile 組合拼接成最終輸出。
負責：技能 Spec 解析、slot → skill 動態解析、程式碼區塊載入、slot injection、schema-driven 動態內容生成。

---

## 輸入

```python
ordered_skills: List[Dict]  # 來自 ④ Dependency Resolver
# [{"skill": "layout-header", "depends": []}, ...]

schema: List[Dict]           # 來自 ② Schema Inferrer

profile: IntentProfile       # 來自 ① Intent Classifier

output_type: str            # "html" | "react"（目前主要支援 html）
```

---

## 輸出

```python
{
    "code": str,             # 最終 HTML/CSS/JS 字串
    "warnings": List[str],    # Spec-aware fallback 警告（哪些 slot 還沒有 Spec 宣告）
    "metadata": {
        "skills_used": List[str],  # 實際使用的技能名
        "schema": List[Dict],      # 原始 schema 副本
        "theme": str,              # 使用的主題
    }
}
```

---

## Spec 格式（技能語意合約）

每個 `.skill` 檔案現在包含五個語意區塊：

```markdown
## Contract          # 這個 Skill 對世界的承諾
## Dependencies      # 依賴哪些 Skill、可選依賴、排斥
## Slots             # 提供哪些 slot、消耗哪些 slot
## Boundaries        # 系統邊界、狀態邊界
## Examples          # 具體範例
```

### Contract 區塊（核心）

```markdown
## Contract
- **語義承諾**：一句話說清楚這個 Skill 做什麼
- **輸入資料格式**：JSON Schema 或自然語言描述
- **輸出語義**：這個 Skill 的輸出對調用者代表什麼
- **操作邊界**：
  - ✅ 做：根據 items 動態渲染資料列、支援排序切換、顯示空狀態
  - ❌ 不做：新增/刪除/編輯背後的資料邏輯、不處理 API 呼叫
- **失敗信號**：
  - `items` 不是陣列 → 渲染 empty-state slot
  - `items` 長度為 0 → 顯示「尚無庫存資料」
```

### Dependencies 區塊

```markdown
## Dependencies
- **依賴**：`badge-status`（庫存狀態顯示）
- **可選依賴**：`button-primary`（編輯按鈕）、`button-danger`（刪除按鈕）
- **排斥**：無
```

### Slots 區塊

```markdown
## Slots
- **slot:thead**：欄位標題列，由 Schema Inferrer 注入
- **slot:tbody**：動態資料行，由 Composer 注入
- **slot:empty-state**：當 items 為空時顯示的提示
- **slot:filter**：搜尋/過濾控制區，由 search-bar 注入
```

### Boundaries 區塊

```markdown
## Boundaries
- **系統邊界**：Presentation Layer（只處理 UI 渲染，不管資料邏輯）
- **狀態邊界**：Stateless（不保有內部狀態）
```

---

## 新增類別：SkillSpec 與 SkillRegistry

### SkillSpec dataclass

```python
@dataclass
class SkillSpec:
    name: str
    semantic_promise: str     # Contract: 語義承諾
    input_format: str         # Contract: 輸入格式描述
    output_semantic: str      # Contract: 輸出語義
    does: List[str]           # Contract: ✅ 做的清單
    does_not: List[str]       # Contract: ❌ 不做的清單
    failure_signals: List[str] # Contract: 失敗信號
    dependencies: List[str]   # Dependencies: 依賴
    optional_deps: List[str]  # Dependencies: 可選依賴
    excludes: List[str]       # Dependencies: 排斥
    slots_provides: List[str] # Slots: 提供這些 slot
    slots_consumes: List[str] # Slots: 需要這些 slot
    boundary_layer: str       # Boundaries: 系統邊界
    is_stateful: bool         # Boundaries: 是否保有狀態
    html: str = ""            # 原始 [html] 區塊
    style: str = ""           # 原始 [style] 區塊
    react: str = ""           # 原始 [react] 區塊
```

### SkillRegistry（單例）

```python
class SkillRegistry:
    _instance: Optional["SkillRegistry"] = None

    def get(cls) -> "SkillRegistry":
        # 延遲初始化，掃描 skills/ 目錄建立反向索引

    def find_skill_for_slot(self, slot_name: str, context: str = "") -> Optional[str]:
        # 根據 slot 名稱找「誰能提供這個 slot」
        # 策略：優先找名字包含 slot_name 的 skill
        # 預留：進階可根據 Contract 語意匹配

    def get_spec(self, skill_name: str) -> Optional[SkillSpec]:
        # 取得某個 Skill 的完整 Spec

    def get_slot_providers(self, slot_name: str) -> List[str]:
        # 取得所有聲明提供某個 slot 的 Skill 清單
```

---

## Spec-aware Slot 解析流程

### 舊流程（硬編碼）

```
slot:header → 一定是 layout-header
slot:content → 一定是 table-data
（技能名字是固定的，無法動態替換）
```

### 新流程（Spec-aware）

```
slot:header
  → SkillRegistry.find_skill_for_slot("header")
  → 如果有 Skill 聲明「我能提供 slot:header」→ 使用該 Skill
  → 如果沒有（尚未轉 Spec）→ fallback 到 "layout-header" + 發 warning
```

```python
slot_map = [
    ("header",   "layout-header"),
    ("search",   "search-bar"),
    ("content",  "table-data"),
    ("modal",    "modal-form"),
    ("confirm",  "confirm-dialog"),
    ("toast",    "toast-notify"),
]

loaded_slots = {}
for slot_name, fallback_skill in slot_map:
    skill_name = registry.find_skill_for_slot(slot_name)
    if skill_name is None:
        skill_name = fallback_skill  # 向後兼容 + warning
        warnings.append(f"[Spec-aware] slot '{slot_name}': no Spec declaration, fallback to '{skill_name}'")
    else:
        if skill_name != fallback_skill:
            warnings.append(f"[Spec-aware] slot '{slot_name}': resolved to '{skill_name}' (Spec override)")
    loaded_slots[slot_name] = load_skill_blocks(skill_name, "html")
```

**當前 L1 測試 Warning 輸出（2026-04-30）：**

```
⚠️  slot 'header': no Spec declaration found, falling back to 'layout-header'
⚠️  slot 'search': no Spec declaration found, falling back to 'search-bar'
⚠️  slot 'content': no Spec declaration found, falling back to 'table-data'  ← table-data 已有 Spec
⚠️  slot 'modal': no Spec declaration found, falling back to 'modal-form'
⚠️  slot 'confirm': no Spec declaration found, falling back to 'confirm-dialog'
⚠️  slot 'toast': no Spec declaration found, falling back to 'toast-notify'
```

---

## Slot 名稱對照（新）

| Slot 名 | 注入內容 | Spec 來源（目標） | Fallback（現狀） |
|---------|---------|----------------|----------------|
| `header` | 頁面頂欄 HTML | `layout-header` | `layout-header` |
| `search` | 搜尋框 HTML | `search-bar` | `search-bar` |
| `content` | 表格（含 schema-driven thead） | `table-data` ✅ pilot | `table-data` |
| `modal` | 表單 modal | `modal-form` | `modal-form` |
| `confirm` | 刪除確認 dialog | `confirm-dialog` | `confirm-dialog` |
| `toast` | toast 容器 | `toast-notify` | `toast-notify` |
| `actions` | 新增按鈕 | 動態生成 | 動態生成 |

---

## 拼接流程（HTML 輸出）

```
1. 建立 SkillRegistry（延遲初始化）

2. Slot → Skill 解析
   對每個 slot_name：嘗試 registry.find_skill_for_slot()
   → 有 Spec 宣告 → 使用該 Skill
   → 沒有 → fallback + warning

3. 載入所有技能 CSS（style 區塊）
   → 主題 CSS（theme-*）單獨存放，最後注入

4. Schema-driven 動態生成：
   a. _build_form_from_schema() → 表單 HTML + JS 函式
   b. _build_dynamic_table_html() → thead HTML + render switch-case JS

5. Slot injection 順序（layout-page）：
   ├─ slot:header      → resolved skill（layout-header 或 Spec 宣告的）
   ├─ slot:search      → resolved skill
   ├─ slot:content     → resolved skill（含 schema-driven thead）
   ├─ slot:modal       → resolved skill（含 schema-driven form）
   ├─ slot:confirm     → resolved skill
   ├─ slot:toast       → resolved skill
   └─ slot:actions     → 動態生成（新增按鈕）

6. 頁面標題：entity_name + context[:40]

7. 組合：
   <!DOCTYPE html>
   <html>
     <head> <style> [CSS: 所有技能 + 主題 + 通用樣式] </style> </head>
     <body>
       [layout-page (含所有 slot injection)]
       <script> [STATE + 所有 JS 函式] </script>
     </body>
   </html>
```

---

## 向後兼容設計

- `load_skill_blocks(skill_name, section)` 完全維持舊介面
- `load_skill_spec(skill_name)` 回傳 `Optional[SkillSpec]`，舊格式 Skill 回傳 `None`
- SkillRegistry 對沒有 Spec 宣告的 Skill 不會造成任何影響
- 當所有 Skill 都轉成 Spec 格式後，warning 訊息會完全消失

---

## 進階擴展點（尚未實作）

1. **語意匹配**：`find_skill_for_slot()` 目前只用「名字包含」策略，未來可根據 `Contract.does` / `Contract.does_not` 做語意匹配
2. **失敗信號觸發**：當 QA Checker 發現問題時，根據 `Contract.failure_signals` 定位是哪個 Skill 的問題
3. **Boundary 驗證**：組裝時檢查「誰说自己 Stateless，卻被發現有狀態操作」
4. **Example 驅動測試**：用 `Contract.Examples` 做 Spec-level 的單元測試

---

## 已知缺口

- **React 輸出**（`_compose_react`）：存在輪廓但輸出不完整，無驗證
- **大多數 Skill 尚未轉 Spec**：目前只有 `table-data` 是 Spec 格式，其餘 40 個仍是舊格式
- **Spec 格式無驗證機制**：如果某個 `.skill` 檔案的 Spec 區塊格式錯誤，parser 會靜默忽略（不拋 exception）
- **localStorage key**：`evcompiler_${page_title}` 可能衝突（目前只取前 20 字）
- **Schema 的 `options`**：badge 類型用字串比對（`高/中/低`），若 Schema options 不同則 badge 渲染錯誤
- **固定 `inventory-form` id**：所有表單都用 `id="inventory-form"`，多表單頁面會衝突
- **table 結構硬編碼**：tbody 用 `.inventory-table tbody` 選擇器，CSS class 名稱是 warehouse 殘留
