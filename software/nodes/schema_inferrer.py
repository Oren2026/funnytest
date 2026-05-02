"""節點 2：Schema Inferrer（資料結構推斷器）"""
from typing import List, Dict
from .intent_classifier import IntentProfile, IntentType


# ============================================================================
# 實體欄位目錄（ENTITY → 標準 Schema）
# 維護方式：當新實體類型被驗證有效時，在此新增條目
# 來源：從對應技能的 Spec Contract section 萃取出 canonical field definitions
# ============================================================================
ENTITY_FIELD_DEFINITIONS: Dict[str, List[Dict]] = {
    # 代辦事項 / 任務
    "任務": [
        {"name": "title", "label": "任務名稱", "type": "text", "required": True, "editable": True, "placeholder": "請輸入任務"},
        {"name": "description", "label": "描述", "type": "text", "required": False, "editable": True},
        {"name": "priority", "label": "優先權", "type": "badge", "required": False, "editable": True,
         "options": ["高", "中", "低"], "default": "中"},
        {"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True,
         "options": ["待處理", "進行中", "已完成"], "default": "待處理"},
        {"name": "dueDate", "label": "截止日期", "type": "date", "required": False, "editable": True},
    ],
    "待辦": [
        {"name": "title", "label": "待辦事項", "type": "text", "required": True, "editable": True, "placeholder": "請輸入待辦"},
        {"name": "description", "label": "備註", "type": "text", "required": False, "editable": True},
        {"name": "priority", "label": "優先權", "type": "badge", "required": False, "editable": True,
         "options": ["高", "中", "低"], "default": "中"},
        {"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True,
         "options": ["待處理", "進行中", "已完成"], "default": "待處理"},
        {"name": "dueDate", "label": "截止日期", "type": "date", "required": False, "editable": True},
    ],
    # 庫存
    "庫存": [
        {"name": "title", "label": "商品名稱", "type": "text", "required": True, "editable": True, "placeholder": "請輸入商品名稱"},
        {"name": "category", "label": "分類", "type": "text", "required": False, "editable": True},
        {"name": "quantity", "label": "庫存數量", "type": "text", "required": False, "editable": True},
        {"name": "price", "label": "單價", "type": "text", "required": False, "editable": True},
        {"name": "stockStatus", "label": "庫存狀態", "type": "badge", "required": False, "editable": True,
         "options": ["有貨", "缺貨", "補貨中"], "default": "有貨"},
        {"name": "description", "label": "說明", "type": "text", "required": False, "editable": True},
    ],
    # 客戶
    "客戶": [
        {"name": "title", "label": "客戶名稱", "type": "text", "required": True, "editable": True, "placeholder": "請輸入客戶名稱"},
        {"name": "contact", "label": "聯絡方式", "type": "text", "required": False, "editable": True},
        {"name": "email", "label": "Email", "type": "text", "required": False, "editable": True},
        {"name": "phone", "label": "電話", "type": "text", "required": False, "editable": True},
        {"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True,
         "options": ["正常", "待聯繫", "已成交"], "default": "正常"},
        {"name": "description", "label": "備註", "type": "text", "required": False, "editable": True},
    ],
    # 書籍
    "書籍": [
        {"name": "title", "label": "書名", "type": "text", "required": True, "editable": True, "placeholder": "請輸入書名"},
        {"name": "author", "label": "作者", "type": "text", "required": False, "editable": True},
        {"name": "category", "label": "分類", "type": "text", "required": False, "editable": True},
        {"name": "status", "label": "借閱狀態", "type": "badge", "required": False, "editable": True,
         "options": ["可借", "已借出", "預約中"], "default": "可借"},
        {"name": "description", "label": "簡介", "type": "text", "required": False, "editable": True},
    ],
    # 產品（通用）
    "產品": [
        {"name": "title", "label": "產品名稱", "type": "text", "required": True, "editable": True, "placeholder": "請輸入產品名稱"},
        {"name": "category", "label": "類型", "type": "text", "required": False, "editable": True},
        {"name": "price", "label": "價格", "type": "text", "required": False, "editable": True},
        {"name": "stockStatus", "label": "庫存狀態", "type": "badge", "required": False, "editable": True,
         "options": ["有貨", "缺貨", "補貨中"], "default": "有貨"},
        {"name": "description", "label": "說明", "type": "text", "required": False, "editable": True},
    ],
    # 報表
    "報表": [
        {"name": "title", "label": "報表名稱", "type": "text", "required": True, "editable": True, "placeholder": "請輸入報表名稱"},
        {"name": "date", "label": "報表日期", "type": "date", "required": False, "editable": True},
        {"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True,
         "options": ["草稿", "已完成", "已送出"], "default": "草稿"},
        {"name": "description", "label": "摘要", "type": "text", "required": False, "editable": True},
    ],
}


# 通用 non-editable 欄位（所有 entity 皆附加）
_NON_EDITABLE_FIELDS = [
    {"name": "id", "label": "ID", "type": "text", "required": False, "editable": False},
    {"name": "createdAt", "label": "建立時間", "type": "date", "required": False, "editable": False},
    {"name": "updatedAt", "label": "更新時間", "type": "date", "required": False, "editable": False},
    {"name": "actions", "label": "操作", "type": "action", "required": False, "editable": False},
]


def infer_schema(profile: IntentProfile) -> List[Dict]:
    """
    根據 IntentProfile 推斷資料結構。

    邏輯（優先順序）：
    1. 若 entity 有標準定義（ENTITY_FIELD_DEFINITIONS）→ 直接使用
    2. 若 context 中有明確欄位格式「（XX、XX、XX）」→ 解析並推斷類型
    3. 若皆無 → fallback 為基礎 title + description
    4. 所有 schema 末尾附加 _NON_EDITABLE_FIELDS
    """
    intent_type = profile.type

    if intent_type == IntentType.CRUD:
        return _infer_crud_schema(profile)
    elif intent_type == IntentType.DASHBOARD:
        return _infer_dashboard_schema(profile)
    elif intent_type == IntentType.GAME:
        return _infer_game_schema(profile)
    elif intent_type == IntentType.TOOL:
        return _infer_tool_schema(profile)
    elif intent_type == IntentType.API:
        return _infer_api_schema(profile)
    else:
        return _infer_crud_schema(profile)


def _infer_crud_schema(profile: IntentProfile) -> List[Dict]:
    """CRUD schema：優先使用 ENTITY_FIELD_DEFINITIONS，解析明確欄位為輔"""
    import re

    entity_name = profile.entities[0] if profile.entities else "項目"
    context = profile.context

    # 1. 從 entity 目錄查找
    if entity_name in ENTITY_FIELD_DEFINITIONS:
        editable = [dict(f) for f in ENTITY_FIELD_DEFINITIONS[entity_name]]
    else:
        editable = []

    # 2. 解析 context 中的明確欄位（格式：「（標題、分類、庫存）」）
    explicit_fields = []
    field_pattern = re.search(r'（([^）]+)）', context)
    if field_pattern:
        explicit_fields = [f.strip() for f in field_pattern.group(1).split('、')]

    if explicit_fields:
        # 第一個當 title（若尚未定義）
        if not editable:
            primary = explicit_fields[0]
            editable.append({"name": "title", "label": primary, "type": "text",
                             "required": True, "editable": True})
        # 其餘欄位根據名稱推斷類型
        existing_names = {f["name"] for f in editable}
        for field in explicit_fields[1:]:
            f_lower = field.lower()
            fname = _infer_field_name(field, f_lower)
            if fname in existing_names:
                continue
            ftype, foptions, fdefault = _infer_field_type(f_lower)
            editable.append({
                "name": fname, "label": field, "type": ftype,
                "required": False, "editable": True,
                **( {"options": foptions, "default": fdefault} if foptions else {} )
            })
    elif not editable:
        # 3. Fallback：無 entity 定義也無明確欄位
        editable.append({"name": "title", "label": f"{entity_name}名稱", "type": "text",
                         "required": True, "editable": True, "placeholder": f"請輸入{entity_name}"})
        if any(kw in context.lower() for kw in ['描述', '說明', 'description']):
            editable.append({"name": "description", "label": "描述", "type": "text",
                             "required": False, "editable": True})

    return editable + [dict(f) for f in _NON_EDITABLE_FIELDS]


def _infer_field_name(field: str, f_lower: str) -> str:
    """從欄位名推斷內部欄位名稱"""
    if any(kw in f_lower for kw in ['名稱', '標題', 'title', 'name']):
        return "title"
    if any(kw in f_lower for kw in ['分類', '類型', 'category', 'type']):
        return "category"
    if any(kw in f_lower for kw in ['作者', '建立人', '負責人']):
        return "author"
    if any(kw in f_lower for kw in ['數量', '庫存']):
        return "quantity"
    if any(kw in f_lower for kw in ['價格', '單價', '成本']):
        return "price"
    if any(kw in f_lower for kw in ['電話', 'phone']):
        return "phone"
    if any(kw in f_lower for kw in ['email', '信箱', 'mail']):
        return "email"
    if any(kw in f_lower for kw in ['庫存狀態', '水位']):
        return "stockStatus"
    if any(kw in f_lower for kw in ['優先', 'priority']):
        return "priority"
    if any(kw in f_lower for kw in ['狀態', 'status']):
        return "status"
    if any(kw in f_lower for kw in ['截止', '期限', 'due']):
        return "dueDate"
    if any(kw in f_lower for kw in ['描述', '說明', 'description', '備註']):
        return "description"
    return "description"


def _infer_field_type(f_lower: str):
    """從欄位名推斷類型，返回 (type, options, default)"""
    if any(kw in f_lower for kw in ['庫存狀態', '水位']):
        return "badge", ["有貨", "缺貨", "補貨中"], "有貨"
    if any(kw in f_lower for kw in ['優先', 'priority']):
        return "badge", ["高", "中", "低"], "中"
    if any(kw in f_lower for kw in ['狀態', 'status']):
        return "badge", ["待處理", "進行中", "已完成"], "待處理"
    if any(kw in f_lower for kw in ['截止', '期限', '日期', 'date', 'due']):
        return "date", None, None
    if any(kw in f_lower for kw in ['數量', '價格', '庫存', 'quantity', 'price']):
        return "text", None, None
    return "text", None, None


def _infer_dashboard_schema(profile: IntentProfile) -> List[Dict]:
    """Dashboard schema：統計卡 + 圖表"""
    return [
        {"name": "metric", "label": "指標名稱", "type": "text", "required": True},
        {"name": "value", "label": "數值", "type": "text", "required": True},
        {"name": "change", "label": "變化", "type": "text", "required": False},
        {"name": "trend", "label": "趨勢", "type": "badge",
         "required": False, "options": ["up", "down", "stable"]},
        {"name": "chartType", "label": "圖表類型", "type": "badge",
         "required": False, "options": ["折線圖", "柱狀圖", "圓餅圖"]},
        {"name": "actions", "label": "操作", "type": "action", "required": False, "editable": False},
    ]


def _infer_game_schema(profile: IntentProfile) -> List[Dict]:
    """Game schema：分數、等級、遊戲狀態"""
    return [
        {"name": "score", "label": "分數", "type": "text", "required": False},
        {"name": "level", "label": "等級", "type": "text", "required": False},
        {"name": "lives", "label": "生命", "type": "text", "required": False},
        {"name": "gameState", "label": "遊戲狀態", "type": "badge",
         "options": ["進行中", "暫停", "結束"]},
        {"name": "actions", "label": "操作", "type": "action"},
    ]


def _infer_tool_schema(profile: IntentProfile) -> List[Dict]:
    """Tool schema：input + result"""
    schema = [
        {"name": "input", "label": "輸入", "type": "text", "required": True},
        {"name": "result", "label": "結果", "type": "text", "required": False},
        {"name": "actions", "label": "操作", "type": "action"},
    ]

    if "計時" in profile.entities or "倒數" in profile.entities:
        schema = [
            {"name": "time", "label": "時間（秒）", "type": "text", "required": True},
            {"name": "result", "label": "顯示", "type": "text", "required": False},
            {"name": "actions", "label": "操作", "type": "action"},
        ]
    elif "計算" in profile.entities:
        schema = [
            {"name": "num1", "label": "數字一", "type": "text", "required": True},
            {"name": "num2", "label": "數字二", "type": "text", "required": True},
            {"name": "result", "label": "結果", "type": "text", "required": False},
            {"name": "actions", "label": "操作", "type": "action"},
        ]

    return schema


def _infer_api_schema(profile: IntentProfile) -> List[Dict]:
    """API schema：endpoint + method + auth"""
    return [
        {"name": "endpoint", "label": "端點", "type": "text",
         "required": True, "placeholder": "/api/users"},
        {"name": "method", "label": "方法", "type": "badge",
         "required": True, "options": ["GET", "POST", "PUT", "DELETE"]},
        {"name": "authRequired", "label": "需要認證", "type": "checkbox", "required": False},
        {"name": "description", "label": "描述", "type": "text", "required": False},
        {"name": "actions", "label": "操作", "type": "action"},
    ]
