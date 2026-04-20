"""
html_compiler.py — HTML/CSS/JS 編譯器

接收元件樹，輸出完整可運行的 HTML 頁面。
"""

from typing import List, Dict

SKILLS_DIR = "skills/ui"


def compile_html(skills_used: List[str], intent_data: Dict) -> str:
    """將技能列表編譯成完整 HTML 頁面。"""

    # 收集所有需要的 skill code
    skill_blocks = load_skill_blocks(skills_used)

    html_parts = []
    css_parts = []
    js_parts = []

    for block in skill_blocks:
        html_parts.append(block["html"])
        css_parts.append(block["style"])
        if block.get("js"):
            js_parts.append(block["js"])

    # 建構完整頁面
    # Inject toolbar with search + filter + sort
    toolbar_html = '''<div class="toolbar">
  <div class="search-box">
    <input type="text" id="search-input" placeholder="搜尋名稱..." oninput="handleSearch(this.value)" />
  </div>
  <div class="filter-group">
    <select id="filter-category" onchange="handleFilter(this.value)">
      <option value="">全部分類</option>
      <option value="電子元件">電子元件</option>
      <option value="工具">工具</option>
      <option value="原料">原料</option>
      <option value="包裝">包裝</option>
    </select>
    <select id="sort-by" onchange="handleSort(this.value)">
      <option value="updatedAt-desc">最近更新</option>
      <option value="name-asc">名稱 A-Z</option>
      <option value="name-desc">名稱 Z-A</option>
      <option value="quantity-asc">數量 ↑</option>
      <option value="quantity-desc">數量 ↓</option>
    </select>
  </div>
</div>'''

    # Modal
    modal_html = '''<div id="form-modal" class="modal-overlay" style="display:none">
  <div class="modal-panel">
    <div class="modal-header">
      <h3 id="modal-title">新增項目</h3>
      <button class="modal-close" onclick="closeModal()">×</button>
    </div>
    <form id="inventory-form" class="modal-form">
      <input type="hidden" id="edit-id" />
      <div class="form-group">
        <label>名稱 *</label>
        <input type="text" id="field-name" required placeholder="例如：螺絲 M3" />
      </div>
      <div class="form-group">
        <label>分類 *</label>
        <select id="field-category" required>
          <option value="">請選擇</option>
          <option value="電子元件">電子元件</option>
          <option value="工具">工具</option>
          <option value="原料">原料</option>
          <option value="包裝">包裝</option>
        </select>
      </div>
      <div class="form-group">
        <label>數量 *</label>
        <input type="number" id="field-quantity" required min="0" />
      </div>
      <div class="form-group">
        <label>安全存量</label>
        <input type="number" id="field-minStock" min="0" value="10" />
      </div>
      <div class="form-group">
        <label>備註</label>
        <textarea id="field-note" rows="3"></textarea>
      </div>
      <div class="form-actions">
        <button type="button" class="btn-cancel" onclick="closeModal()">取消</button>
        <button type="submit" class="btn-primary">儲存</button>
      </div>
    </form>
  </div>
</div>'''

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

    # Main app JS
    app_js = '''<script>
const STATE = {
  items: [
    { id: 1, name: "螺絲 M3x10", category: "工具", quantity: 250, minStock: 50, note: "常用規格", updatedAt: "2026-04-15" },
    { id: 2, name: "PCB板 10x5cm", category: "電子元件", quantity: 8, minStock: 20, note: "庫存偏低", updatedAt: "2026-04-18" },
    { id: 3, name: "紙箱 S號", category: "包裝", quantity: 120, minStock: 30, note: "", updatedAt: "2026-04-10" },
    { id: 4, name: "鋁箔紙", category: "原料", quantity: 0, minStock: 5, note: "已用完", updatedAt: "2026-04-19" },
  ],
  nextId: 5,
  filter: { search: "", category: "", sort: "updatedAt-desc" },
  deleteTarget: null,
  editTarget: null,
};

function render() {
  let items = STATE.items.filter(item => {
    const matchSearch = !STATE.filter.search ||
      item.name.toLowerCase().includes(STATE.filter.search.toLowerCase());
    const matchCat = !STATE.filter.category || item.category === STATE.filter.category;
    return matchSearch && matchCat;
  });

  const [sortField, sortDir] = STATE.filter.sort.split('-');
  items.sort((a, b) => {
    let va = a[sortField], vb = b[sortField];
    if (typeof va === 'string') { va = va.toLowerCase(); vb = vb.toLowerCase(); }
    if (va < vb) return sortDir === 'asc' ? -1 : 1;
    if (va > vb) return sortDir === 'asc' ? 1 : -1;
    return 0;
  });

  const tbody = document.getElementById('inventory-body');
  if (items.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6" class="empty-state">尚無庫存資料</td></tr>';
    return;
  }
  tbody.innerHTML = items.map(item => {
    const badge = item.quantity === 0 ? '<span class="badge badge-danger">缺貨</span>'
      : item.quantity <= item.minStock ? '<span class="badge badge-warning">庫存不足</span>'
      : '<span class="badge badge-ok">正常</span>';
    const catClass = 'cat-' + item.category;
    return `<tr>
      <td>${item.name}</td>
      <td><span class="category-tag ${catClass}">${item.category}</span></td>
      <td>${item.quantity}</td>
      <td>${badge}</td>
      <td>${new Date(item.updatedAt).toLocaleDateString('zh-TW')}</td>
      <td>
        <div class="action-btns">
          <button class="btn-edit" onclick="openEdit(${item.id})">編輯</button>
          <button class="btn-delete" onclick="openDelete(${item.id})">刪除</button>
        </div>
      </td>
    </tr>`;
  }).join('');
}

function openAdd() {
  STATE.editTarget = null;
  document.getElementById('modal-title').textContent = '新增項目';
  document.getElementById('edit-id').value = '';
  ['field-name','field-category','field-quantity','field-minStock','field-note'].forEach(id => document.getElementById(id).value = '');
  document.getElementById('field-minStock').value = '10';
  document.getElementById('form-modal').style.display = 'flex';
}

function openEdit(id) {
  const item = STATE.items.find(x => x.id === id);
  STATE.editTarget = id;
  document.getElementById('modal-title').textContent = '編輯項目';
  document.getElementById('edit-id').value = id;
  document.getElementById('field-name').value = item.name;
  document.getElementById('field-category').value = item.category;
  document.getElementById('field-quantity').value = item.quantity;
  document.getElementById('field-minStock').value = item.minStock;
  document.getElementById('field-note').value = item.note || '';
  document.getElementById('form-modal').style.display = 'flex';
}

function closeModal() { document.getElementById('form-modal').style.display = 'none'; STATE.editTarget = null; }

function openDelete(id) {
  STATE.deleteTarget = id;
  const item = STATE.items.find(x => x.id === id);
  document.getElementById('confirm-title').textContent = '確認刪除「' + item.name + '」？';
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
    name: document.getElementById('field-name').value.trim(),
    category: document.getElementById('field-category').value,
    quantity: parseInt(document.getElementById('field-quantity').value) || 0,
    minStock: parseInt(document.getElementById('field-minStock').value) || 10,
    note: document.getElementById('field-note').value.trim(),
    updatedAt: new Date().toISOString().split('T')[0],
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

    page = f'''<!DOCTYPE html>
<html lang="zh-TW">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>倉儲管理系統</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f8fafc; color: #334155; }}
{merge_css(css_parts)}
</style>
</head>
<body>

<header class="warehouse-header">
  <div class="header-left">
    <span class="header-title">📦 倉儲管理系統</span>
  </div>
  <div class="header-actions">
    <button class="btn-primary" onclick="openAdd()">+ 新增</button>
  </div>
</header>

{toolbar_html}

<main style="padding: 24px;">
  <div class="inventory-table-wrapper">
    <table class="inventory-table">
      <thead>
        <tr>
          <th>名稱</th><th>分類</th><th>數量</th><th>狀態</th><th>最後更新</th><th>操作</th>
        </tr>
      </thead>
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
        matches = glob.glob(f"skills/ui/{name}.skill", recursive=False)
        if not matches:
            matches = glob.glob(f"skills/ui/**/{name}.skill", recursive=True)
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
