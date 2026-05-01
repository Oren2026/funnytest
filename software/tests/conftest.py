"""Shared fixtures for node unit tests."""
import sys
from pathlib import Path

# Ensure software/ is on path
SOFTWARE_DIR = Path(__file__).parent.parent
sys.path.insert(0, str(SOFTWARE_DIR))

import pytest
from nodes.intent_classifier import (
    IntentProfile, IntentType, classify_intent
)
from nodes.schema_inferrer import infer_schema


@pytest.fixture
def crud_profile():
    """Basic CRUD profile for 代辦事項."""
    return classify_intent("代辦事項管理系統，包含任務名稱、優先權、截止日期")


@pytest.fixture
def dashboard_profile():
    """Dashboard profile."""
    return classify_intent("數據儀表板，顯示統計圖表")


@pytest.fixture
def game_profile():
    """Game profile."""
    return classify_intent("做一個貪吃蛇遊戲")


@pytest.fixture
def tool_profile():
    """Tool profile."""
    return classify_intent("倒數計時器")


@pytest.fixture
def api_profile():
    """API profile."""
    return classify_intent("REST API 認證服務，JWT 登入")


@pytest.fixture
def crud_schema(crud_profile):
    """CRUD schema from CRUD profile."""
    return infer_schema(crud_profile)


@pytest.fixture
def dashboard_schema(dashboard_profile):
    """Dashboard schema from dashboard profile."""
    return infer_schema(dashboard_profile)


@pytest.fixture
def game_schema(game_profile):
    """Game schema from game profile."""
    return infer_schema(game_profile)


@pytest.fixture
def tool_schema(tool_profile):
    """Tool schema from tool profile."""
    return infer_schema(tool_profile)


@pytest.fixture
def api_schema(api_profile):
    """API schema from API profile."""
    return infer_schema(api_profile)
