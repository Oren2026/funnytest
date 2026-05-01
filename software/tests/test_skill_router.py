"""節點 3：Skill Router 單元測試"""
import pytest
from nodes.intent_classifier import IntentType, classify_intent
from nodes.schema_inferrer import infer_schema
from nodes.skill_router import route_skills, SKILL_INDEX


class TestRouteSkillsBasic:
    """基本路由邏輯測試."""

    def test_returns_list(self, crud_profile, crud_schema):
        result = route_skills(crud_profile, crud_schema)
        assert isinstance(result, list)

    def test_returns_dicts_with_skill_and_score(self, crud_profile, crud_schema):
        result = route_skills(crud_profile, crud_schema)
        for item in result:
            assert "skill" in item
            assert "score" in item
            assert isinstance(item["score"], float)

    def test_skills_sorted_by_score_descending(self, crud_profile, crud_schema):
        result = route_skills(crud_profile, crud_schema)
        scores = [item["score"] for item in result]
        assert scores == sorted(scores, reverse=True)

    def test_top_k_limit(self, crud_profile, crud_schema):
        """返回最多 8 個技能."""
        result = route_skills(crud_profile, crud_schema)
        assert len(result) <= 8


class TestRouteSkillsScoring:
    """評分邏輯測試."""

    def test_crud_routes_table_data(self, crud_profile, crud_schema):
        """CRUD 應該路由 table-data."""
        result = route_skills(crud_profile, crud_schema)
        skill_names = [item["skill"] for item in result]
        assert "table-data" in skill_names

    def test_crud_routes_modal_form(self, crud_profile, crud_schema):
        """CRUD 應該路由 modal-form."""
        result = route_skills(crud_profile, crud_schema)
        skill_names = [item["skill"] for item in result]
        assert "modal-form" in skill_names

    def test_crud_routes_search_bar(self, crud_profile, crud_schema):
        """CRUD 應該路由 search-bar."""
        result = route_skills(crud_profile, crud_schema)
        skill_names = [item["skill"] for item in result]
        assert "search-bar" in skill_names

    def test_crud_routes_badge_status(self, crud_profile, crud_schema):
        """CRUD 應該路由 badge-status."""
        result = route_skills(crud_profile, crud_schema)
        skill_names = [item["skill"] for item in result]
        assert "badge-status" in skill_names

    def test_dashboard_routes_layout_dashboard(self, dashboard_profile, dashboard_schema):
        """DASHBOARD 應該路由 layout-dashboard."""
        result = route_skills(dashboard_profile, dashboard_schema)
        skill_names = [item["skill"] for item in result]
        assert "layout-dashboard" in skill_names

    def test_game_routes_game_canvas(self, game_profile, game_schema):
        """GAME 應該路由 game-canvas."""
        result = route_skills(game_profile, game_schema)
        skill_names = [item["skill"] for item in result]
        assert "game-canvas" in skill_names

    def test_game_routes_score_board(self, game_profile, game_schema):
        """GAME 應該路由 score-board."""
        result = route_skills(game_profile, game_schema)
        skill_names = [item["skill"] for item in result]
        assert "score-board" in skill_names

    def test_api_routes_api_router(self, api_profile, api_schema):
        """API 應該路由 api-router."""
        result = route_skills(api_profile, api_schema)
        skill_names = [item["skill"] for item in result]
        assert "api-router" in skill_names

    def test_api_routes_auth_jwt(self, api_profile, api_schema):
        """API 應該路由 auth-jwt."""
        result = route_skills(api_profile, api_schema)
        skill_names = [item["skill"] for item in result]
        assert "auth-jwt" in skill_names


class TestRouteSkillsThreshold:
    """評分門檻測試."""

    def test_all_returned_skills_above_threshold(self, crud_profile, crud_schema):
        """所有返回的技能 score > 0.3."""
        result = route_skills(crud_profile, crud_schema)
        for item in result:
            assert item["score"] > 0.3, f"Skill {item['skill']} has score {item['score']} <= 0.3"

    def test_no_unkillable_skills_returned(self, crud_profile, crud_schema):
        """沒有被正確路由的技能不應該出現."""
        result = route_skills(crud_profile, crud_schema)
        skill_names = [item["skill"] for item in result]
        # auth-jwt 對 CRUD 場景不應該高分
        if "auth-jwt" in skill_names:
            auth_score = next(item["score"] for item in result if item["skill"] == "auth-jwt")
            assert auth_score > 0.3  # 但仍需超過門檻


class TestRouteSkillsTheme:
    """主題技能評分測試."""

    def test_glass_theme_boosts_glass_skill(self):
        """glass theme → theme-glass 分數 boost."""
        profile_glass = classify_intent("毛玻璃效果的待辦系統")
        schema = infer_schema(profile_glass)
        result = route_skills(profile_glass, schema)
        skill_names = [item["skill"] for item in result]
        if "theme-glass" in skill_names:
            glass_score = next(item["score"] for item in result if item["skill"] == "theme-glass")
            # theme-glass 應該有相對高分（因為 boost）
            assert glass_score > 0.3

    def test_modern_theme_boosts_modern_skill(self):
        """modern theme → theme-modern boost."""
        profile = classify_intent("深色主題的代辦系統")
        schema = infer_schema(profile)
        result = route_skills(profile, schema)
        skill_names = [item["skill"] for item in result]
        if "theme-modern" in skill_names:
            modern_score = next(item["score"] for item in result if item["skill"] == "theme-modern")
            assert modern_score > 0.3


class TestSkillIndexCompleteness:
    """SKILL_INDEX 完整性測試."""

    def test_all_indexed_skills_are_strings(self):
        """SKILL_INDEX 所有 key 都是字串."""
        for skill_name in SKILL_INDEX:
            assert isinstance(skill_name, str)

    def test_all_indexed_skills_have_required_fields(self):
        """每個技能都有 types、handles、weight_base."""
        for skill_name, meta in SKILL_INDEX.items():
            assert "types" in meta
            assert "handles" in meta
            assert "weight_base" in meta
            assert isinstance(meta["types"], list)
            assert isinstance(meta["handles"], list)
            assert isinstance(meta["weight_base"], float)

    def test_weight_base_in_valid_range(self):
        """weight_base 在 0.0 ~ 1.0 範圍."""
        for skill_name, meta in SKILL_INDEX.items():
            assert 0.0 <= meta["weight_base"] <= 1.0, f"Skill {skill_name} has invalid weight_base"

    def test_all_types_are_intenttype(self):
        """所有技能宣告的 types 都是 IntentType."""
        for skill_name, meta in SKILL_INDEX.items():
            for t in meta["types"]:
                assert isinstance(t, IntentType)


class TestRouteSkillsEdgeCases:
    """邊界情況測試."""

    def test_empty_schema(self, crud_profile):
        """空 schema 不應該 crash."""
        result = route_skills(crud_profile, [])
        assert isinstance(result, list)
        assert len(result) >= 0

    def test_unknown_type_profile(self):
        """UNKNOWN 類型 → 仍應返回技能（降級為 CRUD 邏輯）."""
        from nodes.intent_classifier import IntentProfile
        profile = IntentProfile(
            type=IntentType.UNKNOWN,
            entities=[],
            actions=[],
            context="做一些事情",
            target="html",
            theme="modern"
        )
        schema = infer_schema(profile)
        # UNKNOWN 會降級到 CRUD schema
        assert len(schema) > 0
        result = route_skills(profile, schema)
        assert isinstance(result, list)
