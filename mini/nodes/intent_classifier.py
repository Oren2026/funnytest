"""節點 1：Intent Classifier（意圖分類器）"""
from dataclasses import dataclass, field
from typing import List
from enum import Enum


class IntentType(Enum):
    CRUD = "CRUD"
    DASHBOARD = "DASHBOARD"
    GAME = "GAME"
    TOOL = "TOOL"
    API = "API"
    UNKNOWN = "UNKNOWN"


@dataclass
class IntentProfile:
    type: IntentType
    entities: List[str] = field(default_factory=list)
    actions: List[str] = field(default_factory=list)
    context: str = ""
    target: str = "html"
    theme: str = "modern"


# 常見實體關鍵詞（用於提取）
ENTITY_PATTERNS = {
    "任務": ["代辦", "待辦", "任務", "事項", "工作"],
    "書籍": ["書籍", "書", "圖書", "book"],
    "庫存": ["庫存", "商品", "存貨", "物料"],
    "客戶": ["客戶", "顧客", "會員", "使用者", "帳號"],
    "報表": ["報表", "報告", "統計", "數據"],
    "遊戲": ["遊戲", "game", "貪吃蛇", "俄羅斯方塊", "2048", "射擊", "flappy"],
    "計時": ["計時", "倒數", "stopwatch", "timer"],
    "計算": ["計算", "計算機", "calculator", "單位換算"],
    "密碼": ["密碼", "password", "產生器"],
    "認證": ["登入", "登出", "註冊", "auth", "jwt", "認證"],
    "資料": ["資料", "data", "資料庫", "db"],
}

# 動作關鍵詞
ACTION_PATTERNS = {
    "新增": ["新增", "建立", "創建", "create", "add", "加入"],
    "刪除": ["刪除", "移除", "remove", "delete"],
    "編輯": ["編輯", "修改", "更新", "edit", "update", "修改"],
    "查詢": ["查詢", "搜尋", "尋找", "search", "find", "過濾", "篩選"],
    "列表": ["列表", "清單", "list", "瀏覽"],
    "排序": ["排序", "sort", "由大到小", "由小到大"],
    "匯出": ["匯出", "export", "下載", "download"],
    "圖表": ["圖表", "chart", "グラフ", "視覺化"],
    "統計": ["統計", "statistic", "分析"],
    "完成": ["完成", "completed", "done", "勾選"],
    "審核": ["審核", "審批", "approve"],
    "通知": ["通知", "通知", "notification", "推播"],
}

# IntentType 關鍵詞
TYPE_PATTERNS = {
    IntentType.GAME: ["遊戲", "game", "貪吃蛇", "俄羅斯方塊", "2048", "射擊", "flappy bird", "俄罗斯方块"],
    IntentType.DASHBOARD: ["儀表板", "dashboard", "統計", "數據概覽", "後台", "analytics"],
    IntentType.API: ["rest", "api", "jwt", "認證服務", "登入認證", "後端"],
    IntentType.TOOL: ["計時器", "計算機", "單位換算", "密碼產生器", "倒數", "timer", "calculator"],
    IntentType.CRUD: ["代辦", "庫存", "客戶", "商品", "資料管理", "系統", "新增", "刪除", "編輯"],
}


def classify_intent(intent_text: str) -> IntentProfile:
    """
    將自然語法意圖分類為 IntentProfile。

    規則（優先順序）：
    1. GAME → 有游戲關鍵詞
    2. DASHBOARD → 有儀表板/統計關鍵詞
    3. API → 有 REST/API/JWT 關鍵詞
    4. TOOL → 有工具關鍵詞（計時器、計算機、密碼產生器）
    5. CRUD → 有常見 CRUD 實體關鍵詞
    6. UNKNOWN → 剩餘
    """
    text = intent_text.lower()
    entities = []
    actions = []

    # 1. 分類 IntentType
    detected_type = IntentType.UNKNOWN
    for itype, patterns in TYPE_PATTERNS.items():
        for p in patterns:
            if p.lower() in text:
                detected_type = itype
                break
        if detected_type != IntentType.UNKNOWN:
            break

    # 如果是 UNKNOWN 但有 CRUD 實體，提升為 CRUD
    if detected_type == IntentType.UNKNOWN:
        for entity, keywords in ENTITY_PATTERNS.items():
            for kw in keywords:
                if kw.lower() in text:
                    detected_type = IntentType.CRUD
                    entities.append(entity)
                    break
            if detected_type == IntentType.CRUD:
                break

    # 2. 提取實體
    for entity, keywords in ENTITY_PATTERNS.items():
        for kw in keywords:
            if kw.lower() in text and entity not in entities:
                entities.append(entity)
                break

    # 3. 提取動作
    for action, keywords in ACTION_PATTERNS.items():
        for kw in keywords:
            if kw.lower() in text and action not in actions:
                actions.append(action)
                break

    # 4. 推斷 target
    target = "html"
    if "react" in text:
        target = "react"
    elif "app" in text or "ios" in text or "android" in text:
        target = "flutter"
    elif "swiftui" in text or "swift" in text:
        target = "swift"

    # 5. 推斷 theme
    theme = "modern"
    if "glass" in text or "毛玻璃" in text:
        theme = "glass"
    elif "dark" in text or "深色" in text:
        theme = "modern"  # modern 就是 dark theme
    elif "light" in text or "淺色" in text or "soft" in text:
        theme = "soft"
    elif "brutal" in text or "粗獷" in text:
        theme = "brutal"

    return IntentProfile(
        type=detected_type,
        entities=entities,
        actions=actions,
        context=intent_text,
        target=target,
        theme=theme,
    )
