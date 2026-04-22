"""
html_compiler.py — HTML/CSS/JS 編譯器

接收元件樹，輸出完整可運行的 HTML 頁面。
"""

from typing import List, Dict

from pathlib import Path
SKILLS_BASE = Path(__file__).parent.parent / "skills"


def _build_dynamic_table_html(schema: List[Dict]) -> tuple:
    """
    Build dynamic <thead> and render() JS from a column schema.

    Each schema entry: { key, label, type }
    Supported types: checkbox | text | badge | action | date

    Returns (thead_html, render_fn_js).
    """
    # Build <thead> cells
    th_cells = []
    for col in schema:
        th_cells.append(f"      <th>{col['label']}</th>")
    thead_html = "      <thead>\n        <tr>\n" + "\n".join(th_cells) + "\n        </tr>\n      </thead>"

    # Build column-span for empty state
    col_count = len(schema)

    # Build render() function dynamically
    render_body_lines = [
        "  let items = STATE.items.filter(item => {",
        "    const matchSearch = !STATE.filter.search ||",
        "      Object.values(item).join(' ').toLowerCase().includes(STATE.filter.search.toLowerCase());",
        "    const matchCat = !STATE.filter.category || item.category === STATE.filter.category;",
        "    return matchSearch && matchCat;",
        "  });",
        "",
        "  const [sortField, sortDir] = STATE.filter.sort.split('-');",
        "  items.sort((a, b) => {",
        "    let va = a[sortField], vb = b[sortField];",
        "    if (typeof va === 'string') { va = va.toLowerCase(); vb = vb.toLowerCase(); }",
        "    if (va < vb) return sortDir === 'asc' ? -1 : 1;",
        "    if (va > vb) return sortDir === 'asc' ? 1 : -1;",
        "    return 0;",
        "  });",
        "",
        "  const tbody = document.getElementById('inventory-body');",
        f"  if (items.length === 0) {{",
        f"    tbody.innerHTML = '<tr><td colspan=\"{col_count}\" class=\"empty-state\">尚無資料</td></tr>';",
        "    return;",
        "  }",
        "  tbody.innerHTML = items.map(item => {",
    ]

    # Build per-column cell rendering in array form
    render_body_lines.append("    const cells = [")
    for col in schema:
        key = col["key"]
        col_type = col.get("type", "text")
        if col_type == "checkbox":
            render_body_lines.append(f"      `<td><input type=\"checkbox\" id=\"cb-${{item.id}}\" onclick=\"toggleComplete(${{item.id}})\" ${{item.completed ? 'checked' : ''}} /></td>`,")
        elif col_type == "text":
            render_body_lines.append(f"      `<td>${{item['{key}'] ?? ''}}</td>`,")
        elif col_type == "badge":
            render_body_lines.append(f"      `<td>${{_renderBadge(item['{key}'], item)}}</td>`,")
        elif col_type == "action":
            render_body_lines.append(f"      `<td><div class=\"action-btns\">${{_renderActions(item)}}</div></td>`,")
        elif col_type == "date":
            render_body_lines.append(f"      `<td>${{item['{key}'] ? new Date(item['{key}']).toLocaleDateString('zh-TW') : ''}}</td>`,")
        else:
            render_body_lines.append(f"      `<td>${{item['{key}'] ?? ''}}</td>`,")
    render_body_lines.append("    ].join('');")

    render_body_lines.extend([
        "    const rowClass = item.completed ? ' class=\"todo-completed\"' : '';",
        "    return `<tr${rowClass}>${cells}</tr>`;",
        "  }).join('');",
        "}",
    ])

    # Helper functions for badge and action types
    helpers = """
function _renderBadge(value, item) {
  if (value === undefined || value === null) return '';
  const v = String(value);
  if (v === '高') return '<span class="badge badge-danger">高</span>';
  if (v === '中') return '<span class="badge badge-info">中</span>';
  if (v === '低') return '<span class="badge badge-ok">低</span>';
  return '<span class="badge">' + v + '</span>';
}

function _renderActions(item) {
  const done = item.completed ? '✓' : '○';
  const doneClass = item.completed ? 'btn-done-active' : 'btn-done';
  return `<button class="${doneClass}" onclick="toggleComplete(${item.id})">${done}</button>`
    + `<button class="btn-edit" onclick="openEdit(${item.id})">編輯</button>`
    + `<button class="btn-delete" onclick="openDelete(${item.id})">刪除</button>`;
}

function toggleComplete(id) {
  const item = STATE.items.find(x => x.id === id);
  if (item) {
    item.completed = !item.completed;
    showToast(item.completed ? '已完成' : '已取消完成', item.completed ? 'success' : 'info');
    render();
  }
}
"""

    render_js = "function render() {\n" + "\n".join("  " + line for line in render_body_lines) + "\n}\n" + helpers

    return thead_html, render_js, col_count


def _build_form_from_schema(schema):
    """Generate form HTML from schema definition."""
    form_fields = []
    field_ids = []
    
    for col in schema:
        key = col["key"]
        label = col.get("label", key)
        col_type = col.get("type", "text")
        
        if col_type in ("action", "checkbox"):
            continue
        
        field_ids.append(key)
        
        if col_type == "badge":
            form_fields.append(
                f'      <div class="form-group">\n'
                f'        <label>{label}</label>\n'
                f'        <select id="field-{key}">\n'
                f'          <option value="">請選擇</option>\n'
                f'          <option value="高">高</option>\n'
                f'          <option value="中">中</option>\n'
                f'          <option value="低">低</option>\n'
                f'        </select>\n'
                f'      </div>'
            )
        elif col_type == "date":
            form_fields.append(
                f'      <div class="form-group">\n'
                f'        <label>{label}</label>\n'
                f'        <input type="date" id="field-{key}" />\n'
                f'      </div>'
            )
        else:
            form_fields.append(
                f'      <div class="form-group">\n'
                f'        <label>{label}</label>\n'
                f'        <input type="text" id="field-{key}" />\n'
                f'      </div>'
            )
    
    open_add = "function openAdd() {\\n  STATE.editTarget = null;\\n  document.getElementById('modal-title').textContent = '新增項目';\\n  document.getElementById('edit-id').value = '';\\n" + "\\n".join([f"  document.getElementById('field-{key}').value = '';" for key in field_ids]) + "\\n  document.getElementById('form-modal').style.display = 'flex';\\n}"
    
    open_edit = "function openEdit(id) {\\n  const item = STATE.items.find(x => x.id === id);\\n  STATE.editTarget = id;\\n  document.getElementById('modal-title').textContent = '編輯項目';\\n  document.getElementById('edit-id').value = id;\\n" + "\\n".join([f"  document.getElementById('field-{key}').value = item.{key} || '';" for key in field_ids]) + "\\n  document.getElementById('form-modal').style.display = 'flex';\\n}"
    
    submit_data = "\\n".join([f"    {key}: document.getElementById('field-{key}').value.trim()," for key in field_ids])
    
    form_html = (
        '<div id="form-modal" class="modal-overlay" style="display:none">\n'
        '  <div class="modal-panel">\n'
        '    <div class="modal-header">\n'
        '      <h3 id="modal-title">新增項目</h3>\n'
        '      <button class="modal-close" onclick="closeModal()">×</button>\n'
        '    </div>\n'
        '    <form id="inventory-form" class="modal-form">\n'
        '      <input type="hidden" id="edit-id" />\n'
        + '\n'.join(form_fields) + '\n'
        '      <div class="form-actions">\n'
        '        <button type="button" class="btn-cancel" onclick="closeModal()">取消</button>\n'
        '        <button type="submit" class="btn-primary">儲存</button>\n'
        '      </div>\n'
        '    </form>\n'
        '  </div>\n'
        '</div>'
    )
    
    return form_html, open_add, open_edit, submit_data


def compile_html(skills_used: List[str], intent_data: Dict) -> str:
    """將技能列表編譯成完整 HTML 頁面。"""

    theme = intent_data.get("theme", "glass")
    skill_blocks = load_skill_blocks(skills_used)

    html_parts = []
    css_parts = []
    js_parts = []

    for block in skill_blocks:
        html_parts.append(block["html"])
        css_parts.append(block["style"])
        if block.get("js"):
            js_parts.append(block["js"])

    # Build dynamic table from schema (default fallback if no schema)
    schema = intent_data.get("schema", [
        {"key": "name", "label": "名稱", "type": "text"},
        {"key": "category", "label": "分類", "type": "text"},
        {"key": "quantity", "label": "數量", "type": "text"},
        {"key": "status", "label": "狀態", "type": "badge"},
        {"key": "updatedAt", "label": "最後更新", "type": "date"},
        {"key": "actions", "label": "操作", "type": "action"},
    ])

    form_html, open_add_js, open_edit_js, submit_data_js = _build_form_from_schema(schema)

    dynamic_thead, dynamic_render, col_count = _build_dynamic_table_html(schema)

    # Build sort options dynamically from schema text/date columns
    sort_options = ""
    for col in schema:
        if col.get("type") in ("text", "date", "number"):
            sort_options += f'      <option value="{col["key"]}-asc">{col["label"]} ↑</option>\n'
            sort_options += f'      <option value="{col["key"]}-desc">{col["label"]} ↓</option>\n'

    # Toolbar — filter category only if a "category" column exists
    category_col_exists = any(c.get("key") == "category" for c in schema)
    filter_html = ""
    if category_col_exists:
        filter_html = '''    <select id="filter-category" onchange="handleFilter(this.value)">
      <option value="">全部分類</option>
      <option value="電子元件">電子元件</option>
      <option value="工具">工具</option>
      <option value="原料">原料</option>
      <option value="包裝">包裝</option>
    </select>
    '''
    else:
        filter_html = '    <select id="filter-category" style="display:none"></select>\n'

    toolbar_html = f'''<div class="toolbar">
  <div class="search-box">
    <input type="text" id="search-input" placeholder="搜尋..." oninput="handleSearch(this.value)" />
  </div>
  <div class="filter-group">
{filter_html}    <select id="sort-by" onchange="handleSort(this.value)">
{      sort_options}    </select>
  </div>
</div>'''

    # Modal
    modal_html = form_html

    confirm_html = '''<div id="confirm-overlay" class="modal-overlay" style="display:none">
  <div class="confirm-panel">
    <div class="confirm-icon">⚠️</div>
    <h3 id="confirm-title">確認刪除？</h3>
    <p id="confirm-message">此操作無法撤銷</p>
    <div class="confirm-actions">
      <button class="btn-cancel" onclick="closeConfirm()">取消</button>
      <button id="confirm-btn" class="btn-danger" onclick="doDelete()">刪除</button>
    </div>
  </div>
</div>'''

    toast_container = '<div id="toast-container"></div>'

    # Pre-compute for safe string insertion
    default_sort = (schema[0]["key"] if schema else "updatedAt") + "-desc"

    # Main app JS — use %s placeholder to avoid f-string brace-escaping issues with JS code
    app_js = '''<script>
const STATE = {
  items: @ITEMS@,
  nextId: 5,
  filter: { search: "", category: "", sort: "%%SORT%%" },
  deleteTarget: null,
  editTarget: null,
};

''' + dynamic_render + '''

{open_add_js}

{open_edit_js}

function closeModal() { document.getElementById('form-modal').style.display = 'none'; STATE.editTarget = null; }

function openDelete(id) {
  STATE.deleteTarget = id;
  const item = STATE.items.find(x => x.id === id);
  document.getElementById('confirm-title').textContent = '確認刪除「' + (item.title || item.name || '') + '」？';
  document.getElementById('confirm-overlay').style.display = 'flex';
}

function closeConfirm() { document.getElementById('confirm-overlay').style.display = 'none'; STATE.deleteTarget = null; }

function doDelete() {
  if (STATE.deleteTarget !== null) {
    STATE.items = STATE.items.filter(x => x.id !== STATE.deleteTarget);
    showToast('刪除成功', 'success');
    closeConfirm();
    render();
  }
}

document.getElementById('inventory-form').onsubmit = function(e) {
  e.preventDefault();
  const data = {
{submit_data}
  };
  if (STATE.editTarget !== null) {
    const idx = STATE.items.findIndex(x => x.id === STATE.editTarget);
    STATE.items[idx] = { ...STATE.items[idx], ...data };
    showToast('更新成功', 'success');
  } else {
    STATE.items.push({ id: STATE.nextId++, ...data });
    showToast('新增成功', 'success');
  }
  closeModal();
  render();
};

function handleSearch(v) { STATE.filter.search = v; render(); }
function handleFilter(v) { STATE.filter.category = v; render(); }
function handleSort(v) { STATE.filter.sort = v; render(); }

function showToast(message, type) {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = 'toast toast-' + type;
  toast.textContent = message;
  container.appendChild(toast);
  setTimeout(() => toast.remove(), 3000);
}

render();
</script>'''

    # 載入主題 CSS
    theme_css = load_theme_css(theme)
    base_css = "* { margin: 0; padding: 0; box-sizing: border-box; }\nbody { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }\n"
    # Generate items JS from parsed seed data
    items_list = intent_data.get('items', [])
    if items_list:
        items_js = ', '.join(
            '{id:' + str(i+1) + ', name:"' + item.get('name','') + '", category:"' +
            item.get('priority','') + '", quantity:0, minStock:0, note:"", updatedAt:"' +
            item.get('dueDate','') + '", completed:' + str(item.get('completed',False)).lower() + '}'
            for i, item in enumerate(items_list)
        )
        app_js = app_js.replace('@ITEMS@', items_js)
    else:
        app_js = app_js.replace('@ITEMS@', '{id:1,name:"範例項目",category:"一般",quantity:0,minStock:0,note:"",updatedAt:"2026-04-20"}')

    app_js = app_js.replace('%%SORT%%', default_sort)
    app_js = app_js.replace('{open_add_js}', open_add_js)
    app_js = app_js.replace('{open_edit_js}', open_edit_js)
    app_js = app_js.replace('{submit_data}', submit_data_js)
    page_css = """.todo-completed td { text-decoration: line-through; opacity: 0.6; }
.btn-done, .btn-done-active {
  display: inline-flex; align-items: center; justify-content: center;
  width: 32px; height: 32px; border-radius: 50%; border: 1px solid #d1d5db;
  background: #fff; color: #9ca3af; font-size: 16px; cursor: pointer; transition: all 0.15s;
}
.btn-done:hover { background: #f0fdf4; border-color: #86efac; color: #22c55e; }
.btn-done-active { background: #22c55e; border-color: #22c55e; color: #fff; }
.btn-done-active:hover { background: #16a34a; border-color: #16a34a; }
"""
    page = f'''<!DOCTYPE html>
<html lang="zh-TW">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{intent_data.get("name", "應用程式")}</title>
<style>
{page_css}
{base_css}
{merge_css(css_parts)}
{theme_css}
</style>
</head>
<body>

<header class="warehouse-header">
  <div class="header-left">
    <span class="header-title">📋 {intent_data.get("name", "應用程式")}</span>
  </div>
  <div class="header-actions">
    <button class="btn-primary" onclick="openAdd()">+ 新增</button>
  </div>
</header>

{toolbar_html}

<main style="padding: 24px;">
  <div class="inventory-table-wrapper">
    <table class="inventory-table">
{dynamic_thead}
      <tbody id="inventory-body"></tbody>
    </table>
  </div>
</main>

{modal_html}
{confirm_html}
{toast_container}
{app_js}
</body>
</html>'''

    return page


def load_skill_blocks(skill_names: List[str]) -> List[Dict]:
    import glob
    import os

    blocks = []
    for name in skill_names:
        # Try ui dir first
        matches = glob.glob(str(SKILLS_BASE / "ui" / f"{name}.skill"))
        if not matches:
            matches = glob.glob(str(SKILLS_BASE / "ui" / "**" / f"{name}.skill"), recursive=True)
        if matches:
            blocks.append(parse_skill_file(matches[0]))
    return blocks


def parse_skill_file(path: str) -> Dict:
    block = {"html": "", "style": "", "js": ""}
    current = None
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if stripped.startswith("[html]"):
                current = "html"
            elif stripped.startswith("[react]"):
                current = None  # skip react
            elif stripped.startswith("[style]"):
                current = "style"
            elif stripped.startswith("[js]"):
                current = "js"
            elif current == "html":
                block["html"] += line
            elif current == "style":
                block["style"] += line
            elif current == "js":
                block["js"] += line
    return block


def merge_css(css_parts: List[str]) -> str:
    """合併多個 CSS block，去除重複規則。"""
    lines = []
    seen = set()
    for part in css_parts:
        for line in part.split("\n"):
            stripped = line.strip()
            if not stripped or stripped.startswith("/*") or stripped.startswith("}"):
                if stripped not in seen:
                    seen.add(stripped)
                    lines.append(stripped)
            else:
                lines.append(line)
    return "\n".join(lines)


THEME_MAP = {
    "glass": "theme-glass",
    "modern": "theme-modern",
    "brutal": "theme-brutal",
    "soft": "theme-soft",
}


def load_theme_css(theme: str) -> str:
    """根據主題名載入對應 CSS。"""
    skill_name = THEME_MAP.get(theme, "theme-glass")
    import glob
    matches = glob.glob(str(SKILLS_BASE / "styles" / f"{skill_name}.skill"))
    if not matches:
        return ""
    block = {"html": "", "style": "", "js": ""}
    current = None
    with open(matches[0], "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if stripped.startswith("[html]"):
                current = None
            elif stripped.startswith("[react]"):
                current = None
            elif stripped.startswith("[style]"):
                current = "style"
            elif stripped.startswith("[js]"):
                current = None
            elif current == "style":
                block["style"] += line
    return block["style"]
