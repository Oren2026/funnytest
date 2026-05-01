"""節點 1：Intent Classifier 單元測試"""
import pytest
from nodes.intent_classifier import (
    classify_intent, IntentProfile, IntentType,
    TYPE_PATTERNS, ENTITY_PATTERNS, ACTION_PATTERNS
)


class TestIntentTypeDetection:
    """IntentType 分類邏輯測試."""

    def test_game_type_by_keyword(self):
        profile = classify_intent("做一個貪吃蛇遊戲")
        assert profile.type == IntentType.GAME

    def test_game_type_english(self):
        profile = classify_intent("build a 2048 game with canvas")
        assert profile.type == IntentType.GAME

    def test_dashboard_type(self):
        profile = classify_intent("建立一個數據儀表板，顯示統計資訊")
        assert profile.type == IntentType.DASHBOARD

    def test_dashboard_type_english(self):
        profile = classify_intent("create dashboard with analytics")
        assert profile.type == IntentType.DASHBOARD

    def test_api_type(self):
        profile = classify_intent("REST API 認證服務，JWT 登入")
        assert profile.type == IntentType.API

    def test_api_type_english(self):
        profile = classify_intent("build a JWT auth API")
        assert profile.type == IntentType.API

    def test_tool_type_timer(self):
        profile = classify_intent("做一個倒數計時器")
        assert profile.type == IntentType.TOOL

    def test_tool_type_calculator(self):
        profile = classify_intent("單位換算計算機")
        assert profile.type == IntentType.TOOL

    def test_crud_type_todo(self):
        profile = classify_intent("待辦事項管理系統")
        assert profile.type == IntentType.CRUD

    def test_crud_type_inventory(self):
        profile = classify_intent("庫存管理系統")
        assert profile.type == IntentType.CRUD

    def test_crud_type_customer(self):
        profile = classify_intent("客戶管理系統")
        assert profile.type == IntentType.CRUD

    def test_unknown_promotes_to_crud_on_entity(self):
        """UNKNOWN 但有 CRUD 實體關鍵詞 → 提升為 CRUD."""
        # "使用者" 是 CRUD 實體關鍵詞，但不觸發任何 TYPE_PATTERNS → UNKNOWN → 提升為 CRUD
        profile = classify_intent("處理使用者資料")
        assert profile.type == IntentType.CRUD
        assert len(profile.entities) > 0


class TestEntityExtraction:
    """實體關鍵詞提取測試."""

    def test_single_entity_todo(self):
        profile = classify_intent("代辦事項管理系統")
        assert "任務" in profile.entities

    def test_single_entity_inventory(self):
        profile = classify_intent("庫存管理系統")
        assert "庫存" in profile.entities

    def test_single_entity_book(self):
        profile = classify_intent("書籍管理系統")
        assert "書籍" in profile.entities

    def test_multiple_entities(self):
        """多個實體關鍵詞出現時，應該全部提取."""
        profile = classify_intent("代辦事項與客戶管理系統")
        assert "任務" in profile.entities
        assert "客戶" in profile.entities

    def test_unknown_no_entity(self):
        """完全無法識別 → UNKNOWN 且無實體."""
        profile = classify_intent("asdfghjkl qwerty")
        assert profile.type == IntentType.UNKNOWN
        assert len(profile.entities) == 0

    def test_entity_not_duplicated(self):
        """同一 entity 關鍵詞只出現一次."""
        profile = classify_intent("待辦事項")
        assert profile.entities.count("任務") == 1


class TestActionExtraction:
    """動作關鍵詞提取測試."""

    def test_action_add(self):
        profile = classify_intent("新增代辦事項")
        assert "新增" in profile.actions

    def test_action_delete(self):
        profile = classify_intent("刪除商品")
        assert "刪除" in profile.actions

    def test_action_edit(self):
        profile = classify_intent("編輯任務")
        assert "編輯" in profile.actions

    def test_action_search(self):
        profile = classify_intent("搜尋客戶資料")
        assert "查詢" in profile.actions

    def test_multiple_actions(self):
        profile = classify_intent("新增、刪除、編輯代辦事項")
        assert "新增" in profile.actions
        assert "刪除" in profile.actions
        assert "編輯" in profile.actions


class TestTargetInference:
    """輸出平台推斷測試."""

    def test_target_html_default(self):
        profile = classify_intent("待辦事項系統")
        assert profile.target == "html"

    def test_target_react(self):
        profile = classify_intent("React 待辦事項系統")
        assert profile.target == "react"

    def test_target_flutter_ios(self):
        profile = classify_intent("iOS app 待辦事項")
        assert profile.target == "flutter"

    def test_target_flutter_android(self):
        profile = classify_intent("android 應用程式")
        assert profile.target == "flutter"

    def test_target_swiftui(self):
        profile = classify_intent("SwiftUI 應用程式")
        assert profile.target == "swift"


class TestThemeInference:
    """主題推斷測試."""

    def test_theme_glass(self):
        profile = classify_intent("毛玻璃效果的待辦系統")
        assert profile.theme == "glass"

    def test_theme_glass_english(self):
        profile = classify_intent("glass morphism UI")
        assert profile.theme == "glass"

    def test_theme_modern_default(self):
        profile = classify_intent("待辦事項")
        assert profile.theme == "modern"

    def test_theme_modern_dark(self):
        profile = classify_intent("深色主題的代辦系統")
        assert profile.theme == "modern"

    def test_theme_soft_light(self):
        profile = classify_intent("淺色主題代辦系統")
        assert profile.theme == "soft"

    def test_theme_soft_soft(self):
        profile = classify_intent("soft UI 代辦")
        assert profile.theme == "soft"

    def test_theme_brutal(self):
        profile = classify_intent("brutal 風格介面")
        assert profile.theme == "brutal"


class TestIntentProfileStructure:
    """IntentProfile 結構測試."""

    def test_profile_has_all_fields(self):
        profile = classify_intent("React 深色主題代辦系統")
        assert hasattr(profile, "type")
        assert hasattr(profile, "entities")
        assert hasattr(profile, "actions")
        assert hasattr(profile, "context")
        assert hasattr(profile, "target")
        assert hasattr(profile, "theme")

    def test_context_preserved(self):
        text = "代辦事項管理系統"
        profile = classify_intent(text)
        assert profile.context == text

    def test_entities_is_list(self):
        profile = classify_intent("庫存管理")
        assert isinstance(profile.entities, list)

    def test_actions_is_list(self):
        profile = classify_intent("新增刪除商品")
        assert isinstance(profile.actions, list)

    def test_type_is_intenttype_enum(self):
        profile = classify_intent("貪吃蛇遊戲")
        assert isinstance(profile.type, IntentType)
