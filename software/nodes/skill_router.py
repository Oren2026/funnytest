"""節點 3：Skill Router（技能路由器）"""
from typing import List, Dict
from .intent_classifier import IntentProfile, IntentType


# 技能索引：每個技能的元資料
# 說明：
#   - types: 支援的意圖類型（由 IntentClassifier 輸出）
#   - handles: 關鍵詞觸發（entities + actions + context 匹配）
#   - weight_base: 基礎權重（0.0-1.0）
#
# 注意：layout-page 和 theme skill 由 Composer 直接依賴，不透過 Router 選擇
SKILL_INDEX = {
    # === Content / UI 技能（Router 選擇）===
    "layout-header": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD, IntentType.TOOL],
        "handles": ["導航", "頂欄", "nav", "header", "頁首"],
        "weight_base": 0.8,
    },
    "table-data": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["列表", "清單", "表格", "table", "grid", "資料顯示"],
        "weight_base": 0.9,
    },
    "modal-form": {
        "types": [IntentType.CRUD, IntentType.TOOL],
        "handles": ["表單", "新增", "編輯", "form", "新增表單", "編輯表單"],
        "weight_base": 0.8,
    },
    "search-bar": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["搜尋", "搜尋框", "過濾", "search", "篩選"],
        "weight_base": 0.8,
    },
    "badge-status": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["狀態", "badge", "標籤", "標記"],
        "weight_base": 0.6,
    },
    "confirm-dialog": {
        "types": [IntentType.CRUD],
        "handles": ["確認", "確認刪除", "confirm"],
        "weight_base": 0.6,
    },
    "pagination": {
        "types": [IntentType.CRUD],
        "handles": ["分頁", "pagination", "頁碼"],
        "weight_base": 0.5,
    },
    "alert-banner": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD],
        "handles": ["提示", "警告", "alert", "橫幅", "訊息提示"],
        "weight_base": 0.5,
    },
    "empty-state": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["空", "空狀態", "empty", "尚無", "沒東西"],
        "weight_base": 0.4,
    },
    "card-group": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["卡片", "card", "項目顯示"],
        "weight_base": 0.6,
    },
    "form-layout": {
        "types": [IntentType.CRUD, IntentType.TOOL],
        "handles": ["表單", "表單佈局", "form layout", "欄位"],
        "weight_base": 0.5,
    },
    "sidebar": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["側邊", "sidebar", "側欄", "導航側邊"],
        "weight_base": 0.5,
    },
    "tabs": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["標籤", "tabs", "分頁切換", "tab"],
        "weight_base": 0.5,
    },
    "loading": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD, IntentType.GAME],
        "handles": ["載入", "loading", "讀取中", "spinner"],
        "weight_base": 0.3,
    },
    "progress-bar": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD, IntentType.GAME],
        "handles": ["進度", "progress", "進度條", "loading bar"],
        "weight_base": 0.4,
    },
    # === 主題技能（Router 選擇 + Composer 直接注入）===
    "theme-glass": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD],
        "handles": ["glass", "毛玻璃", "透明", "glassy"],
        "weight_base": 0.5,
    },
    "theme-modern": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD],
        "handles": ["modern", "dark", "深色", "時尚"],
        "weight_base": 0.6,
    },
    "theme-brutal": {
        "types": [IntentType.CRUD, IntentType.TOOL],
        "handles": ["brutal", "粗獷", "原始"],
        "weight_base": 0.3,
    },
    "theme-soft": {
        "types": [IntentType.CRUD, IntentType.TOOL],
        "handles": ["soft", "light", "淺色", "柔和", "亮色"],
        "weight_base": 0.5,
    },
    # === 程式碼技能（Router 選擇）===
    "array-base": {
        "types": [IntentType.TOOL],
        "handles": ["array", "陣列", "陣列操作", "陣列處理"],
        "weight_base": 0.6,
    },
    "bubble_sort": {
        "types": [IntentType.TOOL],
        "handles": ["排序", "sort", "由大到小", "由小到大", "bubble"],
        "weight_base": 0.7,
    },
    "insertion_sort": {
        "types": [IntentType.TOOL],
        "handles": ["排序", "sort", "insertion"],
        "weight_base": 0.7,
    },
    "quick_sort": {
        "types": [IntentType.TOOL],
        "handles": ["排序", "sort", "quick", "快速排序"],
        "weight_base": 0.7,
    },
    "array-sort": {
        "types": [IntentType.TOOL],
        "handles": ["排序", "sort"],
        "weight_base": 0.5,
    },
    "linear_search": {
        "types": [IntentType.TOOL],
        "handles": ["搜尋", "search", "查找", "linear"],
        "weight_base": 0.7,
    },
    "binary_search": {
        "types": [IntentType.TOOL],
        "handles": ["搜尋", "search", "二分", "binary"],
        "weight_base": 0.7,
    },
    "comparison": {
        "types": [IntentType.TOOL],
        "handles": ["比較", "comparison", "比大小", "swap", "交換"],
        "weight_base": 0.5,
    },
    "printf-basic": {
        "types": [IntentType.TOOL],
        "handles": ["printf", "輸出", "列印", "print"],
        "weight_base": 0.4,
    },
    "signal-handler": {
        "types": [IntentType.TOOL],
        "handles": ["signal", "訊號", "中斷", "handler", "sig"],
        "weight_base": 0.3,
    },
    "loop-pattern": {
        "types": [IntentType.TOOL],
        "handles": ["loop", "迴圈", "循環", "iterate"],
        "weight_base": 0.4,
    },
    "memory-static": {
        "types": [IntentType.TOOL],
        "handles": ["memory", "記憶體", "static", "靜態", "buffer"],
        "weight_base": 0.3,
    },
    "linked-list-static": {
        "types": [IntentType.TOOL],
        "handles": ["linked list", "鏈結", "鏈表", "linked-list"],
        "weight_base": 0.5,
    },
    "stack-static": {
        "types": [IntentType.TOOL],
        "handles": ["stack", "堆疊", "stack-static"],
        "weight_base": 0.5,
    },
    "queue-static": {
        "types": [IntentType.TOOL],
        "handles": ["queue", "佇列", "queue-static", "排隊"],
        "weight_base": 0.5,
    },
    "daemon-loop": {
        "types": [IntentType.TOOL],
        "handles": ["daemon", "背景", "守護", "loop"],
        "weight_base": 0.3,
    },
    "dynamic-allocation": {
        "types": [IntentType.TOOL],
        "handles": ["malloc", "alloc", "記憶體配置", "dynamic"],
        "weight_base": 0.3,
    },
    "timer-periodic": {
        "types": [IntentType.TOOL],
        "handles": ["timer", "計時", "定時", "periodic", "timer-periodic"],
        "weight_base": 0.3,
    },
    "printf": {
        "types": [IntentType.TOOL],
        "handles": ["printf", "c_lang", "c語言"],
        "weight_base": 0.2,
    },
    # === 內建按鈕技能 ===
    "button-primary": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD, IntentType.GAME],
        "handles": ["按鈕", "提交", "送出", "新增"],
        "weight_base": 0.5,
    },
    "button-danger": {
        "types": [IntentType.CRUD],
        "handles": ["刪除", "danger", "危險按鈕"],
        "weight_base": 0.4,
    },
    "toast-notify": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.GAME],
        "handles": ["通知", "成功", "錯誤", "toast", "訊息"],
        "weight_base": 0.7,
    },
}


def route_skills(profile: IntentProfile, schema: List[Dict]) -> List[Dict]:
    """
    根據 IntentProfile 和 Schema 為每個技能打分，輸出 Top-K 技能列表。

    打分邏輯：
    - IntentType 匹配：+0.3
    - Keyword 匹配：每個匹配關鍵詞 +0.1
    - Schema 類型覆蓋：每個 field type 有對應技能 +0.2
    - 基礎權重：skill 的 weight_base
    """
    scores = {}

    for skill_name, meta in SKILL_INDEX.items():
        score = 0.0

        # 1. IntentType 匹配
        if profile.type in meta["types"]:
            score += 0.3

        # 2. Keyword 匹配（entities + actions + context）
        text = (profile.context + " " + " ".join(profile.entities) + " " + " ".join(profile.actions)).lower()
        for kw in meta["handles"]:
            if kw.lower() in text:
                score += 0.1

        # 3. 基礎權重
        score += meta["weight_base"] * 0.2

        # 4. Schema 類型覆蓋（額外加分）
        for field in schema:
            field_type = field.get("type", "text")
            if field_type in ["action", "checkbox"]:
                if skill_name in ["table-data", "button-danger", "confirm-dialog"]:
                    score += 0.1
            elif field_type in ["badge"]:
                if skill_name in ["badge-status", "table-data"]:
                    score += 0.1
            elif field_type in ["date", "text"]:
                if skill_name in ["modal-form", "table-data"]:
                    score += 0.05

        # 5. 主題技能：根據 profile.theme 提升對應主題分數
        if "theme-" in skill_name:
            theme_pref = profile.theme
            if theme_pref in skill_name:
                score += 0.3
            elif skill_name == "theme-modern" and profile.theme in ["modern", "dark"]:
                score += 0.1

        # 最低門檻
        if score > 0.3:
            scores[skill_name] = min(score, 1.0)

    # 排序並取 Top-K
    sorted_skills = sorted(scores.items(), key=lambda x: x[1], reverse=True)
    return [{"skill": name, "score": score} for name, score in sorted_skills[:8]]
