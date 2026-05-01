"""節點 6：QA Checker 單元測試"""
import pytest
from nodes.intent_classifier import classify_intent, IntentType
from nodes.schema_inferrer import infer_schema
from nodes.qa_checker import qa_check, QAIssue


class TestQAIssue:
    """QAIssue 結構測試."""

    def test_error_level(self):
        issue = QAIssue("error", "test error")
        assert issue.level == "error"
        assert issue.message == "test error"

    def test_warning_level(self):
        issue = QAIssue("warning", "test warning")
        assert issue.level == "warning"

    def test_info_level(self):
        issue = QAIssue("info", "test info")
        assert issue.level == "info"

    def test_location_default_empty(self):
        issue = QAIssue("error", "test")
        assert issue.location == ""

    def test_location_set(self):
        issue = QAIssue("error", "test", "html")
        assert issue.location == "html"

    def test_repr_format(self):
        issue = QAIssue("error", "missing DOCTYPE", "html")
        r = repr(issue)
        assert "ERROR" in r
        assert "missing DOCTYPE" in r
        assert "html" in r


class TestQACheckPassed:
    """passed=True 條件測試."""

    def _make_compiled(self, html_code, skills=None, theme="modern"):
        return {
            "code": html_code,
            "warnings": [],
            "metadata": {
                "skills_used": skills or [],
                "schema": [],
                "theme": theme
            }
        }

    def test_minimal_valid_html_passes(self):
        """基本有效 HTML → passed=True."""
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is True

    def test_pure_empty_code_fails(self):
        """空白代碼 → 有結構 error."""
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(""), profile, schema)
        assert result["passed"] is False

    def test_warning_only_still_passes(self):
        """只有 warning → passed=True（warning 不影響通過）."""
        # 手動構造一個有 warning 但無 error 的場景
        # 缺少 viewport meta → info 等級，不影響 passed
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        # 只有 info 等級 → passed
        errors = [i for i in result["issues"] if i.level == "error"]
        assert len(errors) == 0


class TestQACheckStructureErrors:
    """結構錯誤 → error 級別."""

    def _make_compiled(self, html_code, skills=None):
        return {
            "code": html_code,
            "warnings": [],
            "metadata": {
                "skills_used": skills or [],
                "schema": [],
                "theme": "modern"
            }
        }

    def test_missing_doctype_error(self):
        html = "<html><head></head><body></body></html>"
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any(i.level == "error" and "DOCTYPE" in i.message for i in result["issues"])

    def test_missing_html_tag_error(self):
        html = "<!DOCTYPE html><head></head><body></body></html>"
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any(i.level == "error" and "<html>" in i.message for i in result["issues"])

    def test_missing_head_tag_error(self):
        html = "<!DOCTYPE html><html></html><body></body></html>"
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any(i.level == "error" and "<head>" in i.message for i in result["issues"])

    def test_missing_body_tag_error(self):
        html = "<!DOCTYPE html><html><head></head></html>"
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False


class TestQACheckJSFunctions:
    """JS 函式缺失 → error."""

    def _make_compiled(self, html_code, skills=None):
        return {
            "code": html_code,
            "warnings": [],
            "metadata": {
                "skills_used": skills or [],
                "schema": [],
                "theme": "modern"
            }
        }

    def test_missing_openadd_error(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any("openAdd" in i.message for i in result["issues"] if i.level == "error")

    def test_missing_openedit_error(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False

    def test_missing_opendelete_error(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False


class TestQACheckSecurity:
    """安全性問題 → error."""

    def _make_compiled(self, html_code):
        return {
            "code": html_code,
            "warnings": [],
            "metadata": {
                "skills_used": [],
                "schema": [],
                "theme": "modern"
            }
        }

    def test_eval_usage_error(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() { eval('1+1'); }\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any("eval" in i.message.lower() for i in result["issues"] if i.level == "error")

    def test_dangerous_innerhtml_error(self):
        """innerHTML = X + 字串拼接 → error."""
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {\n"
            "  var x = document.getElementById('inp').value;\n"
            "  document.getElementById('out').innerHTML = '<b>' + x + '</b>';\n"
            "}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False

    def test_document_write_error(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {\n"
            "  document.write('<b>test</b>');\n"
            "}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        assert result["passed"] is False
        assert any("document.write" in i.message for i in result["issues"] if i.level == "error")


class TestQACheckWarnings:
    """Warning 等級測試."""

    def _make_compiled(self, html_code, skills=None, schema_fields=None):
        return {
            "code": html_code,
            "warnings": [],
            "metadata": {
                "skills_used": skills or [],
                "schema": schema_fields or [],
                "theme": "modern"
            }
        }

    def test_missing_viewport_info(self):
        """沒有 viewport meta → info 等級."""
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        # info 等級 → passed 仍為 True
        assert result["passed"] is True

    def test_missing_toast_info(self):
        """沒有 showToast → info 等級."""
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check(self._make_compiled(html), profile, schema)
        # info → 不影響 passed
        assert result["passed"] is True
        info_msgs = [i.message for i in result["issues"] if i.level == "info"]
        assert any("showToast" in msg or "通知" in msg for msg in info_msgs)


class TestQACheckSchemaCoverage:
    """Schema 欄位覆蓋測試."""

    def _make_compiled_with_field(self, field_name, has_input=True):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<form id='inventory-form'>\n"
        )
        if has_input:
            html += f'<input id="field-{field_name}" type="text" />\n'
        html += (
            "</form>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        return {
            "code": html,
            "warnings": [],
            "metadata": {
                "skills_used": ["modal-form"],
                "schema": [{"name": field_name, "type": "text", "editable": True}],
                "theme": "modern"
            }
        }

    def test_field_with_input_no_warning(self):
        """有對應 input 的欄位 → 無 warning."""
        profile = classify_intent("待辦事項")
        result = qa_check(self._make_compiled_with_field("title", has_input=True), profile, [])
        warnings = [i for i in result["issues"] if i.level == "warning" and "title" in i.message]
        assert len(warnings) == 0

    def test_missing_field_in_schema_triggers_warning(self):
        """有 schema 欄位但 HTML 沒有對應 input → warning."""
        # 直接構造一個缺 field-title input 的 HTML，配合有 field 的 schema
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>Test</title></head>\n"
            "<body>\n"
            "<form id='inventory-form'>\n"
            # 故意只放 description，缺少 title
            "<input id='field-description' type='text' />\n"
            "</form>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = [
            {"name": "title", "type": "text", "editable": True},
            {"name": "description", "type": "text", "editable": True},
        ]
        result = qa_check({
            "code": html,
            "warnings": [],
            "metadata": {
                "skills_used": ["modal-form"],
                "schema": schema,
                "theme": "modern"
            }
        }, profile, schema)
        field_warnings = [i for i in result["issues"]
                         if i.level == "warning" and "title" in i.message]
        assert len(field_warnings) > 0


class TestQACheckReturnFormat:
    """返回格式測試."""

    def test_returns_dict(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>T</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check({
            "code": html,
            "warnings": [],
            "metadata": {"skills_used": [], "schema": schema, "theme": "modern"}
        }, profile, schema)
        assert isinstance(result, dict)
        assert "passed" in result
        assert "issues" in result

    def test_issues_is_list(self):
        html = (
            "<!DOCTYPE html>\n"
            "<html><head><title>T</title></head>\n"
            "<body>\n"
            "<script>\n"
            "function openAdd() {}\n"
            "function openEdit(id) {}\n"
            "function openDelete(id) {}\n"
            "</script>\n"
            "</body></html>"
        )
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        result = qa_check({
            "code": html,
            "warnings": [],
            "metadata": {"skills_used": [], "schema": schema, "theme": "modern"}
        }, profile, schema)
        assert isinstance(result["issues"], list)
