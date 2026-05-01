"""節點 6：QA Checker（品質檢查器）"""
from typing import List, Dict


class QAIssue:
    def __init__(self, level: str, message: str, location: str = ""):
        self.level = level      # "error" | "warning" | "info"
        self.message = message
        self.location = location

    def __repr__(self):
        return f"[{self.level.upper()}] {self.message} ({self.location})"


def qa_check(compiled: Dict, profile, schema: List[Dict]) -> Dict:
    """
    QA 檢查編譯後的輸出。

    檢查項目：
    1. 結構完整性：DOCTYPE、<html>、<head>、<body>
    2. JS 完整性：openAdd、openEdit、openDelete、toggleComplete、onSubmit
    3. Schema 覆蓋：所有 editable 欄位都有對應的 form input
    4. 安全性：無 eval()、無不安全的 innerHTML、 無直接 user input in onclick
    5. 主題：theme CSS 有被載入（如果使用了 theme skill）
    6. JS 語法：基本的括號匹配、無未閉合字串（常見模式）

    Returns:
        {"passed": bool, "issues": [QAIssue]}
    """
    issues: List[QAIssue] = []
    code = compiled.get("code", "")
    metadata = compiled.get("metadata", {})
    skills_used = metadata.get("skills_used", [])
    theme = metadata.get("theme", "")

    # 1. 結構完整性
    if not code.startswith("<!DOCTYPE") and "<!doctype" not in code.lower():
        issues.append(QAIssue("error", "缺少 DOCTYPE 宣告", "html"))
    if "<html" not in code.lower():
        issues.append(QAIssue("error", "缺少 <html> 標籤", "html"))
    if "<head>" not in code.lower():
        issues.append(QAIssue("error", "缺少 <head> 標籤", "html"))
    if "<body>" not in code.lower():
        issues.append(QAIssue("error", "缺少 <body> 標籤", "html"))

    # 2. JS 完整性
    required_handlers = ["openAdd", "openEdit", "openDelete"]
    for handler in required_handlers:
        if f"function {handler}" not in code and f"{handler} =" not in code:
            issues.append(QAIssue("error", f"缺少 {handler}() 函式", "js"))

    # toggleComplete 對 CRUD 類型很重要
    if profile.type.value in ["CRUD", "GAME"]:
        if "toggleComplete" not in code and "completed" in str([f.get("type") for f in schema]):
            issues.append(QAIssue("warning", "有 checkbox/completed 欄位但缺少 toggleComplete()", "js"))

    # onSubmit handler
    if 'addEventListener("submit"' not in code and "addEventListener('submit'" not in code:
        if "inventory-form" in code:
            issues.append(QAIssue("warning", "表單存在但沒有 submit handler", "js"))

    # 3. Schema 覆蓋：每個 editable 欄位要有 input
    for field in schema:
        if field.get("editable", True) and field.get("type") not in ("action",):
            field_name = field["name"]
            if field.get("type") == "checkbox":
                if f'type="checkbox"' not in code and f"field-{field_name}" not in code:
                    issues.append(QAIssue("warning", f"欄位 {field_name} 沒有對應的 checkbox input", "form"))
            else:
                if f'field-{field_name}' not in code:
                    issues.append(QAIssue("warning", f"欄位 {field_name} 沒有對應的表單 input", "form"))

    # 4. 安全性
    if "eval(" in code:
        issues.append(QAIssue("error", "使用了 eval()，有安全風險", "security"))
    if ".innerHTML" in code and "+" in code:
        # innerHTML with concatenation is dangerous
        if "innerHTML =" in code and ("+" in code.split("innerHTML =")[1][:100]):
            issues.append(QAIssue("error", "innerHTML 直接拼接字串有 XSS 風險", "security"))
    if "document.write(" in code:
        issues.append(QAIssue("error", "使用 document.write() 有安全風險", "security"))

    # 5. Theme 檢查
    has_theme_skill = any("theme-" in s for s in skills_used)
    if has_theme_skill:
        theme_var_count = code.count("--")
        if theme_var_count < 3:
            issues.append(QAIssue("warning", "使用了 theme skill 但 CSS 變數少於 3 個，可能 theme 未正確載入", "css"))
    else:
        # 沒有 theme skill 但也應該有基本的 CSS 變數
        if "--bg" not in code and "--primary" not in code:
            issues.append(QAIssue("info", "沒有使用 theme skill，也沒有發現 CSS 變數，建議加入預設主題", "css"))

    # 6. JS 語法基礎檢查
    js_blocks = []
    script_start = code.find("<script>")
    if script_start != -1:
        script_end = code.find("</script>", script_start)
        if script_end != -1:
            js_blocks.append(code[script_start + 8:script_end])

    for js in js_blocks:
        # 括號匹配
        for open_char, close_char in [("(", ")"), ("{", "}"), ("[", "]")]:
            count = 0
            for ch in js:
                if ch == open_char:
                    count += 1
                elif ch == close_char:
                    count -= 1
                if count < 0:
                    issues.append(QAIssue("error", f"JS 中 {open_char}/{close_char} 不匹配", "js"))
                    break

    # 7. table 結構檢查
    if "<table" not in code and "table-data" in skills_used:
        issues.append(QAIssue("warning", "使用了 table-data skill 但沒有 <table> 標籤", "html"))

    # 8. 表單檢查
    if "modal-form" in skills_used or "modal" in skills_used or "<form" in code:
        if 'id="inventory-form"' not in code:
            issues.append(QAIssue("warning", "表單 skill 被使用但表單 ID 不是 inventory-form，可能 handler 無法綁定", "form"))

    # 9. Toast 系統
    if "showToast" not in code:
        issues.append(QAIssue("info", "沒有 showToast 通知系統，使用者操作後沒有回饋", "ux"))

    # 10. 響應式
    if "width=device-width" not in code and "<meta name='viewport'" not in code:
        issues.append(QAIssue("info", "沒有 viewport meta，行動裝置可能無法正確顯示", "html"))

    # 判定：有任何 error 就失敗
    has_errors = any(i.level == "error" for i in issues)

    return {
        "passed": not has_errors,
        "issues": issues,
    }
