"""節點 2：Schema Inferrer 單元測試"""
import pytest
from nodes.intent_classifier import IntentProfile, IntentType, classify_intent
from nodes.schema_inferrer import (
    infer_schema, _infer_crud_schema, _infer_dashboard_schema,
    _infer_game_schema, _infer_tool_schema, _infer_api_schema,
    STANDARD_CRUD_FIELDS
)


class TestInferCrudSchema:
    """CRUD Schema 推斷測試."""

    def test_has_id_field(self, crud_profile):
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "id" in field_names

    def test_id_not_editable(self, crud_profile):
        schema = infer_schema(crud_profile)
        id_field = next(f for f in schema if f["name"] == "id")
        assert id_field["editable"] is False

    def test_has_title_field(self, crud_profile):
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "title" in field_names

    def test_title_is_editable(self, crud_profile):
        schema = infer_schema(crud_profile)
        title_field = next(f for f in schema if f["name"] == "title")
        assert title_field["editable"] is True

    def test_has_actions_field(self, crud_profile):
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "actions" in field_names

    def test_actions_type_is_action(self, crud_profile):
        schema = infer_schema(crud_profile)
        actions_field = next(f for f in schema if f["name"] == "actions")
        assert actions_field["type"] == "action"

    def test_actions_not_editable(self, crud_profile):
        schema = infer_schema(crud_profile)
        actions_field = next(f for f in schema if f["name"] == "actions")
        assert actions_field["editable"] is False

    def test_has_createdAt(self, crud_profile):
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "createdAt" in field_names

    def test_createdAt_not_editable(self, crud_profile):
        schema = infer_schema(crud_profile)
        f = next(f for f in schema if f["name"] == "createdAt")
        assert f["editable"] is False

    def test_priority_badge_for_todo(self, crud_profile):
        """待辦事項預設有 priority badge."""
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "priority" in field_names
        priority_field = next(f for f in schema if f["name"] == "priority")
        assert priority_field["type"] == "badge"
        assert "高" in priority_field.get("options", [])

    def test_status_badge_for_todo(self, crud_profile):
        """待辦事項預設有 status badge."""
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "status" in field_names
        status_field = next(f for f in schema if f["name"] == "status")
        assert status_field["type"] == "badge"

    def test_dueDate_for_todo(self, crud_profile):
        """待辦事項預設有 dueDate."""
        schema = infer_schema(crud_profile)
        field_names = [f["name"] for f in schema]
        assert "dueDate" in field_names
        due_field = next(f for f in schema if f["name"] == "dueDate")
        assert due_field["type"] == "date"

    def test_explicit_fields_from_context(self):
        """明確欄位（格式：OO、XX、XX）應被解析."""
        # 解析器匹配「（）」（全形括號），需要使用全形括號
        profile = classify_intent("顯示書籍列表（書名、作者、分類）")
        schema = infer_schema(profile)
        field_labels = [f["label"] for f in schema]
        assert "書名" in field_labels
        assert "作者" in field_labels
        assert "分類" in field_labels

    def test_explicit_field_first_becomes_title(self):
        """明確欄位的第一個 → title."""
        profile = classify_intent("顯示書籍列表，包含書名、作者")
        schema = infer_schema(profile)
        # 第一個明確欄位 → title label 會是書名
        title_field = next(f for f in schema if f["name"] == "title")
        assert title_field["label"] == "書名"

    def test_author_field_from_keywords(self):
        """含「作者」關鍵字的明確欄位 → author type=text."""
        # 需要使用全形括號讓 explicit_fields 解析成立
        profile = classify_intent("顯示書籍列表（書名、作者）")
        schema = infer_schema(profile)
        field_names = [f["name"] for f in schema]
        assert "author" in field_names

    def test_category_field_detection(self):
        """含「分類」關鍵字 → category badge."""
        profile = classify_intent("顯示書籍列表（書名、分類）")
        schema = infer_schema(profile)
        field_names = [f["name"] for f in schema]
        assert "category" in field_names

    def test_no_entities_fallback(self):
        """無 entities → 使用「項目」."""
        profile = IntentProfile(type=IntentType.CRUD, entities=[])
        schema = infer_schema(profile)
        title_field = next(f for f in schema if f["name"] == "title")
        assert "項目" in title_field["label"]

    def test_returns_list(self, crud_profile):
        schema = infer_schema(crud_profile)
        assert isinstance(schema, list)
        assert len(schema) > 0

    def test_schema_dict_format(self, crud_profile):
        schema = infer_schema(crud_profile)
        for field in schema:
            assert isinstance(field, dict)
            assert "name" in field
            assert "label" in field
            assert "type" in field


class TestInferDashboardSchema:
    """Dashboard Schema 推斷測試."""

    def test_returns_correct_fields(self, dashboard_profile):
        schema = infer_schema(dashboard_profile)
        field_names = [f["name"] for f in schema]
        assert "metric" in field_names
        assert "value" in field_names
        assert "change" in field_names
        assert "trend" in field_names
        assert "chartType" in field_names

    def test_trend_is_badge(self, dashboard_profile):
        schema = infer_schema(dashboard_profile)
        trend_field = next(f for f in schema if f["name"] == "trend")
        assert trend_field["type"] == "badge"
        assert "up" in trend_field.get("options", [])

    def test_chartType_is_badge(self, dashboard_profile):
        schema = infer_schema(dashboard_profile)
        chart_field = next(f for f in schema if f["name"] == "chartType")
        assert chart_field["type"] == "badge"


class TestInferGameSchema:
    """Game Schema 推斷測試."""

    def test_returns_game_fields(self, game_profile):
        schema = infer_schema(game_profile)
        field_names = [f["name"] for f in schema]
        assert "score" in field_names
        assert "level" in field_names
        assert "lives" in field_names
        assert "gameState" in field_names
        assert "actions" in field_names

    def test_gamestate_is_badge(self, game_profile):
        schema = infer_schema(game_profile)
        gs_field = next(f for f in schema if f["name"] == "gameState")
        assert gs_field["type"] == "badge"


class TestInferToolSchema:
    """Tool Schema 推斷測試."""

    def test_basic_tool_fields(self, tool_profile):
        """計時器應該有 time 和 result."""
        schema = infer_schema(tool_profile)
        field_names = [f["name"] for f in schema]
        assert "time" in field_names or "input" in field_names
        assert "result" in field_names or "result" in field_names

    def test_tool_has_actions(self, tool_profile):
        schema = infer_schema(tool_profile)
        field_names = [f["name"] for f in schema]
        assert "actions" in field_names


class TestInferApiSchema:
    """API Schema 推斷測試."""

    def test_returns_api_fields(self, api_profile):
        schema = infer_schema(api_profile)
        field_names = [f["name"] for f in schema]
        assert "endpoint" in field_names
        assert "method" in field_names
        assert "authRequired" in field_names

    def test_method_is_badge(self, api_profile):
        schema = infer_schema(api_profile)
        method_field = next(f for f in schema if f["name"] == "method")
        assert method_field["type"] == "badge"
        assert "GET" in method_field.get("options", [])
        assert "POST" in method_field.get("options", [])

    def test_authrequired_is_checkbox(self, api_profile):
        schema = infer_schema(api_profile)
        auth_field = next(f for f in schema if f["name"] == "authRequired")
        assert auth_field["type"] == "checkbox"


class TestSchemaFieldProperties:
    """Schema 欄位屬性一致性測試."""

    def test_editable_false_fields_always_last(self):
        """Non-editable 欄位（id, createdAt, updatedAt, actions）必須連續佔據 schema 尾部."""
        schema = infer_schema(classify_intent("代辦事項管理系統"))
        # 找出 editable 欄位的最末位置
        last_editable_idx = -1
        for i, f in enumerate(schema):
            if f.get("editable") is True:
                last_editable_idx = i
        # 所有 non-editable 欄位必須在 last_editable_idx 之後
        non_editable_at_end = True
        for i, f in enumerate(schema):
            if f.get("editable") is False and i <= last_editable_idx:
                non_editable_at_end = False
                break
        assert non_editable_at_end, \
            f"Non-editable fields must be at the end; schema: {[(f['name'], f.get('editable')) for f in schema]}"

    def test_badge_fields_have_options(self, crud_schema):
        """Badge 類型欄位一定有 options."""
        for f in crud_schema:
            if f.get("type") == "badge":
                assert "options" in f, f"Badge field {f['name']} missing options"
                assert isinstance(f["options"], list)
                assert len(f["options"]) > 0

    def test_action_field_type(self, crud_schema):
        """type=action 的欄位一定是 non-editable."""
        for f in crud_schema:
            if f.get("type") == "action":
                assert f.get("editable") is False
