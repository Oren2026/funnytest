import React, { useState, useMemo } from 'react';

const InventoryTable = ({ items, onEdit, onDelete }) => (
  <div className="inventory-table-wrapper">
    <table className="inventory-table">
      <thead>
        <tr>
          <th>名稱</th>
          <th>分類</th>
          <th>數量</th>
          <th>狀態</th>
          <th>最後更新</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {items.length === 0 ? (
          <tr><td colSpan="6" className="empty-state">尚無庫存資料</td></tr>
        ) : (
          items.map(item => (
            <tr key={item.id}>
              <td>{item.name}</td>
              <td><span className={`category-tag cat-${item.category}`}>{item.category}</span></td>
              <td>{item.quantity}</td>
              <td>{getStockBadge(item.quantity, item.minStock)}</td>
              <td>{new Date(item.updatedAt).toLocaleDateString('zh-TW')}</td>
              <td>
                <div className="action-btns">
                  <button className="btn-edit" onClick={() => onEdit(item)}>編輯</button>
                  <button className="btn-delete" onClick={() => onDelete(item)}>刪除</button>
                </div>
              </td>
            </tr>
          ))
        )}
      </tbody>
    </table>
  </div>
);

const getStockBadge = (qty, min) => {
  if (qty === 0) return <span className="badge badge-danger">缺貨</span>;
  if (qty <= min) return <span className="badge badge-warning">庫存不足</span>;
  return <span className="badge badge-ok">正常</span>;
};


const ButtonDanger = ({ label, onClick }) => (
  <button className="btn-danger" onClick={onClick}>
    {label}
  </button>
);


const ModalForm = ({ isOpen, mode, initial, onSave, onClose }) => {
  const [form, setForm] = React.useState(initial || { name: '', category: '', quantity: 0, minStock: 10, note: '' });
  if (!isOpen) return null;
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-panel" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{mode === 'add' ? '新增項目' : '編輯項目'}</h3>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>
        <form className="modal-form" onSubmit={e => { e.preventDefault(); onSave(form); }}>
          <div className="form-group">
            <label>名稱 *</label>
            <input type="text" required value={form.name} onChange={e => setForm({...form, name: e.target.value})} placeholder="例如：螺絲 M3" />
          </div>
          <div className="form-group">
            <label>分類 *</label>
            <select required value={form.category} onChange={e => setForm({...form, category: e.target.value})}>
              <option value="">請選擇</option>
              <option value="電子元件">電子元件</option>
              <option value="工具">工具</option>
              <option value="原料">原料</option>
              <option value="包裝">包裝</option>
            </select>
          </div>
          <div className="form-group">
            <label>數量 *</label>
            <input type="number" required min="0" value={form.quantity} onChange={e => setForm({...form, quantity: +e.target.value})} />
          </div>
          <div className="form-group">
            <label>安全存量</label>
            <input type="number" min="0" value={form.minStock} onChange={e => setForm({...form, minStock: +e.target.value})} />
          </div>
          <div className="form-actions">
            <button type="button" className="btn-cancel" onClick={onClose}>取消</button>
            <button type="submit" className="btn-primary">儲存</button>
          </div>
        </form>
      </div>
    </div>
  );
};


const Header = ({ children }) => (
  <header className="warehouse-header">
    <div className="header-left">
      <span className="header-title">📦 倉儲管理系統</span>
    </div>
    <div className="header-actions">
      {children}
    </div>
  </header>
);


const ButtonPrimary = ({ label, onClick }) => (
  <button className="btn-primary" onClick={onClick}>
    {label}
  </button>
);


const Toolbar = ({ onSearch, onFilter, onSort }) => (
  <div className="toolbar">
    <div className="search-box">
      <input type="text" placeholder="搜尋名稱..." onChange={e => onSearch(e.target.value)} />
    </div>
    <div className="filter-group">
      <select onChange={e => onFilter(e.target.value)}>
        <option value="">全部分類</option>
        <option value="電子元件">電子元件</option>
        <option value="工具">工具</option>
        <option value="原料">原料</option>
        <option value="包裝">包裝</option>
      </select>
      <select onChange={e => {
        const [field, dir] = e.target.value.split('-');
        onSort(field, dir);
      }}>
        <option value="updatedAt-desc">最近更新</option>
        <option value="name-asc">名稱 A-Z</option>
        <option value="quantity-asc">數量 ↑</option>
        <option value="quantity-desc">數量 ↓</option>
      </select>
    </div>
  </div>
);


const ConfirmDialog = ({ isOpen, title, message, confirmLabel, onConfirm, onCancel }) => {
  if (!isOpen) return null;
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="confirm-panel" onClick={e => e.stopPropagation()}>
        <div className="confirm-icon">⚠️</div>
        <h3>{title}</h3>
        <p>{message}</p>
        <div className="confirm-actions">
          <button className="btn-cancel" onClick={onCancel}>取消</button>
          <button className="btn-danger" onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  );
};


const Badge = ({ type, children }) => (
  <span className={`badge badge-${type}`}>{children}</span>
);
// type: ok | warning | danger


const ToastContext = React.createContext();
const ToastProvider = ({ children }) => {
  const [toasts, setToasts] = React.useState([]);
  const addToast = (message, type) => {
    const id = Date.now();
    setToasts(t => [...t, { id, message, type }]);
    setTimeout(() => setToasts(t => t.filter(x => x.id !== id)), 3000);
  };
  return (
    <ToastContext.Provider value={addToast}>
      {children}
      <div className="toast-container">
        {toasts.map(t => (
          <div key={t.id} className={`toast toast-${t.type}`}>{t.message}</div>
        ))}
      </div>
    </ToastContext.Provider>
  );
};
const useToast = () => React.useContext(ToastContext);
// Usage: const toast = useToast(); toast('操作成功', 'success');



const initialItems = [
  { id: 1, name: "螺絲 M3x10", category: "工具", quantity: 250, minStock: 50, note: "常用規格", updatedAt: "2026-04-15" },
  { id: 2, name: "PCB板 10x5cm", category: "電子元件", quantity: 8, minStock: 20, note: "庫存偏低", updatedAt: "2026-04-18" },
  { id: 3, name: "紙箱 S號", category: "包裝", quantity: 120, minStock: 30, note: "", updatedAt: "2026-04-10" },
  { id: 4, name: "鋁箔紙", category: "原料", quantity: 0, minStock: 5, note: "已用完", updatedAt: "2026-04-19" },
];

export default function WarehouseApp() {
  const [items, setItems] = useState(initialItems);
  const [nextId, setNextId] = useState(5);
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('');
  const [sort, setSort] = useState('updatedAt-desc');
  const [modal, setModal] = useState({ open: false, mode: 'add', data: null });
  const [confirm, setConfirm] = useState({ open: false, target: null });
  const [toasts, setToasts] = useState([]);

  const showToast = (msg, type) => {
    const id = Date.now();
    setToasts(t => [...t, { id, msg, type }]);
    setTimeout(() => setToasts(t => t.filter(x => x.id !== id)), 3000);
  };

  const filtered = useMemo(() => {
    let result = items.filter(item => {
      const matchSearch = !search || item.name.toLowerCase().includes(search.toLowerCase());
      const matchCat = !category || item.category === category;
      return matchSearch && matchCat;
    });
    const [field, dir] = sort.split('-');
    result.sort((a, b) => {
      let va = a[field], vb = b[field];
      if (typeof va === 'string') { va = va.toLowerCase(); vb = vb.toLowerCase(); }
      if (va < vb) return dir === 'asc' ? -1 : 1;
      if (va > vb) return dir === 'asc' ? 1 : -1;
      return 0;
    });
    return result;
  }, [items, search, category, sort]);

  const getBadge = (qty, min) => {
    if (qty === 0) return <span className="badge badge-danger">缺貨</span>;
    if (qty <= min) return <span className="badge badge-warning">庫存不足</span>;
    return <span className="badge badge-ok">正常</span>;
  };

  const catClass = (cat) => 'category-tag cat-' + cat;

  const handleSave = (data) => {
    if (modal.mode === 'edit') {
      setItems(items.map(x => x.id === modal.data.id ? { ...x, ...data, updatedAt: new Date().toISOString().split('T')[0] } : x));
      showToast('更新成功', 'success');
    } else {
      setItems([...items, { id: nextId, ...data, updatedAt: new Date().toISOString().split('T')[0] }]);
      setNextId(n => n + 1);
      showToast('新增成功', 'success');
    }
    setModal({ open: false, mode: 'add', data: null });
  };

  const handleDelete = () => {
    if (!confirm.target) return;
    setItems(items.filter(x => x.id !== confirm.target));
    showToast('刪除成功', 'success');
    setConfirm({ open: false, target: null });
  };

  return (
    <>
      <Header>
        <button className="btn-primary" onClick={() => setModal({ open: true, mode: 'add', data: null })}>
          + 新增
        </button>
      </Header>

      <Toolbar
        onSearch={setSearch}
        onFilter={setCategory}
        onSort={(f, d) => setSort(f + '-' + d)}
      />

      <main style={{ padding: '24px' }}>
        <div className="inventory-table-wrapper">
          <table className="inventory-table">
            <thead>
              <tr>
                <th>名稱</th><th>分類</th><th>數量</th><th>狀態</th><th>最後更新</th><th>操作</th>
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr><td colSpan="6" className="empty-state">尚無庫存資料</td></tr>
              ) : filtered.map(item => (
                <tr key={item.id}>
                  <td>{item.name}</td>
                  <td><span className={catClass(item.category)}>{item.category}</span></td>
                  <td>{item.quantity}</td>
                  <td>{getBadge(item.quantity, item.minStock)}</td>
                  <td>{new Date(item.updatedAt).toLocaleDateString('zh-TW')}</td>
                  <td>
                    <div className="action-btns">
                      <button className="btn-edit" onClick={() => setModal({ open: true, mode: 'edit', data: item })}>編輯</button>
                      <button className="btn-delete" onClick={() => setConfirm({ open: true, target: item.id })}>刪除</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </main>

      <ModalForm
        isOpen={modal.open}
        mode={modal.mode}
        initial={modal.data}
        onSave={handleSave}
        onClose={() => setModal({ open: false, mode: 'add', data: null })}
      />

      <ConfirmDialog
        isOpen={confirm.open}
        title={confirm.target ? `確認刪除「${items.find(x => x.id === confirm.target)?.name}」？` : ''}
        message="此操作無法撤銷"
        confirmLabel="刪除"
        onConfirm={handleDelete}
        onCancel={() => setConfirm({ open: false, target: null })}
      />

      <div className="toast-container">
        {toasts.map(t => (
          <div key={t.id} className={`toast toast-${t.type}`}>{t.msg}</div>
        ))}
      </div>
    </>
  );
}
