"""
intent_parser.py — 意圖解析器

分析自然語意描述，對應到需要的技能。
"""
import re

from compiler.html_compiler import compile_html
from compiler.react_compiler import compile_react


# 意圖 → 技能映射
INTENT_MAP = {
    # 核心佈局
    "header": ["layout-header"],
    "toolbar": ["search-bar"],
    "table": ["table-data"],
    "form": ["modal-form"],
    "confirm": ["confirm-dialog"],
    "toast": ["toast-notify"],

    # 按鈕
    "按鈕": ["button-primary", "button-danger"],
    "新增": ["button-primary", "modal-form"],
    "刪除": ["button-danger", "confirm-dialog"],
    "編輯": ["button-primary", "modal-form"],

    # 資料顯示
    "列表": ["table-data", "badge-status"],
    "表格": ["table-data", "badge-status"],
    "搜尋": ["search-bar"],
    "篩選": ["search-bar"],
    "排序": ["search-bar"],
    "狀態": ["badge-status"],
    "庫存": ["table-data", "search-bar", "badge-status"],

    # 通知
    "提示": ["toast-notify"],
    "錯誤": ["toast-notify"],
    "成功": ["toast-notify"],
}


def parse_schema(raw_text: str) -> list:
    """
    從 seed 文字中解析 <schema> 區塊，抽出每個欄位的
    name、label、type，回傳 list[dict]。

    支援格式：
        <schema>
            <field name="title"   label="標題"   type="string" />
            <field name="status"  label="狀態"   type="enum"   />
            <field name="count"   label="數量"   type="number" />
        </schema>

    若找不到 <schema> 區塊，回傳空列表。
    """
    schema_match = re.search(r"<schema>(.*?)</schema>", raw_text, re.DOTALL)
    if not schema_match:
        return []

    schema_text = schema_match.group(1)
    fields = []

    # 匹配 <field name="..." label="..." type="..." />  （允許屬性順序任意）
    for match in re.finditer(
        r'<field\s+([^>]+)/?>',
        schema_text,
        re.DOTALL,
    ):
        attrs_text = match.group(1)
        attrs = dict(re.findall(r'(\w+)=\"([^\"]+)\"', attrs_text))
        if "name" in attrs:
            fields.append({
                "key":   attrs.get("name", ""),
                "label": attrs.get("label", ""),
                "type":  attrs.get("type",  "text"),
            })

    return fields


def parse_items(raw_text):
    import re
    items = []
    m = re.search(r'##\s*初始資料\s*\n((?:.+\n)*)', raw_text)
    if not m:
        return []
    for line in m.group(1).split('\n'):
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        # Remove leading "- " or "- " or similar
        line = re.sub(r"^-\s*", "", line)
        parts = [p.strip() for p in line.split(',')]
        if len(parts) >= 4:
            name = parts[0].strip('"').strip("'")
            due = parts[1].strip()
            pri = parts[2].strip()
            status = parts[3].strip()
            completed = '完成' in status
            items.append({'name': name, 'dueDate': due, 'priority': pri, 'completed': completed})
    return items

def parse_seed_theme(raw_text: str) -> str:
    """從原始文字抽出 theme 設定。"""
    m = re.search(r'<style\s+theme="([^"]+)"', raw_text)
    if m:
        return m.group(1)
    return "glass"


def parse_intent(intent_text: str) -> dict:
    """分析自然語意，回傳解析後的結構。"""

    text = intent_text.lower()
    skills_used = set()
    intent_type = "unknown"

    # 提取 <name> 標籤作為應用程式名稱
    name_match = re.search(r"<name>([^<]+)</name>", intent_text)
    app_name = name_match.group(1).strip() if name_match else None

    # 偵測意圖類型
    if any(k in text for k in ["登入", "登入頁", "login"]):
        intent_type = "login"
        skills_used.update(["layout-header", "button-primary", "input-field", "toast-notify"])
    elif any(k in text for k in ["倉儲", "庫存", "warehouse", "inventory"]):
        intent_type = "warehouse"
        skills_used.update([
            "layout-header", "button-primary", "button-danger",
            "table-data", "modal-form", "toast-notify",
            "search-bar", "badge-status", "confirm-dialog"
        ])
    elif any(k in text for k in ["代辦", "待辦", "todo", "task"]):
        intent_type = "todo"
        skills_used.update([
            "layout-header", "button-primary", "button-danger",
            "table-data", "modal-form", "toast-notify",
            "search-bar", "badge-status", "confirm-dialog"
        ])
    elif any(k in text for k in ["表單", "form"]):
        intent_type = "form"
        skills_used.update(["modal-form", "button-primary", "toast-notify"])
    else:
        # 通用關鍵字匹配
        intent_type = "generic"
        for keyword, skills in INTENT_MAP.items():
            if keyword in text:
                skills_used.update(skills)

    # 如果沒匹配到任何技能，使用預設組合
    if not skills_used:
        skills_used = ["layout-header", "button-primary", "table-data"]

    return {
        "intent_type": intent_type,
        "skills": list(skills_used),
        "name": app_name,
        "schema": parse_schema(intent_text),
        "items": parse_items(intent_text),
        "original": intent_text,
    }

def synthesize(intent_text: str, target: str) -> str:
    """根據意圖與目標平台生成代碼。"""

    parsed = parse_intent(intent_text)
    skills = parsed["skills"]
    parsed["theme"] = parse_seed_theme(intent_text)

    if target == "html":
        return compile_html(skills, parsed)
    elif target == "react":
        return compile_react(skills, parsed)
    else:
        raise ValueError(f"Unknown target: {target}")


if __name__ == "__main__":
    intent = "倉儲管理系統：上方有新增按鈕，中間是庫存列表，支援搜尋、篩選、排序，支援新增/編輯/刪除，刪除前確認"
    for target in ["html", "react"]:
        result = synthesize(intent, target)
        print(f"\n=== {target.upper()} OUTPUT ({len(result)} chars) ===")
        print(result[:500])
        print("...")
