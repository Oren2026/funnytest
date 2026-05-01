"""節點 2：Schema Inferrer（資料結構推斷器）"""
from typing import List, Dict
from .intent_classifier import IntentProfile, IntentType


STANDARD_CRUD_FIELDS = [
    {"name": "id", "label": "ID", "type": "text", "required": False, "editable": False},
    {"name": "title", "label": "標題", "type": "text", "required": True, "editable": True, "placeholder": "請輸入名稱"},
    {"name": "description", "label": "描述", "type": "text", "required": False, "editable": True},
    {"name": "priority", "label": "優先權", "type": "badge", "required": False, "editable": True, "options": ["高", "中", "低"], "default": "中"},
    {"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True, "options": ["進行中", "已完成", "待處理"], "default": "待處理"},
    {"name": "dueDate", "label": "截止日期", "type": "date", "required": False, "editable": True},
    {"name": "createdAt", "label": "建立時間", "type": "date", "required": False, "editable": False},
    {"name": "updatedAt", "label": "更新時間", "type": "date", "required": False, "editable": False},
    {"name": "actions", "label": "操作", "type": "action", "required": False, "editable": False},
]


def infer_schema(profile: IntentProfile) -> List[Dict]:
    """
    根據 IntentProfile 推斷資料結構。

    邏輯：
    - CRUD：使用 STANDARD_CRUD_FIELDS，根據 entities 調整 label 和新增欄位
    - DASHBOARD：生成圖表卡 + 統計卡 schema
    - GAME：生成分數、等級、存檔狀態
    - TOOL：根據 tools 類型生成對應的 input/result 欄位
    - API：生成 endpoint + method + fields
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
    """CRUD schema：基礎欄位 + 根據實體調整 + 從 context 解析具體欄位"""
    import re
    editable_fields = []  # [name, label, type, ...]
    entity_name = profile.entities[0] if profile.entities else "項目"
    context = profile.context

    # 嘗試從 context 解析具體欄位（格式：顯示OO列表（XX、XX、XX））
    explicit_fields = []
    field_pattern = re.search(r'（([^）]+)）', context)
    if field_pattern:
        explicit_fields = [f.strip() for f in field_pattern.group(1).split('、')]

    # 如果有明確欄位，用明確欄位；否則用 entity fallback
    if explicit_fields:
        # 第一個欄位當 title/name
        primary = explicit_fields[0]
        editable_fields.append({"name": "title", "label": primary, "type": "text", "required": True, "editable": True})
        # 其餘欄位根據名稱推斷類型
        for field in explicit_fields[1:]:
            f_lower = field.lower()
            if any(kw in f_lower for kw in ['分類', '類型', 'category', 'type']):
                editable_fields.append({"name": "category", "label": field, "type": "text", "required": False, "editable": True})
            elif any(kw in f_lower for kw in ['庫存', '庫存狀態', '庫存水位']):
                editable_fields.append({"name": "stockStatus", "label": field, "type": "badge",
                               "required": False, "editable": True, "options": ["有貨", "缺貨", "補貨中"], "default": "有貨"})
            elif any(kw in f_lower for kw in ['作者', '建立人', '負責人']):
                editable_fields.append({"name": "author", "label": field, "type": "text", "required": False, "editable": True})
            elif any(kw in f_lower for kw in ['數量', '庫存數量', '庫存']):
                editable_fields.append({"name": "quantity", "label": field, "type": "text", "required": False, "editable": True})
            elif any(kw in f_lower for kw in ['價格', '單價', '成本']):
                editable_fields.append({"name": "price", "label": field, "type": "text", "required": False, "editable": True})
            elif any(kw in f_lower for kw in ['電話', 'email', 'mail', '信箱']):
                editable_fields.append({"name": "contact", "label": field, "type": "text", "required": False, "editable": True})
            elif any(kw in f_lower for kw in ['狀態', 'status']):
                editable_fields.append({"name": "status", "label": field, "type": "badge",
                               "required": False, "editable": True, "options": ["進行中", "已完成", "待處理"], "default": "待處理"})
            else:
                editable_fields.append({"name": "description", "label": field, "type": "text", "required": False, "editable": True})
    else:
        # 根據實體調整 title label（原本的 fallback 邏輯）
        title_labels = {
            "任務": "任務名稱",
            "庫存": "商品名稱",
            "客戶": "客戶名稱",
            "報表": "報表名稱",
            "書籍": "書名",
        }
        title_label = title_labels.get(entity_name, f"{entity_name}名稱")
        editable_fields.append({"name": "title", "label": title_label, "type": "text", "required": True, "editable": True})

        if "描述" in context or "description" in context.lower():
            editable_fields.append({"name": "description", "label": "描述", "type": "text", "required": False, "editable": True})

        if entity_name in ["任務", "待辦", "工作"]:
            editable_fields.append({"name": "priority", "label": "優先權", "type": "badge", "required": False, "editable": True, "options": ["高", "中", "低"], "default": "中"})
            editable_fields.append({"name": "status", "label": "狀態", "type": "badge", "required": False, "editable": True, "options": ["進行中", "已完成", "待處理"], "default": "待處理"})

        if entity_name in ["任務", "待辦", "工作", "庫存"]:
            editable_fields.append({"name": "dueDate", "label": "截止日期", "type": "date", "required": False, "editable": True})

    # Non-editable 欄位統一加在尾部（invariant: editable 在前，non-editable 在後）
    non_editable = [
        {"name": "id", "label": "ID", "type": "text", "required": False, "editable": False},
        {"name": "createdAt", "label": "建立時間", "type": "date", "required": False, "editable": False},
        {"name": "updatedAt", "label": "更新時間", "type": "date", "required": False, "editable": False},
        {"name": "actions", "label": "操作", "type": "action", "required": False, "editable": False},
    ]

    return editable_fields + non_editable


def _infer_dashboard_schema(profile: IntentProfile) -> List[Dict]:
    """Dashboard schema：統計卡 + 圖表"""
    return [
        {"name": "metric", "label": "指標名稱", "type": "text", "required": True},
        {"name": "value", "label": "數值", "type": "text", "required": True},
        {"name": "change", "label": "變化", "type": "text", "required": False},
        {"name": "trend", "label": "趨勢", "type": "badge", "required": False, "options": ["up", "down", "stable"]},
        {"name": "chartType", "label": "圖表類型", "type": "badge", "required": False, "options": ["折線圖", "柱狀圖", "圓餅圖"]},
    ]


def _infer_game_schema(profile: IntentProfile) -> List[Dict]:
    """Game schema：分數、等級、遊戲狀態"""
    return [
        {"name": "score", "label": "分數", "type": "text", "required": False},
        {"name": "level", "label": "等級", "type": "text", "required": False},
        {"name": "lives", "label": "生命", "type": "text", "required": False},
        {"name": "gameState", "label": "遊戲狀態", "type": "badge", "options": ["進行中", "暫停", "結束"]},
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
        {"name": "endpoint", "label": "端點", "type": "text", "required": True, "placeholder": "/api/users"},
        {"name": "method", "label": "方法", "type": "badge", "required": True, "options": ["GET", "POST", "PUT", "DELETE"]},
        {"name": "authRequired", "label": "需要認證", "type": "checkbox", "required": False},
        {"name": "description", "label": "描述", "type": "text", "required": False},
        {"name": "actions", "label": "操作", "type": "action"},
    ]
