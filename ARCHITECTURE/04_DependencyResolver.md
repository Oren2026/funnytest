# ④ Dependency Resolver（依賴解析器）

**檔案：** `software/nodes/dependency_resolver.py`

## 職責

接收技能名稱列表，讀取每個技能的 `# depends:` 宣告，用 Kahn 拓撲排序算出正確的組合順序，並遞迴擴展依賴圖。

## 輸入

```python
List[str]  # 技能名稱列表，來自 ③ Skill Router
# 例：["table-data", "modal-form", "badge-status", "search-bar"]

Path  # skills 目錄路徑（software/skills/）
```

## 輸出

```python
List[Dict]  # 拓撲排序後的技能列表
[
    {"skill": "toast-notify", "depends": []},
    {"skill": "layout-header", "depends": []},
    {"skill": "button-primary", "depends": []},
    {"skill": "badge-status", "depends": []},
    {"skill": "modal-form", "depends": ["button-primary"]},
    {"skill": "table-data", "depends": []},
    {"skill": "search-bar", "depends": []},
]
```

## 控制邏輯

### 1. 讀取技能依賴

每個 `.skill` 檔案頂部宣告：
```
# skill: modal-form
# depends: button-primary, confirm-dialog
# prohibit: none
```

解析正則：`r'^#\s*depends:\s*(.+)$'`

`# depends: none` → 回傳空列表

### 2. 遞迴擴展依賴圖

```python
def expand_deps(skill, visited):
    deps = load_skill_depends(skill)
    for dep in deps:
        if dep not in visited:
            expand_deps(dep, visited)
```

所有依賴技能都會加入圖中，包括技能庫中未在 Top-8 中入選的間接依賴。

### 3. Kahn 拓撲排序

```
圖：adjacency list {skill: [dep1, dep2]}
入度表：in_degree[skill] = 被依賴次數

佇列：所有入度=0 的節點
while 佇列非空:
    取出節點，加入 ordered
    該節點的每個鄰居 in_degree--
    若鄰居 in_degree==0 → 加入佇列
```

### 4. 循環檢測

```python
if len(ordered) != len(all_nodes):
    remaining = set(all_nodes) - set(ordered)
    raise ValueError(f"循環依賴檢測：{remaining}")
```

## 錯誤處理

| 錯誤 | 原因 | 處理 |
|------|------|------|
| `ValueError: 循環依賴檢測` | A→B→C→A | 印出循環節點，程式終止 |
| 技能檔案不存在 | `# depends: xxx` 但 `xxx.skill` 不存在 | `load_skill_depends` 回傳 `[]`（寬容） |
| 間接依賴未入選 | 候選技能擴展出未在原始列表的依賴技能 | 保留在圖中但最終 filter |

## 依賴節點

**下游：**
- ⑤ Composer（接收完整 `{skill, depends}` 列表）

## 技能目錄搜尋順序

技能檔案搜尋以下子目錄（按順序，找到第一個就停）：
```
skills/ui/ → skills/styles/ → skills/core/ → skills/algorithms/ → skills/structures/ → skills/system/ → skills/behaviors/
```

## 已知缺口

- **寬容降級**：技能依賴不存在的檔案 → 默默回傳 `[]`，無 warning
- **最終 filter 複雜**：line 92-110 的邏輯試圖只保留原始技能 + 直接依賴，但過濾順序不一定穩定
- **無 prohibit 處理**：`# prohibit:` 欄位存在但 `resolve_dependencies` 完全忽略它
- **無版本管理**：技能 A 的不同版本無法共存
