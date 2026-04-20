"""節點 5：Composer（組合器）"""
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
            if start != -1:
                end = content.find("[/", start)
                if end != -1:
                    return content[start + len(tag):end].strip()
            return ""
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
        '<div id="form-modal" class="modal-overlay" style="display:none">\n'
        '  <div class="modal-panel">\n'
        '    <div class="modal-header">\n'
        '      <h3 id="modal-title">新增項目</h3>\n'
        '      <button class="modal-close" onclick="closeModal()">x</button>\n'
        '    </div>\n'
        '    <form id="inventory-form" class="modal-form">\n'
        '      <input type="hidden" id="edit-id" />\n'
        + "\n".join(form_fields) + '\n'
        '      <div class="form-actions">\n'
        '        <button type="button" class="btn-cancel" onclick="closeModal()">取消</button>\n'
        '        <button type="submit" class="btn-primary">儲存</button>\n'
        '      </div>\n'
        '    </form>\n'
        '  </div>\n'
        '</div>'
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

    thead_html = "<thead><tr>" + "".join(thead_cells) + "</tr></thead>"

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

    return thead_html, "\n".join(render_cases)


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
    """Compose HTML output."""
    all_css = []
    theme_css = ""

    for skill in skills_used:
        css = load_skill_blocks(skill, "style")
        if css:
            if "theme-" in skill:
                theme_css = css
            else:
                all_css.append(css)

    form_html, open_add_js, open_edit_js, submit_data_js = _build_form_from_schema(schema)
    thead_html, render_cases_js = _build_dynamic_table_html(schema)

    default_sort = (schema[0]["name"] if schema else "id") + "-desc"

    app_js = (
        "<script>\n"
        "const STATE = {\n"
        "  items: [],\n"
        "  nextId: 1,\n"
        "  filter: { search: '', sort: '" + default_sort + "' },\n"
        "  deleteTarget: null,\n"
        "  editTarget: null,\n"
        "};\n"
        "\n"
        + open_add_js + "\n"
        "\n"
        + open_edit_js + "\n"
        "\n"
        "function closeModal() {\n"
        "  document.getElementById('form-modal').style.display = 'none';\n"
        "  STATE.editTarget = null;\n"
        "}\n"
        "\n"
        "function openDelete(id) {\n"
        "  STATE.deleteTarget = id;\n"
        "  const item = STATE.items.find(x => x.id === id);\n"
        "  document.getElementById('confirm-title').textContent = '確認刪除？';\n"
        "  document.getElementById('confirm-message').textContent = '此操作無法撤銷';\n"
        "  document.getElementById('form-modal').style.display = 'none';\n"
        "  document.getElementById('confirm-overlay').style.display = 'flex';\n"
        "}\n"
        "\n"
        "function closeConfirm() {\n"
        "  document.getElementById('confirm-overlay').style.display = 'none';\n"
        "  STATE.deleteTarget = null;\n"
        "}\n"
        "\n"
        "function doDelete() {\n"
        "  if (STATE.deleteTarget === null) return;\n"
        "  STATE.items = STATE.items.filter(x => x.id !== STATE.deleteTarget);\n"
        "  closeConfirm();\n"
        "  showToast('已刪除', 'info');\n"
        "  render();\n"
        "}\n"
        "\n"
        "function showToast(message, type) {\n"
        "  const c = document.getElementById('toast-container');\n"
        "  const t = document.createElement('div');\n"
        "  t.className = 'toast toast-' + (type || 'info');\n"
        "  t.textContent = message;\n"
        "  c.appendChild(t);\n"
        "  setTimeout(function() { t.remove(); }, 3000);\n"
        "}\n"
        "\n"
        "function toggleComplete(id) {\n"
        "  const item = STATE.items.find(x => x.id === id);\n"
        "  if (item) {\n"
        "    item.completed = !item.completed;\n"
        "    showToast(item.completed ? '已完成' : '已取消完成', item.completed ? 'success' : 'info');\n"
        "    render();\n"
        "  }\n"
        "}\n"
        "\n"
        "function _renderBadge(value, item) {\n"
        "  if (value === undefined || value === null) return '';\n"
        "  const v = String(value);\n"
        "  if (v === '高') return '<span class=\"badge badge-danger\">高</span>';\n"
        "  if (v === '中') return '<span class=\"badge badge-info\">中</span>';\n"
        "  if (v === '低') return '<span class=\"badge badge-ok\">低</span>';\n"
        "  return '<span class=\"badge\">' + v + '</span>';\n"
        "}\n"
        "\n"
        "function _renderActions(item) {\n"
        "  return '<button class=\"btn-edit\" onclick=\"openEdit(' + item.id + ')\">編輯</button>"
        "<button class=\"btn-delete\" onclick=\"openDelete(' + item.id + ')\">刪除</button>';\n"
        "}\n"
        "\n"
        "document.getElementById('inventory-form').addEventListener('submit', function(e) {\n"
        "  e.preventDefault();\n"
        "  const data = {\n"
        + submit_data_js + "\n"
        "  };\n"
        "  const editId = document.getElementById('edit-id').value;\n"
        "  if (editId) {\n"
        "    const idx = STATE.items.findIndex(x => x.id === parseInt(editId));\n"
        "    if (idx !== -1) STATE.items[idx] = Object.assign({}, STATE.items[idx], data);\n"
        "    showToast('已更新', 'success');\n"
        "  } else {\n"
        "    data.id = STATE.nextId++;\n"
        "    STATE.items.push(data);\n"
        "    showToast('已新增', 'success');\n"
        "  }\n"
        "  closeModal();\n"
        "  render();\n"
        "});\n"
        "\n"
        "function render() {\n"
        "  const tbody = document.querySelector('#data-table tbody');\n"
        "  const search = STATE.filter.search.toLowerCase();\n"
        "  let items = STATE.items.filter(function(item) {\n"
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
        "  tbody.innerHTML = items.map(function(item) {\n"
        "    var completed = item.completed ? ' todo-completed' : '';\n"
        "    var cells = [];\n"
        "    var schemaFields = " + _get_schema_json(schema) + ";\n"
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
        "}\n"
        "\n"
        "document.getElementById('search-input').addEventListener('input', function(e) {\n"
        "  STATE.filter.search = e.target.value;\n"
        "  render();\n"
        "});\n"
        "\n"
        "render();\n"
        "</script>"
    )

    page_css = (
        "<style>\n"
        + "\n\n".join(all_css) + "\n\n"
        + (theme_css + "\n\n" if theme_css else "")
        + ".todo-completed td { text-decoration: line-through; opacity: 0.6; }\n"
        + ".toast { padding: 10px 20px; border-radius: 8px; position: fixed; bottom: 20px; right: 20px; z-index: 1000; font-size: 14px; }\n"
        + ".toast-success { background: #22c55e; color: white; }\n"
        + ".toast-info { background: #3b82f6; color: white; }\n"
        + ".toast-error { background: #ef4444; color: white; }\n"
        + "#confirm-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: none; align-items: center; justify-content: center; z-index: 999; }\n"
        + ".confirm-panel { background: white; border-radius: 12px; padding: 32px; text-align: center; max-width: 400px; }\n"
        + ".confirm-actions { display: flex; gap: 12px; justify-content: center; margin-top: 24px; }\n"
        + ".btn-cancel { padding: 10px 20px; border-radius: 8px; border: 1px solid #e5e7eb; background: white; cursor: pointer; }\n"
        + ".btn-danger { padding: 10px 20px; border-radius: 8px; border: none; background: #ef4444; color: white; cursor: pointer; }\n"
        + ".todo-check { width: 18px; height: 18px; cursor: pointer; }\n"
        + ".col-checkbox { width: 40px; }\n"
        + ".col-actions { width: 120px; }\n"
        + ".col-date { width: 120px; }\n"
        + ".container { max-width: 900px; margin: 0 auto; padding: 20px; }\n"
        + ".app-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }\n"
        + ".search-bar { margin-bottom: 16px; }\n"
        + ".search-bar input { width: 100%; padding: 10px 16px; border: 1px solid #e5e7eb; border-radius: 8px; font-size: 14px; }\n"
        + ".table-container { overflow-x: auto; }\n"
        + "table { width: 100%; border-collapse: collapse; }\n"
        + "th, td { padding: 12px 16px; text-align: left; border-bottom: 1px solid #e5e7eb; }\n"
        + "th { font-weight: 600; color: #374151; }\n"
        + ".btn-primary { padding: 10px 20px; border-radius: 8px; border: none; background: #3b82f6; color: white; cursor: pointer; font-size: 14px; }\n"
        + ".btn-edit { padding: 6px 12px; border-radius: 6px; border: 1px solid #e5e7eb; background: white; cursor: pointer; margin-right: 4px; }\n"
        + ".btn-delete { padding: 6px 12px; border-radius: 6px; border: 1px solid #fca5a5; background: white; color: #ef4444; cursor: pointer; }\n"
        + ".badge { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 12px; }\n"
        + ".badge-danger { background: #fee2e2; color: #ef4444; }\n"
        + ".badge-info { background: #dbeafe; color: #3b82f6; }\n"
        + ".badge-ok { background: #dcfce7; color: #22c55e; }\n"
        + ".modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: none; align-items: center; justify-content: center; z-index: 998; }\n"
        + ".modal-panel { background: white; border-radius: 12px; padding: 32px; width: 90%; max-width: 500px; }\n"
        + ".modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }\n"
        + ".modal-close { background: none; border: none; font-size: 24px; cursor: pointer; }\n"
        + ".form-group { margin-bottom: 16px; }\n"
        + ".form-group label { display: block; margin-bottom: 4px; font-weight: 500; font-size: 14px; }\n"
        + ".form-group input, .form-group select { width: 100%; padding: 8px 12px; border: 1px solid #e5e7eb; border-radius: 6px; font-size: 14px; box-sizing: border-box; }\n"
        + ".form-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 20px; }\n"
        + ".form-actions .btn-cancel { background: white; border: 1px solid #e5e7eb; }\n"
        + ".form-actions .btn-primary { background: #3b82f6; color: white; }\n"
        + "</style>"
    )

    confirm_html = (
        '<div id="confirm-overlay">\n'
        '  <div class="confirm-panel">\n'
        '    <h3 id="confirm-title">確認刪除？</h3>\n'
        '    <p id="confirm-message">此操作無法撤銷</p>\n'
        '    <div class="confirm-actions">\n'
        '      <button class="btn-cancel" onclick="closeConfirm()">取消</button>\n'
        '      <button class="btn-danger" onclick="doDelete()">刪除</button>\n'
        '    </div>\n'
        '  </div>\n'
        '</div>'
    )

    toast_container = '<div id="toast-container"></div>'

    page_title = (profile.context or "應用程式")[:50]
    page_header = (profile.context or "應用程式")[:30]

    page = (
        "<!DOCTYPE html>\n"
        "<html lang='zh-TW'>\n"
        "<head>\n"
        "  <meta charset='UTF-8'>\n"
        "  <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n"
        "  <title>" + page_title + "</title>\n"
        + page_css + "\n"
        "</head>\n"
        "<body>\n"
        "  <div class='container'>\n"
        "    <header class='app-header'>\n"
        "      <h1>" + page_header + "</h1>\n"
        "      <button class='btn-primary' onclick='openAdd()'>+ 新增</button>\n"
        "    </header>\n"
        "    <div class='search-bar'>\n"
        "      <input type='text' id='search-input' placeholder='搜尋...' />\n"
        "    </div>\n"
        "    <div class='table-container'>\n"
        "      <table id='data-table'>" + thead_html + "<tbody></tbody></table>\n"
        "    </div>\n"
        "  </div>\n"
        + form_html + "\n"
        + confirm_html + "\n"
        + toast_container + "\n"
        + app_js + "\n"
        "</body>\n"
        "</html>"
    )

    return {
        "code": page,
        "warnings": warnings,
        "metadata": {"skills_used": skills_used, "schema": schema, "theme": profile.theme}
    }


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
