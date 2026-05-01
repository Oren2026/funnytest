"""節點 3：Skill Router（技能路由器）"""
from typing import List, Dict
from .intent_classifier import IntentProfile, IntentType


# 技能索引：每個技能的元資料
SKILL_INDEX = {
    # UI 技能
    "layout-header": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD, IntentType.TOOL],
        "handles": ["導航", "頂欄", "nav", "header", "頁首"],
        "weight_base": 0.8,
    },
    "layout-dashboard": {
        "types": [IntentType.DASHBOARD],
        "handles": ["儀表板", "grid", "面板", "側邊欄", "dashboard"],
        "weight_base": 0.9,
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
    "button-primary": {
        "types": [IntentType.CRUD, IntentType.TOOL, IntentType.DASHBOARD, IntentType.GAME],
        "handles": ["按鈕", "提交", "按鈕", "送出"],
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
    "sort-control": {
        "types": [IntentType.CRUD, IntentType.DASHBOARD],
        "handles": ["排序", "sort", "由大到小"],
        "weight_base": 0.7,
    },
    "pagination": {
        "types": [IntentType.CRUD],
        "handles": ["分頁", "pagination", "頁碼"],
        "weight_base": 0.5,
    },
    # 主題技能
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
    # 圖表技能
    "chart-line": {
        "types": [IntentType.DASHBOARD],
        "handles": ["折線圖", "趨勢", "趨勢線", "line chart"],
        "weight_base": 0.8,
    },
    "chart-bar": {
        "types": [IntentType.DASHBOARD],
        "handles": ["柱狀圖", "長條圖", "bar chart", "統計"],
        "weight_base": 0.8,
    },
    "card-stat": {
        "types": [IntentType.DASHBOARD],
        "handles": ["統計卡", "卡片", "數字卡", "stat card"],
        "weight_base": 0.9,
    },
    # 遊戲技能
    "game-canvas": {
        "types": [IntentType.GAME],
        "handles": ["遊戲", "canvas", "canvas遊戲"],
        "weight_base": 0.9,
    },
    "game-loop": {
        "types": [IntentType.GAME],
        "handles": ["遊戲", "loop", "遊戲循環", "frame"],
        "weight_base": 0.8,
    },
    "score-board": {
        "types": [IntentType.GAME],
        "handles": ["分數", "積分", "排行榜", "score"],
        "weight_base": 0.8,
    },
    "local-storage": {
        "types": [IntentType.GAME, IntentType.TOOL],
        "handles": ["存檔", "儲存", "local", "localStorage"],
        "weight_base": 0.6,
    },
    # API 技能
    "api-router": {
        "types": [IntentType.API],
        "handles": ["api", "router", "端點", "endpoints"],
        "weight_base": 0.9,
    },
    "auth-jwt": {
        "types": [IntentType.API],
        "handles": ["認證", "jwt", "登入", "auth", "權杖"],
        "weight_base": 0.8,
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
