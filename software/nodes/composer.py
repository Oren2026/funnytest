"""節點 5：Composer（組合器）— Spec-aware version"""
import re
import json
from typing import List, Dict, Optional
from pathlib import Path
from dataclasses import dataclass, field


SKILLS_BASE = Path(__file__).parent.parent / "skills"


@dataclass
class SkillSpec:
    """一個 Skill 的完整 Spec 描述（從 .skill 檔案解析）"""
    name: str
    semantic_promise: str = ""          # Contract: 語義承諾（一句话）
    input_format: str = ""              # Contract: 輸入格式描述
    output_semantic: str = ""            # Contract: 輸出語義
    does: List[str] = field(default_factory=list)   # Contract: ✅ 做的清單
    does_not: List[str] = field(default_factory=list) # Contract: ❌ 不做的清單
    failure_signals: List[str] = field(default_factory=list)  # Contract: 失敗信號
    dependencies: List[str] = field(default_factory=list)   # Dependencies: 依賴
    optional_deps: List[str] = field(default_factory=list)  # Dependencies: 可選依賴
    excludes: List[str] = field(default_factory=list)      # Dependencies: 排斥
    slots_provides: List[str] = field(default_factory=list) # Slots: 提供這些 slot
    slots_consumes: List[str] = field(default_factory=list) # Slots: 需要這些 slot（由別人注入）
    boundary_layer: str = ""             # Boundaries: 系統邊界（presentation/business/data）
    is_stateful: bool = False            # Boundaries: 是否保有狀態
    # 原始程式碼區塊
    html: str = ""
    style: str = ""
    react: str = ""


def load_skill_spec(skill_name: str) -> Optional[SkillSpec]:
    """讀取並解析一個 .skill 檔案的 Spec 區塊（Contract / Dependencies / Slots / Boundaries）。"""
    for subdir in ["ui", "styles", "core", "algorithms", "structures", "system", "behaviors"]:
        skill_dir = SKILLS_BASE / subdir
        if not skill_dir.exists():
            continue
        matches = list(skill_dir.glob(skill_name + ".skill"))
        if not matches:
            continue
        content = matches[0].read_text()
        spec = _parse_spec_sections(skill_name, content)
        # 同時載入程式碼區塊
        spec.html = _extract_section(content, "html")
        spec.style = _extract_section(content, "style")
        spec.react = _extract_section(content, "react")
        return spec
    return None


def _parse_spec_sections(name: str, content: str) -> SkillSpec:
    """從 skill 檔案內容解析五個 Spec 區塊。"""
    spec = SkillSpec(name=name)

    # 解析 ## Contract
    contract_match = re.search(r"## Contract\n(.*?)(?=\n## |$)", content, re.DOTALL)
    if contract_match:
        contract_text = contract_match.group(1)
        # 語義承諾
        m = re.search(r"\*\*語義承諾\*\*[：:]\s*(.+)", contract_text)
        if m:
            spec.semantic_promise = m.group(1).strip()
        # 輸入格式（簡單取第一個 ```json 塊）
        m = re.search(r"```json\n(.*?)```", contract_text, re.DOTALL)
        if m:
            spec.input_format = m.group(1).strip()
        # 輸出語義
        m = re.search(r"\*\*輸出語義\*\*[：:]\s*(.+)", contract_text)
        if m:
            spec.output_semantic = m.group(1).strip()
        # ✅ 做
        for m in re.findall(r"✅\s*(.+)", contract_text):
            spec.does.append(m.strip())
        # ❌ 不做
        for m in re.findall(r"❌\s*(.+)", contract_text):
            spec.does_not.append(m.strip())
        # 失敗信號
        for m in re.findall(r"`([^`]+)`", contract_text):
            val = m.strip()
            if val and val not in spec.failure_signals:
                spec.failure_signals.append(val)

    # 解析 ## Dependencies
    deps_match = re.search(r"## Dependencies\n(.*?)(?=\n## |$)", content, re.DOTALL)
    if deps_match:
        deps_text = deps_match.group(1)
        for m in re.findall(r"\*\*依賴\*\*[：:]\s*(.+)", deps_text):
            spec.dependencies = [x.strip() for x in re.split(r"[,，]", m) if x.strip()]
        for m in re.findall(r"\*\*可選依賴\*\*[：:]\s*(.+)", deps_text):
            spec.optional_deps = [x.strip() for x in re.split(r"[,，]", m) if x.strip()]
        for m in re.findall(r"\*\*排斥\*\*[：:]\s*(.+)", deps_text):
            spec.excludes = [x.strip() for x in re.split(r"[,，]", m) if x.strip()]
        # 如果是「無」就保持空列表
        if spec.dependencies == ["無"]:
            spec.dependencies = []
        if spec.excludes == ["無"]:
            spec.excludes = []

    # 解析 ## Slots
    slots_match = re.search(r"## Slots\n(.*?)(?=\n## |$)", content, re.DOTALL)
    if slots_match:
        slots_text = slots_match.group(1)
        # slot:xxx 格式
        for m in re.findall(r"\*\*slot:(\w+)\*\*[：:]?\s*(.*)", slots_text):
            slot_name = m[0]
            spec.slots_provides.append(slot_name)
        # 也解析 <!-- slot:xxx --> 語法（舊格式兼容）
        for m in re.findall(r"<!--\s*slot:(\w+)\s*-->", content):
            if m not in spec.slots_provides:
                spec.slots_provides.append(m)
        # consumes: 由 parent 注入
        for m in re.findall(r"由\s+(\w+)\s+注入", slots_text):
            if m not in spec.slots_consumes:
                spec.slots_consumes.append(m)

    # 解析 ## Boundaries
    bound_match = re.search(r"## Boundaries\n(.*?)(?=\n## |$)", content, re.DOTALL)
    if bound_match:
        bound_text = bound_match.group(1)
        m = re.search(r"\*\*系統邊界\*\*[：:]\s*(.+)", bound_text)
        if m:
            spec.boundary_layer = m.group(1).strip()
        m = re.search(r"Stateless", bound_text)
        if not m:
            # 沒有明確說 Stateless，預設 Stateful
            spec.is_stateful = True

    return spec


def _extract_section(content: str, section: str) -> str:
    """從 skill 檔案內容提取 [section] 區塊。"""
    tag = "[" + section + "]"
    start = content.find(tag)
    if start == -1:
        tag = "[" + section.upper() + "]"
        start = content.find(tag)
    if start == -1:
        return ""
    end = content.find("\n[", start + 1)
    if end == -1:
        end = len(content)
    return content[start + len(tag):end].strip()


class SkillRegistry:
    """
    全域 Skill 註冊表，建立 slot → skill 的反向索引。
    讓 Composer 可以根據「誰能提供這個 slot」動態選擇，而不是硬編碼技能名。
    """

    _instance: Optional["SkillRegistry"] = None

    def __init__(self):
        self.by_name: Dict[str, SkillSpec] = {}
        self.slot_providers: Dict[str, List[str]] = {}  # slot_name → [skill_name, ...]
        self.initialized: bool = False

    @classmethod
    def get(cls) -> "SkillRegistry":
        if cls._instance is None:
            cls._instance = cls()
            cls._instance._build()
        return cls._instance

    def _build(self):
        """掃描 skills/ 目錄，建立註冊表。"""
        if self.initialized:
            return
        for subdir in ["ui", "styles", "core", "algorithms", "structures", "system", "behaviors"]:
            skill_dir = SKILLS_BASE / subdir
            if not skill_dir.exists():
                continue
            for skill_file in skill_dir.glob("*.skill"):
                spec = load_skill_spec(skill_file.stem)
                if spec is None:
                    continue
                self.by_name[spec.name] = spec
                for slot in spec.slots_provides:
                    if slot not in self.slot_providers:
                        self.slot_providers[slot] = []
                    self.slot_providers[slot].append(spec.name)
        self.initialized = True

    def find_skill_for_slot(self, slot_name: str, context: str = "") -> Optional[str]:
        """
        根據 slot 名稱和上下文語意，找到最適合提供這個 slot 的技能。
        目前實現：優先找名字包含 slot 相關關鍵字的 skill。
        進階實現：根據 SkillSpec.contract 語意匹配（預留擴展點）。
        """
        providers = self.slot_providers.get(slot_name, [])
        if not providers:
            return None
        # 簡單策略：名字包含 slot_name 的優先
        for p in providers:
            if slot_name in p:
                return p
        return providers[0] if providers else None

    def get_spec(self, skill_name: str) -> Optional[SkillSpec]:
        return self.by_name.get(skill_name)

    def get_slot_providers(self, slot_name: str) -> List[str]:
        return self.slot_providers.get(slot_name, [])

    def reload(self):
        """清除快取並重建（用於 skill 檔案變更後）。"""
        self._instance = None
        self.__init__()



def load_skill_blocks(skill_name: str, section: str = "html") -> str:
    """Load a specific [section] block from a skill file."""
    for subdir in ["ui", "styles", "core", "algorithms", "structures", "system", "behaviors"]:
        skill_dir = SKILLS_BASE / subdir
        if not skill_dir.exists():
            continue
        matches = list(skill_dir.glob(skill_name + ".skill"))
        if matches:
            content = matches[0].read_text()
            tag = "[" + section + "]"
            start = content.find(tag)
            if start == -1:
                tag = "[" + section.upper() + "]"
                start = content.find(tag)
            if start == -1:
                return ""
            # Section ends at next [section] marker or end of file
            end = content.find("\n[", start + 1)
            if end == -1:
                end = len(content)
            return content[start + len(tag):end].strip()
    return ""


def _build_form_from_schema(schema: List[Dict]) -> tuple:
    """Generate (form_html, open_add_js, open_edit_js, submit_data_js) from schema."""
    form_fields = []
    field_ids = []
    open_add_lines = []
    open_edit_lines = []
    submit_lines = []

    for col in schema:
        key = col["name"]
        label = col.get("label", key)
        col_type = col.get("type", "text")

        if col_type in ("action",):
            continue
        if not col.get("editable", True):
            continue

        field_ids.append("field-" + key)

        if col_type == "badge":
            options = col.get("options", ["高", "中", "低"])
            opts_lines = []
            for o in options:
                opts_lines.append("          <option value=\"" + o + "\">" + o + "</option>")
            opts_html = "".join(opts_lines)
            form_fields.append(
                "      <div class=\"form-group\">\n"
                "        <label>" + label + "</label>\n"
                "        <select id=\"field-" + key + "\">\n"
                "          <option value=\"\">請選擇</option>\n"
                + opts_html + "\n"
                "        </select>\n"
                "      </div>"
            )
            open_add_lines.append("  document.getElementById('field-" + key + "').value = '';")
            open_edit_lines.append("  document.getElementById('field-" + key + "').value = item." + key + " || '';")
            submit_lines.append("    " + key + ": document.getElementById('field-" + key + "').value,")
        elif col_type == "checkbox":
            form_fields.append(
                "      <div class=\"form-group checkbox-group\">\n"
                "        <label>" + label + "</label>\n"
                "        <input type=\"checkbox\" id=\"field-" + key + "\" />\n"
                "      </div>"
            )
            open_add_lines.append("  document.getElementById('field-" + key + "').checked = false;")
            open_edit_lines.append("  document.getElementById('field-" + key + "').checked = !!(item." + key + ");")
            submit_lines.append("    " + key + ": document.getElementById('field-" + key + "').checked,")
        elif col_type == "date":
            form_fields.append(
                "      <div class=\"form-group\">\n"
                "        <label>" + label + "</label>\n"
                "        <input type=\"date\" id=\"field-" + key + "\" />\n"
                "      </div>"
            )
            open_add_lines.append("  document.getElementById('field-" + key + "').value = '';")
            open_edit_lines.append("  document.getElementById('field-" + key + "').value = item." + key + " || '';")
            submit_lines.append("    " + key + ": document.getElementById('field-" + key + "').value,")
        else:
            placeholder = col.get("placeholder", "")
            placeholder_attr = ' placeholder=\"' + placeholder + '"' if placeholder else ""
            form_fields.append(
                "      <div class=\"form-group\">\n"
                "        <label>" + label + "</label>\n"
                "        <input type=\"text\" id=\"field-" + key + "\"" + placeholder_attr + " />\n"
                "      </div>"
            )
            open_add_lines.append("  document.getElementById('field-" + key + "').value = '';")
            open_edit_lines.append("  document.getElementById('field-" + key + "').value = item." + key + " || '';")
            submit_lines.append("    " + key + ": document.getElementById('field-" + key + "').value.trim(),")

    form_html = (
        '      <input type="hidden" id="edit-id" />\n'
        + "\n".join(form_fields) + '\n'
        '      <div class="form-actions">\n'
        '        <button type="button" class="btn-cancel" onclick="closeModal()">取消</button>\n'
        '        <button type="submit" class="btn-primary">儲存</button>\n'
        '      </div>'
    )

    open_add_js = (
        "function openAdd() {\n"
        "  STATE.editTarget = null;\n"
        "  document.getElementById('modal-title').textContent = '新增項目';\n"
        "  document.getElementById('edit-id').value = '';\n"
        + "\n".join(open_add_lines) + "\n"
        "  document.getElementById('form-modal').style.display = 'flex';\n"
        "}"
    )

    open_edit_js = (
        "function openEdit(id) {\n"
        "  const item = STATE.items.find(x => x.id === id);\n"
        "  STATE.editTarget = id;\n"
        "  document.getElementById('modal-title').textContent = '編輯項目';\n"
        "  document.getElementById('edit-id').value = id;\n"
        + "\n".join(open_edit_lines) + "\n"
        "  document.getElementById('form-modal').style.display = 'flex';\n"
        "}"
    )

    submit_data_js = "\n".join(submit_lines)

    return form_html, open_add_js, open_edit_js, submit_data_js


def _build_dynamic_table_html(schema: List[Dict]) -> tuple:
    """Build (thead_html, render_cases_js) from schema."""
    thead_cells = []
    for col in schema:
        label = col.get("label", col["name"])
        col_type = col.get("type", "text")
        if col_type == "checkbox":
            thead_cells.append('<th class="col-checkbox"></th>')
        elif col_type == "action":
            thead_cells.append('<th class="col-actions">' + label + '</th>')
        elif col_type == "sortable":
            thead_cells.append('<th class="sortable" data-sort="' + col["name"] + '">' + label + ' |</th>')
        else:
            thead_cells.append('<th>' + label + '</th>')

    thead_rows_html = "<tr>" + "".join(thead_cells) + "</tr>"

    render_cases = []
    for col in schema:
        key = col["name"]
        col_type = col.get("type", "text")
        if col_type == "checkbox":
            case_str = (
                '    case "' + key + '": return `<input type="checkbox" '
                'onchange="toggleComplete(${item.id})" '
                '${item.' + key + ' ? "checked" : ""} class="todo-check">`;'
            )
            render_cases.append(case_str)
        elif col_type == "badge":
            render_cases.append('    case "' + key + '": return _renderBadge(item.' + key + ', item);')
        elif col_type == "action":
            render_cases.append('    case "' + key + '": return _renderActions(item);')
        elif col_type == "date":
            render_cases.append('    case "' + key + '": return `<td class="col-date">${item.' + key + '}</td>`;')
        elif col_type == "sortable":
            render_cases.append('    case "' + key + '": return `<td class="col-sortable">${item.' + key + '}</td>`;')
        else:
            render_cases.append('    case "' + key + '": return `<td>${item.' + key + '}</td>`;')

    return thead_rows_html, "\n".join(render_cases)


def compose_output(
    ordered_skills: List[Dict],
    schema: List[Dict],
    profile,
    output_type: str,
) -> Dict:
    """Compose HTML or React output from ordered skills + schema."""
    warnings = []
    skills_used = [s["skill"] for s in ordered_skills]

    if output_type == "html":
        return _compose_html(skills_used, schema, profile, warnings)
    elif output_type == "react":
        return _compose_react(skills_used, schema, profile, warnings)
    else:
        warnings.append("Unsupported output type: " + output_type + ", falling back to HTML")
        return _compose_html(skills_used, schema, profile, warnings)


def _compose_html(skills_used, schema, profile, warnings) -> Dict:
    """
    Compose HTML output by loading skill blocks and injecting schema-driven content.

    Spec-aware enhancement:
    - Uses SkillRegistry to find which skill provides each slot
    - Falls back to hardcoded names for backward compatibility
    - Emits warnings when using fallback (indicating missing Spec declarations)
    """
    registry = SkillRegistry.get()

    # --- Load CSS from all skills ---
    all_css = []
    theme_css = ""
    for skill in skills_used:
        css = load_skill_blocks(skill, "style")
        if css:
            if "theme-" in skill:
                theme_css = css
            else:
                all_css.append(css)

    # --- Spec-aware slot → skill resolution ---
    # Each (slot_name, fallback_skill_name) pair:
    # 1. Try registry.find_skill_for_slot(slot_name)
    # 2. Fall back to fallback_skill_name if not found or not Spec-formatted
    slot_map = [
        ("header",   "layout-header"),
        ("search",   "search-bar"),
        ("content",  "table-data"),
        ("modal",    "modal-form"),
        ("confirm",  "confirm-dialog"),
        ("toast",    "toast-notify"),
    ]

    loaded_slots = {}
    for slot_name, fallback_skill in slot_map:
        skill_name = registry.find_skill_for_slot(slot_name)
        if skill_name is None:
            # No Spec declaration → use fallback (with warning)
            skill_name = fallback_skill
            if profile and profile.context:
                warnings.append(f"[Spec-aware] slot '{slot_name}': no Spec declaration found, falling back to '{skill_name}'")
        else:
            if skill_name != fallback_skill:
                warnings.append(f"[Spec-aware] slot '{slot_name}': resolved to '{skill_name}' (Spec override)")
        html = load_skill_blocks(skill_name, "html")
        loaded_slots[slot_name] = html

    header_html   = loaded_slots.get("header",   "")
    search_html   = loaded_slots.get("search",   "")
    table_html    = loaded_slots.get("content", "")
    modal_html    = loaded_slots.get("modal",    "")
    confirm_html  = loaded_slots.get("confirm",  "")
    toast_html    = loaded_slots.get("toast",    "")
    page_layout   = load_skill_blocks("layout-page", "html")

    # --- Schema-driven content generation ---
    form_html, open_add_js, open_edit_js, submit_data_js = _build_form_from_schema(schema)
    thead_html, render_cases_js = _build_dynamic_table_html(schema)

    # --- Inject form fields into modal-form ---
    if modal_html and form_html:
        modal_html = _inject_slot(modal_html, "form-fields", form_html)

    # --- Build action buttons (injected into header slot) ---
    entity_name = (profile.entities[0] if profile.entities else "應用程式")
    page_title = entity_name + " - " + (profile.context or "")[:40]
    page_header = entity_name
    add_button = f'<button class="btn-primary" onclick="openAdd()">+ 新增</button>'

    # --- Build thead with schema (before injecting into page) ---
    table_with_schema = table_html or _fallback_table()
    table_with_schema = _inject_slot(table_with_schema, "thead", thead_html)

    # --- Inject into layout-page slots ---
    page_html = page_layout or _fallback_page_layout()
    page_html = _inject_slot(page_html, "header", header_html)
    page_html = _inject_slot(page_html, "search", search_html)
    page_html = _inject_slot(page_html, "content", table_with_schema)
    page_html = _inject_slot(page_html, "modal", modal_html or _fallback_modal())
    page_html = _inject_slot(page_html, "confirm", confirm_html or _fallback_confirm())
    page_html = _inject_slot(page_html, "toast", toast_html or _fallback_toast())

    # --- Inject header actions slot ---
    page_html = _inject_slot(page_html, "actions", add_button)

    # --- Inject page title into header ---
    # Replace in span content (handle both warehouse title and any existing title)
    page_html = re.sub(
        r'(<span class="header-title">)[^<]*(</span>)',
        r'\1📦 ' + page_header + r'\2',
        page_html
    )
    # Fallback: text replacement
    page_html = page_html.replace("倉儲管理系統", page_header)
    page_html = page_html.replace("📦 倉儲管理系統", "📦 " + page_header)

    # --- Build search input JS hook ---
    search_input_js = (
        "document.getElementById('search-input').addEventListener('input', function(e) {\n"
        "  STATE.filter.search = e.target.value;\n"
        "  render();\n"
        "});\n"
    )

    # --- Default sort ---
    default_sort = (schema[0]["name"] if schema else "id") + "-desc"

    # --- Full app JS ---
    storage_key = "evcompiler_" + page_title.replace(" ", "_").lower()[:20]
    app_js = (
        "<script>\n"
        "var STATE_KEY = '" + storage_key + "';\n"
        "function saveState() {\n"
        "  try { localStorage.setItem(STATE_KEY, JSON.stringify({ items: STATE.items, nextId: STATE.nextId })); } catch(e) {}\n"
        "}\n"
        "function loadState() {\n"
        "  try {\n"
        "    var saved = localStorage.getItem(STATE_KEY);\n"
        "    if (saved) {\n"
        "      var data = JSON.parse(saved);\n"
        "      STATE.items = data.items || [];\n"
        "      STATE.nextId = data.nextId || 1;\n"
        "    }\n"
        "  } catch(e) {}\n"
        "}\n"
        "const STATE = {\n"
        "  items: [],\n"
        "  nextId: 1,\n"
        "  filter: { search: '', sort: '" + default_sort + "' },\n"
        "  deleteTarget: null,\n"
        "  editTarget: null,\n"
        "};\n"
        "loadState();\n"
        "\n"
        + open_add_js + "\n\n"
        + open_edit_js + "\n\n"
        "function closeModal() {\n"
        "  var m = document.getElementById('form-modal');\n"
        "  if (m) m.style.display = 'none';\n"
        "  STATE.editTarget = null;\n"
        "}\n\n"
        "function openDelete(id) {\n"
        "  STATE.deleteTarget = id;\n"
        "  var confirmEl = document.getElementById('confirm-overlay');\n"
        "  if (confirmEl) {\n"
        "    var titleEl = document.getElementById('confirm-title');\n"
        "    var msgEl = document.getElementById('confirm-message');\n"
        "    if (titleEl) titleEl.textContent = '確認刪除？';\n"
        "    if (msgEl) msgEl.textContent = '此操作無法撤銷';\n"
        "    confirmEl.style.display = 'flex';\n"
        "  }\n"
        "  var modalEl = document.getElementById('form-modal');\n"
        "  if (modalEl) modalEl.style.display = 'none';\n"
        "}\n\n"
        "function closeConfirm() {\n"
        "  var confirmEl = document.getElementById('confirm-overlay');\n"
        "  if (confirmEl) confirmEl.style.display = 'none';\n"
        "  STATE.deleteTarget = null;\n"
        "}\n\n"
        "function doDelete() {\n"
        "  if (STATE.deleteTarget === null) return;\n"
        "  STATE.items = STATE.items.filter(function(x) { return x.id !== STATE.deleteTarget; });\n"
        "  saveState();\n"
        "  closeConfirm();\n"
        "  showToast('已刪除', 'info');\n"
        "  render();\n"
        "}\n\n"
        "function showToast(message, type) {\n"
        "  var c = document.getElementById('toast-container');\n"
        "  if (!c) return;\n"
        "  var t = document.createElement('div');\n"
        "  t.className = 'toast toast-' + (type || 'info');\n"
        "  t.textContent = message;\n"
        "  c.appendChild(t);\n"
        "  setTimeout(function() { t.remove(); }, 3000);\n"
        "}\n\n"
        "function _renderBadge(value, item) {\n"
        "  if (value === undefined || value === null) return '';\n"
        "  var v = String(value);\n"
        "  if (v === '高') return '<span class=\"badge badge-danger\">高</span>';\n"
        "  if (v === '中') return '<span class=\"badge badge-info\">中</span>';\n"
        "  if (v === '低') return '<span class=\"badge badge-ok\">低</span>';\n"
        "  return '<span class=\"badge\">' + v + '</span>';\n"
        "}\n\n"
        "function _renderActions(item) {\n"
        "  var done = item.completed ? '✓' : '○';\n"
        "  var doneClass = item.completed ? 'btn-done-active' : 'btn-done';\n"
        "  return '<button class=\"' + doneClass + '\" onclick=\"toggleComplete(' + item.id + ')\">' + done + '</button>'\n"
        "    + '<button class=\"btn-edit\" onclick=\"openEdit(' + item.id + ')\">編輯</button>'\n"
        "    + '<button class=\"btn-delete\" onclick=\"openDelete(' + item.id + ')\">刪除</button>';\n"
        "}\n\n"
        "function toggleComplete(id) {\n"
        "  var item = STATE.items.find(function(x) { return x.id === id; });\n"
        "  if (item) {\n"
        "    item.completed = !item.completed;\n"
        "    showToast(item.completed ? '已完成' : '已取消完成', item.completed ? 'success' : 'info');\n"
        "    render();\n"
        "  }\n"
        "}\n\n"
        "document.getElementById('inventory-form').addEventListener('submit', function(e) {\n"
        "  e.preventDefault();\n"
        "  var data = {\n"
        + submit_data_js + "\n"
        "  };\n"
        "  var editId = document.getElementById('edit-id').value;\n"
        "  if (editId) {\n"
        "    var idx = STATE.items.findIndex(function(x) { return x.id === parseInt(editId); });\n"
        "    if (idx !== -1) {\n"
        "      data.updatedAt = new Date().toLocaleDateString('zh-TW');\n"
        "      STATE.items[idx] = Object.assign({}, STATE.items[idx], data);\n"
        "    }\n"
        "    showToast('已更新', 'success');\n"
        "  } else {\n"
        "    data.id = STATE.nextId++;\n"
        "    data.createdAt = new Date().toLocaleDateString('zh-TW');\n"
        "    data.updatedAt = new Date().toLocaleDateString('zh-TW');\n"
        "    STATE.items.push(data);\n"
        "    showToast('已新增', 'success');\n"
        "  }\n"
        "  saveState();\n"
        "  closeModal();\n"
        "  render();\n"
        "});\n\n"
        "function render() {\n"
        "  var tbody = document.querySelector('.inventory-table tbody');\n"
        "  if (!tbody) return;\n"
        "  var search = STATE.filter.search.toLowerCase();\n"
        "  var items = STATE.items.filter(function(item) {\n"
        "    if (!search) return true;\n"
        "    return Object.values(item).some(function(v) { return String(v).toLowerCase().includes(search); });\n"
        "  });\n"
        "  items.sort(function(a, b) {\n"
        "    var parts = STATE.filter.sort.split('-');\n"
        "    var field = parts[0];\n"
        "    var dir = parts[1] || 'asc';\n"
        "    var va = a[field] || '';\n"
        "    var vb = b[field] || '';\n"
        "    var cmp = va < vb ? -1 : va > vb ? 1 : 0;\n"
        "    return dir === 'asc' ? cmp : -cmp;\n"
        "  });\n"
        "  var schemaFields = " + _get_schema_json(schema) + ";\n"
        "  tbody.innerHTML = items.map(function(item) {\n"
        "    var completed = item.completed ? ' todo-completed' : '';\n"
        "    var cells = [];\n"
        "    for (var i = 0; i < schemaFields.length; i++) {\n"
        "      var col = schemaFields[i];\n"
        "      var val = '';\n"
        "      switch (col.name) {\n"
        + render_cases_js + "\n"
        "      }\n"
        "      cells.push(val);\n"
        "    }\n"
        "    return '<tr class=\"' + completed + '\">' + cells.join('') + '</tr>';\n"
        "  }).join('');\n"
        "}\n\n"
        + search_input_js + "\n"
        "render();\n"
        "</script>\n"
    )

    # --- Assemble page ---
    page = (
        "<!DOCTYPE html>\n"
        "<html lang='zh-TW'>\n"
        "<head>\n"
        "  <meta charset='UTF-8'>\n"
        "  <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n"
        "  <title>" + page_title + "</title>\n"
        "  <style>\n"
        + "\n\n".join(all_css) + "\n\n"
        + (theme_css + "\n\n" if theme_css else "")
        + ".todo-completed td { text-decoration: line-through; opacity: 0.6; }\n"
        + ".toast { padding: 10px 20px; border-radius: 8px; position: fixed; bottom: 20px; right: 20px; z-index: 1000; font-size: 14px; }\n"
        + ".toast-success { background: #22c55e; color: white; }\n"
        + ".toast-info { background: #3b82f6; color: white; }\n"
        + ".toast-error { background: #ef4444; color: white; }\n"
        + ".btn-done, .btn-done-active { display: inline-flex; align-items: center; justify-content: center; width: 32px; height: 32px; border-radius: 50%; border: 1px solid #d1d5db; background: #fff; color: #9ca3af; font-size: 16px; cursor: pointer; transition: all 0.15s; }\n"
        + ".btn-done:hover { background: #f0fdf4; border-color: #86efac; color: #22c55e; }\n"
        + ".btn-done-active { background: #22c55e; border-color: #22c55e; color: #fff; }\n"
        + ".btn-done-active:hover { background: #16a34a; border-color: #16a34a; }\n"
        + "  </style>\n"
        "</head>\n"
        "<body>\n"
        + page_html + "\n"
        + app_js
        + "</body>\n"
        + "</html>\n"
    )

    return {
        "code": page,
        "warnings": warnings,
        "metadata": {"skills_used": skills_used, "schema": schema, "theme": profile.theme}
    }


def _inject_slot(html: str, slot_name: str, content: str) -> str:
    """Replace <!-- slot:NAME --> comment marker with content."""
    marker = f'<!-- slot:{slot_name} -->'
    if marker in html:
        return html.replace(marker, content)
    # Fallback: look for data-slot attribute
    pattern = r'(<[^>]*\sdata-slot="' + re.escape(slot_name) + r'"[^>]*>)'
    result, n = re.subn(pattern, r'\1' + content, html)
    if n:
        return result
    return html


def _inject_form_into_modal(modal_html: str, form_html: str) -> str:
    """Replace the inner content of <form> inside modal-form with schema-driven form fields."""
    # Replace ONLY the inner content of <form>...</form>, preserving the form tags
    pattern = r'(<form[^>]*>)(.*?)(</form>)'
    replacement = r'\1\n' + form_html + r'\n\3'
    result, n = re.subn(pattern, replacement, modal_html, flags=re.DOTALL)
    if n:
        return result
    # Fallback: just append
    return modal_html + "\n" + form_html


def _fallback_page_layout() -> str:
    return (
        "<div class='container'>\n"
        "  <div data-slot='header'></div>\n"
        "  <div data-slot='search'></div>\n"
        "  <div data-slot='content'></div>\n"
        "  <div data-slot='modal'></div>\n"
        "  <div data-slot='confirm'></div>\n"
        "  <div data-slot='toast'></div>\n"
        "</div>\n"
    )


def _fallback_modal() -> str:
    return (
        "<div id='form-modal' class='modal-overlay' style='display:none'>\n"
        "  <div class='modal-panel'>\n"
        "    <div class='modal-header'>\n"
        "      <h3 id='modal-title'>新增項目</h3>\n"
        "      <button class='modal-close' onclick='closeModal()'>×</button>\n"
        "    </div>\n"
        "    <form id='inventory-form' class='modal-form'></form>\n"
        "  </div>\n"
        "</div>\n"
    )


def _fallback_confirm() -> str:
    return (
        "<div id='confirm-overlay' style='display:none'>\n"
        "  <div class='confirm-panel'>\n"
        "    <h3 id='confirm-title'>確認刪除？</h3>\n"
        "    <p id='confirm-message'>此操作無法撤銷</p>\n"
        "    <div class='confirm-actions'>\n"
        "      <button class='btn-cancel' onclick='closeConfirm()'>取消</button>\n"
        "      <button class='btn-danger' onclick='doDelete()'>刪除</button>\n"
        "    </div>\n"
        "  </div>\n"
        "</div>\n"
    )


def _fallback_toast() -> str:
    return "<div id='toast-container'></div>"


def _fallback_table() -> str:
    return (
        "<div class='table-container'>\n"
        "  <table id='data-table'>\n"
        "    <thead data-slot='thead'></thead>\n"
        "    <tbody></tbody>\n"
        "  </table>\n"
        "</div>\n"
    )

def _get_schema_json(schema):
    """Convert schema list to JS array literal."""
    items = []
    for f in schema:
        items.append('{ name: "' + f["name"] + '", type: "' + f.get("type", "text") + '" }')
    return "[" + ", ".join(items) + "]"


def _compose_react(skills_used, schema, profile, warnings) -> Dict:
    """Compose React output (stub)."""
    try:
        from pathlib import Path
        import sys
        sys.path.insert(0, str(Path(__file__).parent.parent))
        from compiler.react_compiler import compile_react
        code = compile_react(skills_used, {"schema": schema, "theme": profile.theme, "context": profile.context})
        return {"code": code, "warnings": warnings, "metadata": {"skills_used": skills_used, "schema": schema, "theme": profile.theme}}
    except Exception as e:
        warnings.append("React compilation failed, falling back to HTML: " + str(e))
        return _compose_html(skills_used, schema, profile, warnings)
