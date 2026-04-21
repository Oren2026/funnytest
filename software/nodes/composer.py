"""節點 5：Composer（組合器）"""
import re
from typing import List, Dict
from pathlib import Path


SKILLS_BASE = Path(__file__).parent.parent / "skills"


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
    """Compose HTML output by loading skill blocks and injecting schema-driven content."""
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

    # --- Load [html] blocks from key skills ---
    header_html   = load_skill_blocks("layout-header", "html")
    search_html   = load_skill_blocks("search-bar", "html")
    table_html    = load_skill_blocks("table-data", "html")
    modal_html    = load_skill_blocks("modal-form", "html")
    confirm_html  = load_skill_blocks("confirm-dialog", "html")
    toast_html    = load_skill_blocks("toast-notify", "html")
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
        "  return '<button class=\"btn-edit\" onclick=\"openEdit(' + item.id + ')\">編輯</button>'\n"
        "    + '<button class=\"btn-delete\" onclick=\"openDelete(' + item.id + ')\">刪除</button>';\n"
        "}\n\n"
        "document.getElementById('inventory-form').addEventListener('submit', function(e) {\n"
        "  e.preventDefault();\n"
        "  var data = {\n"
        + submit_data_js + "\n"
        "  };\n"
        "  var editId = document.getElementById('edit-id').value;\n"
        "  if (editId) {\n"
        "    var idx = STATE.items.findIndex(function(x) { return x.id === parseInt(editId); });\n"
        "    if (idx !== -1) STATE.items[idx] = Object.assign({}, STATE.items[idx], data);\n"
        "    showToast('已更新', 'success');\n"
        "  } else {\n"
        "    data.id = STATE.nextId++;\n"
        "    STATE.items.push(data);\n"
        "    showToast('已新增', 'success');\n"
        "  }\n"
        "  saveState();\n"
        "  closeModal();\n"
        "  render();\n"
        "});\n\n"
        "function render() {\n"
        "  var tbody = document.querySelector('#data-table tbody');\n"
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
