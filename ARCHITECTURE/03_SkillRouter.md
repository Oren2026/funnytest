# ③ Skill Router（技能路由器）

**檔案：** `software/nodes/skill_router.py`

## 職責

根據 `IntentProfile` 和 `Schema` 為每個技能打分，輸出 Top-K 排序的候選技能清單。

## 輸入

```python
IntentProfile  # 來自 ①
List[Dict]     # Schema 來自 ②
```

## 輸出

```python
List[Dict]  # 排序後技能清單
[
    {"skill": "table-data", "score": 0.95},
    {"skill": "modal-form", "score": 0.88},
    {"skill": "badge-status", "score": 0.82},
    ...
]
# 取 Top-8
```

## 技能索引（SKILL_INDEX）

**純靜態字典**，手動維護，位於 `skill_router.py` 頂部。

每個技能定義：
```python
{
    "types": List[IntentType],   # 可服務的意圖類型
    "handles": List[str],         # 觸發關鍵詞
    "weight_base": float,          # 基礎權重（0.3 ~ 0.9）
}
```

### 技能列表（截至 2026-04-26）

**UI 技能**
| 技能名 | types | handles | weight_base |
|--------|-------|---------|-------------|
| layout-header | CRUD, DASHBOARD, TOOL | 導航、頂欄、nav、header | 0.8 |
| layout-dashboard | DASHBOARD | 儀表板、grid、面板、側邊欄 | 0.9 |
| table-data | CRUD, DASHBOARD | 列表、清單、表格、table | 0.9 |
| modal-form | CRUD, TOOL | 表單、新增、編輯、form | 0.8 |
| button-primary | CRUD, TOOL, DASHBOARD, GAME | 按鈕、提交、btn | 0.5 |
| button-danger | CRUD | 刪除、danger | 0.4 |
| toast-notify | CRUD, TOOL, GAME | 通知、成功、錯誤 | 0.7 |
| search-bar | CRUD, DASHBOARD | 搜尋、過濾、search | 0.8 |
| badge-status | CRUD, DASHBOARD | 狀態、badge、標籤 | 0.6 |
| confirm-dialog | CRUD | 確認、確認刪除 | 0.6 |
| sort-control | CRUD, DASHBOARD | 排序、sort | 0.7 |
| pagination | CRUD | 分頁、pagination | 0.5 |
| card-group | DASHBOARD | 卡片 | 0.7 |
| form-layout | CRUD | 表單佈局 | 0.5 |
| empty-state | CRUD | 空狀態 | 0.3 |
| sidebar | DASHBOARD | 側邊欄 | 0.6 |
| tabs | CRUD | 分頁籤 | 0.5 |
| progress-bar | TOOL, GAME | 進度條 | 0.5 |
| loading | TOOL | 載入中 | 0.3 |

**主題技能**
| 技能名 | handles | weight_base |
|--------|---------|-------------|
| theme-glass | glass、毛玻璃 | 0.5 |
| theme-modern | modern、dark、深色 | 0.6 |
| theme-brutal | brutal、粗獷 | 0.3 |
| theme-soft | soft、light、淺色、柔和 | 0.5 |

**圖表技能**
| 技能名 | types | handles | weight_base |
|--------|-------|---------|-------------|
| chart-line | DASHBOARD | 折線圖、趨勢線 | 0.8 |
| chart-bar | DASHBOARD | 柱狀圖、長條圖 | 0.8 |
| card-stat | DASHBOARD | 統計卡、數字卡 | 0.9 |

**遊戲技能**
| 技能名 | types | handles | weight_base |
|--------|-------|---------|-------------|
| game-canvas | GAME | 遊戲、canvas | 0.9 |
| game-loop | GAME | 遊戲循環、frame | 0.8 |
| score-board | GAME | 分數、積分、排行榜 | 0.8 |
| local-storage | GAME, TOOL | 存檔、localStorage | 0.6 |

**API 技能**
| 技能名 | types | handles | weight_base |
|--------|-------|---------|-------------|
| api-router | API | api、router、endpoints | 0.9 |
| auth-jwt | API | 認證、jwt、登入、auth | 0.8 |

## 控制邏輯（加權評分）

```
score = 0.0

1. IntentType 匹配：+0.3（每 skill 最多一次）
2. Keyword 匹配：+0.1 × 匹配數（entities + actions + context 全文）
3. Schema 類型覆蓋：+0.1 或 +0.05（每 matched field）
4. Base weight：+0.2 × weight_base
5. 主題加成：profile.theme 匹配 +0.3

→ 門檻：score > 0.3 才入選
→ 取 Top-8
```

### 評分公式

```
final_score = min(score, 1.0)
```

## 錯誤處理

- **無技能超過門檻**：所有 score ≤ 0.3 → 返回空列表
- **技能不在索引**：SKILL_INDEX 是完整列舉，discovery 無法自動化
- **分數相同**：Python `sorted` 穩定排序 → 維持原始定義順序

## 依賴節點

**下游：**
- ④ Dependency Resolver（接收 `[{skill, score}]` → 取 `skill` 名稱列表）

## 已知缺口

- **靜態索引**：新技能需要手動加入 `SKILL_INDEX` 字典，無法從 `skills/` 目錄自動發現
- **無學習機制**：權重是專家經驗值，無法從失敗案例中調整
- **Theme 技能只匹配名稱**：不考慮其他 theme skill 的衝突
- **無跨技能排斥邏輯**：CRUD 場景不應同時路由 `theme-brutal` + `theme-glass`，但目前無衝突檢查
