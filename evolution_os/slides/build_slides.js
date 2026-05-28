const pptxgen = require('pptxgenjs');
const pptx = new pptxgen();

// 配色
const C = {
  dark: '1A1A2E', blue: '16213E', teal: '0F3460',
  accent: '00ADB5', white: 'EEEEEE', gray: 'AAAAAA',
  light: 'F5F5F5', red: 'E94560'
};

// 10 slides大綱
const slides_data = [
  { title: 'Evolution OS', sub: '意圖驅動的多節點 AI 軟體建構系統', bg: C.dark },
  { title: '研究動機', sub: '現有 AI 工具的四大瓶頸', bg: C.blue },
  { title: '問題背景', sub: '單一 Prompt 無法處理多領域複雜任務', bg: C.blue },
  { title: '解決方向', sub: 'Intent-Driven + Self-Optimization', bg: C.blue },
  { title: '系統架構', sub: 'Planner · Executor · Memory Graph', bg: C.dark },
  { title: '三階段流程', sub: 'S1 確認需求 → S2 分析問題 → S3 規劃派工', bg: C.dark },
  { title: '分工決策邏輯', sub: '根據複雜度自動選擇 Solo 或 Fork 模式', bg: C.dark },
  { title: 'PlannerManifest', sub: '結構化輸出，為下游執行系統提供藍圖', bg: C.dark },
  { title: '系統展示', sub: 'Demo: 簡單任務 vs 複雜任務', bg: C.teal },
  { title: '結論與未來', sub: '已達成目標 · v0.3+ 發展方向', bg: C.dark }
];

slides_data.forEach((s, i) => {
  const sl = pptx.addSlide();
  sl.background = { color: s.bg };

  // 左側裝飾線
  sl.addShape('rect', { x: 0, y: 0, w: 0.08, h: 5.625, fill: { color: C.accent } });

  // 頁碼
  sl.addText(`${i + 1} / ${slides_data.length}`, {
    x: 0.3, y: 5.1, w: 1, h: 0.4,
    color: C.gray, fontSize: 10, fontFace: 'Arial'
  });

  // 主標題
  sl.addText(s.title, {
    x: 0.5, y: 1.8, w: 9, h: 1.2,
    color: C.white, fontSize: 44, bold: true, fontFace: 'Arial',
    align: 'left', valign: 'middle'
  });

  // 副標題
  sl.addText(s.sub, {
    x: 0.5, y: 3.1, w: 9, h: 0.8,
    color: C.accent, fontSize: 20, fontFace: 'Arial',
    align: 'left', valign: 'top'
  });

  // 右下裝飾點
  sl.addShape('ellipse', { x: 9.2, y: 5.0, w: 0.15, h: 0.15, fill: { color: C.accent } });
  sl.addShape('ellipse', { x: 9.0, y: 5.0, w: 0.15, h: 0.15, fill: { color: C.gray } });
  sl.addShape('ellipse', { x: 8.8, y: 5.0, w: 0.15, h: 0.15, fill: { color: C.gray } });
});

pptx.writeFile({ fileName: '/Users/oren/Desktop/funnytest/evolution_os/Evolution_OS_期末報告.pptx' })
  .then(() => console.log('OK'))
  .catch(e => console.error(e));
