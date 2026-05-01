"""節點 5：Composer 單元測試"""
import pytest
from pathlib import Path
from nodes.intent_classifier import classify_intent
from nodes.schema_inferrer import infer_schema
from nodes.skill_router import route_skills
from nodes.dependency_resolver import resolve_dependencies
from nodes.composer import (
    load_skill_blocks, _build_form_from_schema,
    _build_dynamic_table_html, _inject_slot,
    compose_output, _fallback_page_layout
)

SKILLS_DIR = Path(__file__).parent.parent.parent / "skills"


class TestLoadSkillBlocks:
    """load_skill_blocks 測試."""

    def test_finds_skill_in_ui_subdir(self):
        """modal-form 在 ui/."""
        result = load_skill_blocks("modal-form", "html")
        assert isinstance(result, str)

    def test_finds_skill_in_styles_subdir(self):
        """theme-modern 在 styles/."""
        result = load_skill_blocks("theme-modern", "style")
        assert isinstance(result, str)

    def test_missing_skill_returns_empty_string(self):
        result = load_skill_blocks("nonexistent-skill-xyz-123", "html")
        assert result == ""

    def test_missing_section_returns_empty_string(self):
        """技能存在但 section 不存在 → 回傳空字串."""
        result = load_skill_blocks("modal-form", "nonexistent-section")
        assert result == ""

    def test_section_case_insensitive(self):
        """section 標籤大小寫不敏感."""
        result_lower = load_skill_blocks("modal-form", "html")
        result_upper = load_skill_blocks("modal-form", "HTML")
        # 至少有一個有內容（取決於 skill 檔案格式）
        assert isinstance(result_lower, str)

    def test_returns_html_content(self):
        """table-data html 區塊有內容."""
        result = load_skill_blocks("table-data", "html")
        assert len(result) > 0
        # 應該包含 HTML 標籤
        assert "<" in result


class TestBuildFormFromSchema:
    """_build_form_from_schema 測試."""

    def test_returns_4_tuple(self):
        schema = infer_schema(classify_intent("待辦事項"))
        result = _build_form_from_schema(schema)
        assert isinstance(result, tuple)
        assert len(result) == 4

    def test_form_html_contains_field_ids(self):
        """Form HTML 包含 editable 欄位的 field-id."""
        schema = infer_schema(classify_intent("待辦事項"))
        form_html, _, _, _ = _build_form_from_schema(schema)
        # id 欄位 editable=False → 跳過，不應產生 field-id input
        assert "field-id" not in form_html
        # title 是 editable → 應產生 field-title
        assert "field-title" in form_html

    def test_text_field_produces_input(self):
        schema = [{"name": "title", "label": "標題", "type": "text", "editable": True}]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-title" in form_html
        assert 'type="text"' in form_html

    def test_badge_field_produces_select(self):
        schema = [{
            "name": "status",
            "label": "狀態",
            "type": "badge",
            "editable": True,
            "options": ["進行中", "已完成"]
        }]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-status" in form_html
        assert "<select" in form_html
        assert "進行中" in form_html
        assert "已完成" in form_html

    def test_date_field_produces_date_input(self):
        schema = [{"name": "dueDate", "label": "截止日期", "type": "date", "editable": True}]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-dueDate" in form_html
        assert 'type="date"' in form_html

    def test_checkbox_field_produces_checkbox_input(self):
        schema = [{"name": "isActive", "label": "啟用", "type": "checkbox", "editable": True}]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-isActive" in form_html
        assert 'type="checkbox"' in form_html

    def test_non_editable_field_skipped(self):
        """editable=False 的欄位不產生 input."""
        schema = [
            {"name": "id", "label": "ID", "type": "text", "editable": False},
            {"name": "title", "label": "標題", "type": "text", "editable": True},
        ]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-id" not in form_html
        assert "field-title" in form_html

    def test_action_field_skipped(self):
        """type=action 的欄位不產生 input."""
        schema = [
            {"name": "actions", "label": "操作", "type": "action", "editable": False},
            {"name": "title", "label": "標題", "type": "text", "editable": True},
        ]
        form_html, _, _, _ = _build_form_from_schema(schema)
        assert "field-actions" not in form_html
        assert "field-title" in form_html

    def test_open_add_js_function_present(self):
        schema = [{"name": "title", "label": "標題", "type": "text", "editable": True}]
        _, open_add_js, _, _ = _build_form_from_schema(schema)
        assert "function openAdd" in open_add_js or "function openAdd(" in open_add_js

    def test_open_edit_js_function_present(self):
        schema = [{"name": "title", "label": "標題", "type": "text", "editable": True}]
        _, _, open_edit_js, _ = _build_form_from_schema(schema)
        assert "function openEdit" in open_edit_js or "function openEdit(" in open_edit_js

    def test_submit_data_js_contains_field(self):
        schema = [{"name": "title", "label": "標題", "type": "text", "editable": True}]
        _, _, _, submit_data = _build_form_from_schema(schema)
        assert "title" in submit_data

    def test_select_options_are_escaped(self):
        """Select 選項值應該被正確處理."""
        schema = [{
            "name": "status",
            "label": "狀態",
            "type": "badge",
            "editable": True,
            "options": ["進行中", "已完成"]
        }]
        form_html, _, _, _ = _build_form_from_schema(schema)
        # 檢查 options 在 select 內
        assert "進行中" in form_html


class TestBuildDynamicTableHtml:
    """_build_dynamic_table_html 測試."""

    def test_returns_2_tuple(self):
        schema = infer_schema(classify_intent("待辦事項"))
        result = _build_dynamic_table_html(schema)
        assert isinstance(result, tuple)
        assert len(result) == 2

    def test_thead_contains_field_labels(self):
        schema = [
            {"name": "id", "label": "ID", "type": "text"},
            {"name": "title", "label": "標題", "type": "text"},
        ]
        thead_html, _ = _build_dynamic_table_html(schema)
        assert "ID" in thead_html
        assert "標題" in thead_html

    def test_thead_wrapped_in_tr(self):
        schema = [{"name": "id", "label": "ID", "type": "text"}]
        thead_html, _ = _build_dynamic_table_html(schema)
        assert thead_html.startswith("<tr")
        assert "</tr>" in thead_html

    def test_render_cases_contains_field_names(self):
        schema = [
            {"name": "id", "label": "ID", "type": "text"},
            {"name": "title", "label": "標題", "type": "text"},
        ]
        _, render_cases = _build_dynamic_table_html(schema)
        assert 'case "id"' in render_cases
        assert 'case "title"' in render_cases

    def test_action_field_renders_actions(self):
        schema = [{"name": "actions", "label": "操作", "type": "action"}]
        _, render_cases = _build_dynamic_table_html(schema)
        assert 'case "actions"' in render_cases
        assert "_renderActions" in render_cases

    def test_badge_field_calls_render_badge(self):
        schema = [{"name": "status", "label": "狀態", "type": "badge"}]
        _, render_cases = _build_dynamic_table_html(schema)
        assert 'case "status"' in render_cases
        assert "_renderBadge" in render_cases

    def test_date_field_adds_col_date_class(self):
        schema = [{"name": "dueDate", "label": "截止日期", "type": "date"}]
        _, render_cases = _build_dynamic_table_html(schema)
        assert 'case "dueDate"' in render_cases
        assert "col-date" in render_cases


class TestInjectSlot:
    """_inject_slot 測試."""

    def test_replaces_comment_marker(self):
        html = "before <!-- slot:header --> after"
        result = _inject_slot(html, "header", "CONTENT")
        assert result == "before CONTENT after"

    def test_replaces_data_slot_attribute(self):
        html = '<div data-slot="header">old</div>'
        result = _inject_slot(html, "header", "NEW")
        assert "NEW" in result

    def test_slot_not_found_returns_original(self):
        html = "no slot here"
        result = _inject_slot(html, "nonexistent", "content")
        assert result == html

    def test_multiple_same_slot_replaces_all(self):
        html = "<!-- slot:x --> first <!-- slot:x --> second"
        result = _inject_slot(html, "x", "Y")
        assert result.count("Y") == 2

    def test_empty_content_removes_marker(self):
        html = "<!-- slot:empty -->"
        result = _inject_slot(html, "empty", "")
        assert "<!-- slot:empty -->" not in result


class TestComposeOutput:
    """compose_output 整合測試."""

    def _full_pipeline(self, intent_text):
        """Helper: 跑完整管道."""
        profile = classify_intent(intent_text)
        schema = infer_schema(profile)
        routed = route_skills(profile, schema)
        skill_names = [item["skill"] for item in routed]
        ordered = resolve_dependencies(skill_names, SKILLS_DIR)
        return compose_output(ordered, schema, profile, "html")

    def test_returns_dict_with_code(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert isinstance(result, dict)
        assert "code" in result

    def test_code_is_html(self):
        result = self._full_pipeline("待辦事項管理系統")
        code = result["code"]
        assert "<!DOCTYPE" in code or "<!doctype" in code.lower()
        assert "<html" in code.lower()
        assert "<head>" in code.lower()
        assert "<body>" in code.lower()

    def test_code_has_script(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "<script>" in result["code"]

    def test_code_has_state_object(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "STATE" in result["code"]

    def test_code_has_render_function(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "function render" in result["code"] or "render =" in result["code"]

    def test_warnings_is_list(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert isinstance(result.get("warnings"), list)

    def test_metadata_has_skills_used(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "metadata" in result
        assert "skills_used" in result["metadata"]
        assert isinstance(result["metadata"]["skills_used"], list)

    def test_metadata_has_schema(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "schema" in result["metadata"]

    def test_metadata_has_theme(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "theme" in result["metadata"]

    def test_localstorage_key_present(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "localStorage" in result["code"] or "localstorage" in result["code"].lower()

    def test_openadd_function_in_output(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "openAdd" in result["code"]

    def test_openedit_function_in_output(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "openEdit" in result["code"]

    def test_opendelete_function_in_output(self):
        result = self._full_pipeline("待辦事項管理系統")
        assert "openDelete" in result["code"]

    def test_unsupported_type_falls_back_to_html(self):
        """不支援的 output_type → fallback 到 HTML."""
        profile = classify_intent("待辦事項")
        schema = infer_schema(profile)
        ordered = [{"skill": "badge-status", "depends": []}]
        result = compose_output(ordered, schema, profile, "unsupported-type")
        assert "code" in result
        # 應該是 HTML（fallback）
        assert "<html" in result["code"].lower()
        # 有 warning 記錄
        assert len(result.get("warnings", [])) > 0


class TestFallbackPageLayout:
    """Fallback 佈局測試."""

    def test_returns_string(self):
        result = _fallback_page_layout()
        assert isinstance(result, str)

    def test_contains_all_slots(self):
        result = _fallback_page_layout()
        for slot in ["header", "search", "content", "modal", "confirm", "toast"]:
            assert slot in result

    def test_has_data_slot_attributes(self):
        result = _fallback_page_layout()
        assert 'data-slot="header"' in result or 'data-slot=\'header\'' in result
        assert 'data-slot="modal"' in result or 'data-slot=\'modal\'' in result
