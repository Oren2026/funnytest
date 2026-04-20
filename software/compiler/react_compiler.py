"""
react_compiler.py — React 元件編譯器

輸出完整 React 專案結構（可粘貼到 Vite / Next.js 使用）。
"""

from typing import Dict
import glob
from pathlib import Path
SKILLS_BASE = Path(__file__).parent.parent / "skills"


def compile_react(skills_used: list, intent_data: Dict) -> str:
    """編譯成完整 React 元件。"""

    skill_blocks = load_skill_blocks(skills_used)

    # 合併所有 style
    all_css = "\n".join(b["style"] for b in skill_blocks if b.get("style"))

    # 收集 React 元件
    react_components = []
    for b in skill_blocks:
        react_components.append(b["react"])

    page = f'''import React, {{ useState, useMemo }} from 'react';

{chr(10).join(react_components)}

const initialItems = [
  {{ id: 1, name: "螺絲 M3x10", category: "工具", quantity: 250, minStock: 50, note: "常用規格", updatedAt: "2026-04-15" }},
  {{ id: 2, name: "PCB板 10x5cm", category: "電子元件", quantity: 8, minStock: 20, note: "庫存偏低", updatedAt: "2026-04-18" }},
  {{ id: 3, name: "紙箱 S號", category: "包裝", quantity: 120, minStock: 30, note: "", updatedAt: "2026-04-10" }},
  {{ id: 4, name: "鋁箔紙", category: "原料", quantity: 0, minStock: 5, note: "已用完", updatedAt: "2026-04-19" }},
];

export default function WarehouseApp() {{
  const [items, setItems] = useState(initialItems);
  const [nextId, setNextId] = useState(5);
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('');
  const [sort, setSort] = useState('updatedAt-desc');
  const [modal, setModal] = useState({{ open: false, mode: 'add', data: null }});
  const [confirm, setConfirm] = useState({{ open: false, target: null }});
  const [toasts, setToasts] = useState([]);

  const showToast = (msg, type) => {{
    const id = Date.now();
    setToasts(t => [...t, {{ id, msg, type }}]);
    setTimeout(() => setToasts(t => t.filter(x => x.id !== id)), 3000);
  }};

  const filtered = useMemo(() => {{
    let result = items.filter(item => {{
      const matchSearch = !search || item.name.toLowerCase().includes(search.toLowerCase());
      const matchCat = !category || item.category === category;
      return matchSearch && matchCat;
    }});
    const [field, dir] = sort.split('-');
    result.sort((a, b) => {{
      let va = a[field], vb = b[field];
      if (typeof va === 'string') {{ va = va.toLowerCase(); vb = vb.toLowerCase(); }}
      if (va < vb) return dir === 'asc' ? -1 : 1;
      if (va > vb) return dir === 'asc' ? 1 : -1;
      return 0;
    }});
    return result;
  }}, [items, search, category, sort]);

  const getBadge = (qty, min) => {{
    if (qty === 0) return <span className="badge badge-danger">缺貨</span>;
    if (qty <= min) return <span className="badge badge-warning">庫存不足</span>;
    return <span className="badge badge-ok">正常</span>;
  }};

  const catClass = (cat) => 'category-tag cat-' + cat;

  const handleSave = (data) => {{
    if (modal.mode === 'edit') {{
      setItems(items.map(x => x.id === modal.data.id ? {{ ...x, ...data, updatedAt: new Date().toISOString().split('T')[0] }} : x));
      showToast('更新成功', 'success');
    }} else {{
      setItems([...items, {{ id: nextId, ...data, updatedAt: new Date().toISOString().split('T')[0] }}]);
      setNextId(n => n + 1);
      showToast('新增成功', 'success');
    }}
    setModal({{ open: false, mode: 'add', data: null }});
  }};

  const handleDelete = () => {{
    if (!confirm.target) return;
    setItems(items.filter(x => x.id !== confirm.target));
    showToast('刪除成功', 'success');
    setConfirm({{ open: false, target: null }});
  }};

  return (
    <>
      <Header>
        <button className="btn-primary" onClick={{() => setModal({{ open: true, mode: 'add', data: null }})}}>
          + 新增
        </button>
      </Header>

      <Toolbar
        onSearch={{setSearch}}
        onFilter={{setCategory}}
        onSort={{(f, d) => setSort(f + '-' + d)}}
      />

      <main style={{{{ padding: '24px' }}}}>
        <div className="inventory-table-wrapper">
          <table className="inventory-table">
            <thead>
              <tr>
                <th>名稱</th><th>分類</th><th>數量</th><th>狀態</th><th>最後更新</th><th>操作</th>
              </tr>
            </thead>
            <tbody>
              {{filtered.length === 0 ? (
                <tr><td colSpan="6" className="empty-state">尚無庫存資料</td></tr>
              ) : filtered.map(item => (
                <tr key={{item.id}}>
                  <td>{{item.name}}</td>
                  <td><span className={{catClass(item.category)}}>{{item.category}}</span></td>
                  <td>{{item.quantity}}</td>
                  <td>{{getBadge(item.quantity, item.minStock)}}</td>
                  <td>{{new Date(item.updatedAt).toLocaleDateString('zh-TW')}}</td>
                  <td>
                    <div className="action-btns">
                      <button className="btn-edit" onClick={{() => setModal({{ open: true, mode: 'edit', data: item }})}}>編輯</button>
                      <button className="btn-delete" onClick={{() => setConfirm({{ open: true, target: item.id }})}}>刪除</button>
                    </div>
                  </td>
                </tr>
              ))}}
            </tbody>
          </table>
        </div>
      </main>

      <ModalForm
        isOpen={{modal.open}}
        mode={{modal.mode}}
        initial={{modal.data}}
        onSave={{handleSave}}
        onClose={{() => setModal({{ open: false, mode: 'add', data: null }})}}
      />

      <ConfirmDialog
        isOpen={{confirm.open}}
        title={{confirm.target ? `確認刪除「${{items.find(x => x.id === confirm.target)?.name}}」？` : ''}}
        message="此操作無法撤銷"
        confirmLabel="刪除"
        onConfirm={{handleDelete}}
        onCancel={{() => setConfirm({{ open: false, target: null }})}}
      />

      <div className="toast-container">
        {{toasts.map(t => (
          <div key={{t.id}} className={{`toast toast-${{t.type}}`}}>{{t.msg}}</div>
        ))}}
      </div>
    </>
  );
}}
'''
    return page


def load_skill_blocks(skill_names: list) -> list:
    blocks = []
    for name in skill_names:
        matches = glob.glob(str(SKILLS_BASE / "ui" / f"{name}.skill"))
        if not matches:
            matches = glob.glob(str(SKILLS_BASE / "ui" / "**" / f"{name}.skill"), recursive=True)
        if matches:
            blocks.append(parse_skill_file(matches[0]))
    return blocks


def parse_skill_file(path: str) -> Dict:
    block = {"html": "", "style": "", "react": ""}
    current = None
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if stripped.startswith("[html]"):
                current = None
            elif stripped.startswith("[react]"):
                current = "react"
            elif stripped.startswith("[style]"):
                current = "style"
            elif stripped.startswith("[js]"):
                current = None
            elif current == "react":
                block["react"] += line
            elif current == "style":
                block["style"] += line
    return block
