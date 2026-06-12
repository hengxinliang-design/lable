/// Self-contained HTML playground page served at `GET /`.
/// All HTML/CSS/JS is inlined — no external dependencies.
/// Dimensions follow Labelary convention (inches); converted to mm before
/// calling POST /convert.  Render always produces PNG; PDF is downloaded lazily.
pub const PLAYGROUND_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Label Platform Workbench</title>
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

  :root {
    --deep-teal: #01666A;
    --lagoon: #159CBA;
    --shell: #FAF5F2;
    --coral-light: #FEC7C2;
    --coral: #FF714F;
    --bg: var(--shell);
    --surface: #ffffff;
    --surface2: #fff8f6;
    --border: #dfd6d2;
    --accent: var(--lagoon);
    --accent-hover: #087f99;
    --text: #11181f;
    --text-dim: #5f6d72;
    --error: #b9402d;
    --success: var(--deep-teal);
    --radius: 8px;
    --font-mono: "JetBrains Mono","Fira Code","Cascadia Code",Consolas,monospace;
    --font-ui: system-ui,-apple-system,"Segoe UI",sans-serif;
  }

  html, body { height: 100%; }

  body {
    background: var(--bg); color: var(--text);
    font-family: var(--font-ui); font-size: 14px;
    display: flex; flex-direction: column; overflow: hidden;
  }

  header {
    flex-shrink: 0;
    background: linear-gradient(135deg, var(--deep-teal), var(--lagoon));
    border-bottom: 1px solid rgba(1,102,106,0.18);
    padding: 13px 22px; display: flex; align-items: center; gap: 14px;
    color: #fff;
  }

  .logo { display: flex; align-items: center; gap: 8px; }

  .logo-icon {
    width: 30px; height: 30px; background: var(--coral);
    border-radius: 6px; display: flex; align-items: center;
    justify-content: center; font-size: 14px; font-weight: 800; color: #fff;
  }

  .logo-text { font-size: 16px; font-weight: 700; color: #fff; }

  .tagline {
    color: rgba(255,255,255,0.82); font-size: 13px;
    border-left: 1px solid rgba(255,255,255,0.32); padding-left: 14px;
  }

  .header-spacer { flex: 1; }

  .badge {
    font-size: 11px; color: #fff; background: rgba(255,255,255,0.14);
    border: 1px solid rgba(255,255,255,0.28); border-radius: 999px; padding: 3px 9px;
  }

  .module-nav {
    flex-shrink: 0;
    display: flex; align-items: center; gap: 8px;
    padding: 10px 14px; background: var(--surface);
    border-bottom: 1px solid var(--border); overflow-x: auto;
  }

  .module-tab {
    border: 1px solid transparent; background: transparent; color: var(--text-dim);
    border-radius: 7px; padding: 8px 12px; font-size: 13px; font-weight: 700;
    font-family: var(--font-ui); cursor: pointer; white-space: nowrap;
  }

  .module-tab:hover { color: var(--deep-teal); background: var(--surface2); }

  .module-tab.active {
    color: #fff; background: var(--deep-teal);
    border-color: rgba(1,102,106,0.22);
  }

  .workspace { flex: 1; overflow: hidden; background: var(--bg); }
  .module-panel { display: none; height: 100%; overflow: auto; }
  .module-panel.active { display: block; }

  .designer-grid {
    flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;
    height: 100%;
  }

  .module-dashboard { padding: 18px; }
  .module-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 14px; }
  .module-title h2 { font-size: 19px; line-height: 1.2; margin: 0 0 5px; }
  .module-title p { color: var(--text-dim); font-size: 13px; line-height: 1.5; max-width: 760px; }
  .status-pill { border-radius: 999px; padding: 5px 10px; background: var(--coral-light); color: #5b241b; font-size: 12px; font-weight: 800; white-space: nowrap; }
  .dashboard-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
  .module-card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 14px; min-height: 122px; }
  .module-card h3 { font-size: 13px; margin-bottom: 8px; color: var(--deep-teal); }
  .module-card p, .module-card li { color: var(--text-dim); font-size: 12px; line-height: 1.45; }
  .module-card ul { margin-left: 16px; }
  .module-card.compact { min-height: 0; }
  .metric-row { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-bottom: 12px; }
  .metric { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 12px; }
  .metric strong { display: block; font-size: 24px; color: var(--deep-teal); line-height: 1.1; }
  .metric span { color: var(--text-dim); font-size: 12px; }
  .context-bar { display: flex; align-items: center; justify-content: space-between; gap: 12px; background: #e4f4f6; border: 1px solid rgba(1,102,106,0.18); border-radius: var(--radius); padding: 10px 12px; margin-bottom: 12px; }
  .context-bar strong { color: var(--deep-teal); }
  .context-bar span { color: var(--text-dim); font-size: 12px; }
  .alert-strip { border: 2px solid var(--coral); background: #fff1ee; border-radius: var(--radius); padding: 13px 14px; margin-bottom: 12px; }
  .alert-strip strong { display: block; color: var(--error); margin-bottom: 4px; }
  .module-table { width: 100%; border-collapse: collapse; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); overflow: hidden; table-layout: auto; }
  .module-table th, .module-table td { border-bottom: 1px solid var(--border); padding: 9px 10px; text-align: left; font-size: 12px; }
  .module-table th { background: var(--surface2); color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.4px; }
  .module-table tr:last-child td { border-bottom: none; }
  .field-map-table { min-width: 1040px; table-layout: fixed; }
  .field-map-table th:nth-child(1), .field-map-table td:nth-child(1) { width: 82px; }
  .field-map-table th:nth-child(2), .field-map-table td:nth-child(2) { width: 40%; }
  .field-map-table th:nth-child(3), .field-map-table td:nth-child(3) { width: 140px; }
  .field-map-table th:nth-child(4), .field-map-table td:nth-child(4) { width: 280px; }
  .field-map-table th:nth-child(5), .field-map-table td:nth-child(5) { width: 96px; }
  .field-source, .field-api-name, .field-value-input {
    width: 100%; min-width: 0; border: 1px solid var(--border); border-radius: 5px;
    background: #fff; color: var(--text); font: 12px/1.35 var(--font-ui);
    padding: 7px 8px; outline: none;
  }
  .field-api-name { font-family: var(--font-mono); }
  .field-value-input { min-height: 34px; resize: vertical; overflow: hidden; }
  .field-source:focus, .field-api-name:focus, .field-value-input:focus { border-color: var(--lagoon); box-shadow: 0 0 0 2px rgba(21,156,186,0.12); }
  .row-action {
    width: 100%; border: 1px solid var(--border); border-radius: 5px; background: #fff;
    color: var(--error); font: 12px/1.35 var(--font-ui); padding: 7px 8px; cursor: pointer;
  }
  .chip { display: inline-flex; align-items: center; border-radius: 999px; padding: 2px 8px; font-size: 11px; font-weight: 700; background: var(--surface2); color: var(--text-dim); }
  .chip.ok { background: #e4f4f6; color: var(--deep-teal); }
  .chip.warn { background: var(--coral-light); color: #5b241b; }
  .chip.bad { background: #fff1ee; color: var(--error); }
  .form-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-bottom: 12px; }
  .form-field { display: flex; flex-direction: column; gap: 5px; }
  .form-field label { font-size: 11px; font-weight: 700; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.4px; }
  .form-field input, .form-field select, .form-field textarea {
    width: 100%; border: 1px solid var(--border); border-radius: 6px; background: #fff;
    color: var(--text); font: 12px/1.4 var(--font-ui); padding: 8px; outline: none;
  }
  .form-field textarea { min-height: 190px; font-family: var(--font-mono); resize: vertical; }
  .form-field input:focus, .form-field select:focus, .form-field textarea:focus { border-color: var(--lagoon); }
  .action-row { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; margin: 10px 0 12px; }
  .action-btn {
    border: 1px solid var(--border); background: #fff; color: var(--deep-teal);
    border-radius: 6px; padding: 8px 12px; font-size: 12px; font-weight: 800;
    cursor: pointer; font-family: var(--font-ui);
  }
  .action-btn.primary { background: var(--coral); color: #11181f; border-color: var(--coral); }
  .action-btn.dark { background: var(--deep-teal); color: #fff; border-color: var(--deep-teal); }
  .action-btn:disabled { opacity: 0.55; cursor: not-allowed; }
  .response-box {
    background: #014e52; color: #f8fbfb; border-radius: var(--radius); padding: 12px;
    min-height: 96px; white-space: pre-wrap; font: 12px/1.5 var(--font-mono); overflow: auto;
  }
  .two-col { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 12px; }
  .two-col.wide-left { grid-template-columns: minmax(680px, 1.35fr) minmax(360px, 0.85fr); }
  .two-col.data-workspace { grid-template-columns: minmax(760px, 1.35fr) minmax(420px, 0.85fr); align-items: start; }
  .scroll-box { max-height: 300px; overflow: auto; border: 1px solid var(--border); border-radius: var(--radius); background: #fff; }
  .scroll-box.field-map-scroll { max-height: 520px; }
  .data-preview-card { position: sticky; top: 0; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 12px; }
  .data-preview-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 10px; }
  .data-preview-head h3 { margin: 0; }
  #data-preview-frame {
    min-height: 460px; max-height: 65vh; overflow: auto; display: flex; align-items: flex-start;
    justify-content: center; background: var(--shell); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 12px;
  }
  #data-preview-img { display: none; max-width: 100%; background: #fff; border: 1px solid var(--border); box-shadow: 0 10px 26px rgba(1,102,106,0.16); }
  #data-preview-img.visible { display: block; }
  #data-preview-empty { margin: auto; color: var(--text-dim); font-size: 12px; text-align: center; }

  /* ── Editor panel ── */
  .editor-panel {
    display: flex; flex-direction: column;
    border-right: 1px solid var(--border); overflow: hidden;
    background: var(--surface);
  }

  .panel-header {
    flex-shrink: 0; padding: 8px 14px; background: var(--surface2);
    border-bottom: 1px solid var(--border); font-size: 11px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.8px; color: var(--text-dim);
    display: flex; align-items: center; gap: 6px;
  }

  .panel-header .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--coral); }

  #zpl-input {
    flex: 1; background: #fff; color: var(--text);
    font-family: var(--font-mono); font-size: 13px; line-height: 1.6;
    padding: 16px; border: none; outline: none; resize: none;
    tab-size: 2; overflow: auto; caret-color: var(--accent);
  }

  #zpl-input::selection { background: rgba(254,199,194,0.65); }

  /* ── Settings bar ── */
  .settings-bar {
    flex-shrink: 0; background: var(--surface);
    border-top: 1px solid var(--border);
    padding: 9px 14px; display: flex; flex-wrap: wrap; gap: 10px; align-items: center;
  }

  .sg { display: flex; align-items: center; gap: 5px; }

  .sg label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.5px; color: var(--text-dim); white-space: nowrap;
  }

  .sg input, .sg select {
    background: #fff; border: 1px solid var(--border);
    border-radius: 5px; color: var(--text); font-size: 12px;
    font-family: var(--font-ui); padding: 4px 7px; outline: none;
    transition: border-color 0.15s;
  }

  .sg input:focus, .sg select:focus { border-color: var(--accent); }
  .sg input[type="number"] { width: 58px; }
  .size-sep { color: var(--text-dim); font-size: 13px; }
  .unit-label { font-size: 11px; color: var(--text-dim); }

  #open-file-btn {
    background: #fff; color: var(--deep-teal);
    border: 1px solid var(--border); border-radius: 6px;
    padding: 6px 13px; font-size: 12px; font-weight: 600;
    font-family: var(--font-ui); cursor: pointer;
    display: flex; align-items: center; gap: 6px;
    transition: border-color 0.15s, color 0.15s; white-space: nowrap;
  }

  #open-file-btn:hover { border-color: var(--lagoon); color: var(--lagoon); }

  #file-input { display: none; }

  #render-btn {
    margin-left: auto; background: var(--coral); color: #11181f;
    border: none; border-radius: 6px; padding: 7px 18px;
    font-size: 13px; font-weight: 600; font-family: var(--font-ui);
    cursor: pointer; display: flex; align-items: center; gap: 6px;
    transition: background 0.15s, transform 0.1s; white-space: nowrap;
  }

  #render-btn:hover:not(:disabled) { background: #ff8a6d; }
  #render-btn:active:not(:disabled) { transform: scale(0.97); }
  #render-btn:disabled { opacity: 0.55; cursor: not-allowed; }

  .shortcut {
    font-size: 10px; opacity: 0.7;
    background: rgba(17,24,31,0.12); border-radius: 3px; padding: 1px 4px;
  }

  /* ── Preview panel ── */
  .preview-panel {
    display: flex; flex-direction: column;
    background: var(--shell); overflow: hidden; position: relative;
  }

  #preview-scroll {
    flex: 1; overflow-y: auto;
    display: flex; flex-direction: column; align-items: center;
    padding: 20px 20px 0; gap: 16px;
  }

  .empty-state { margin: auto; text-align: center; color: var(--text-dim); }
  .empty-state svg { opacity: 0.2; margin-bottom: 12px; display: block; margin-left: auto; margin-right: auto; }
  .empty-state p { font-size: 13px; }

  #loading {
    display: none; position: absolute; inset: 0;
    background: rgba(250,245,242,0.82); align-items: center;
    justify-content: center; flex-direction: column; gap: 12px; z-index: 10;
  }

  #loading.active { display: flex; }

  .spinner {
    width: 32px; height: 32px; border: 3px solid var(--border);
    border-top-color: var(--accent); border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .loading-text { color: var(--text-dim); font-size: 13px; }

  #error-banner {
    display: none; width: 100%;
    background: rgba(224,85,85,0.10); border: 1px solid var(--error);
    border-radius: var(--radius); padding: 12px 16px;
    color: var(--error); font-size: 12px; font-family: var(--font-mono);
    word-break: break-word; white-space: pre-wrap;
  }

  #error-banner.visible { display: block; }

  #preview-img {
    display: none; max-width: 100%;
    border: 1px solid var(--border); border-radius: 4px;
    box-shadow: 0 12px 32px rgba(1,102,106,0.18); background: #fff;
  }

  #preview-img.visible { display: block; }

  /* ── Download bar ── */
  #download-bar {
    display: none; width: 100%;
    padding: 14px 0 20px; margin-top: 4px;
    border-top: 1px solid var(--border);
    gap: 10px; justify-content: center; align-items: center; flex-wrap: wrap;
  }

  #download-bar.visible { display: flex; }

  .dl-btn {
    display: inline-flex; align-items: center; gap: 7px;
    padding: 8px 20px; border-radius: 6px; font-size: 13px;
    font-weight: 600; font-family: var(--font-ui);
    cursor: pointer; border: none; transition: background 0.15s, opacity 0.15s;
    text-decoration: none;
  }

  .dl-btn-png {
    background: #fff; color: var(--deep-teal); border: 1px solid var(--border);
  }

  .dl-btn-png:hover { border-color: var(--lagoon); color: var(--lagoon); }

  .dl-btn-pdf { background: var(--deep-teal); color: #fff; }
  .dl-btn-pdf:hover:not(:disabled) { background: #014e52; }
  .dl-btn-pdf:disabled { opacity: 0.55; cursor: not-allowed; }

  .mini-spinner {
    display: none; width: 13px; height: 13px;
    border: 2px solid rgba(255,255,255,0.3); border-top-color: #fff;
    border-radius: 50%; animation: spin 0.7s linear infinite;
  }

  .dl-btn-pdf.loading .mini-spinner { display: inline-block; }
  .dl-btn-pdf.loading .dl-icon-pdf  { display: none; }

  /* ── Status bar ── */
  .status-bar {
    flex-shrink: 0; background: var(--surface);
    border-top: 1px solid var(--border);
    padding: 4px 14px; font-size: 11px; color: var(--text-dim);
    display: flex; gap: 16px; align-items: center;
  }

  .status-ok  { color: var(--success); }
  .status-err { color: var(--error); }
</style>
</head>
<body>

<header>
  <div class="logo">
    <span class="logo-icon">L</span>
    <span class="logo-text">Label Platform</span>
  </div>
  <span class="tagline">Operations Workbench</span>
  <span class="header-spacer"></span>
  <span class="badge">Coral Reef</span>
</header>

<nav class="module-nav" aria-label="Product modules">
  <button class="module-tab active" data-module="design">Label 制作</button>
  <button class="module-tab" data-module="data">数据源处理</button>
  <button class="module-tab" data-module="api">API 接口测试</button>
  <button class="module-tab" data-module="print">打印配置网关</button>
  <button class="module-tab" data-module="monitor">接口日志监控</button>
</nav>

<main class="workspace">
<section class="module-panel active" data-panel="design">
<div class="designer-grid">
  <!-- ── Left: Editor + Settings ── -->
  <div class="editor-panel">
    <div class="panel-header">
      <span class="dot"></span>Template Source
    </div>

    <textarea id="zpl-input" spellcheck="false" autocomplete="off" autocorrect="off" autocapitalize="off">^XA
^FX Top section with logo, name and address.
^CF0,60
^FO50,50^GB100,100,100^FS
^FO75,75^FR^GB100,100,100^FS
^FO93,93^GB40,40,40^FS
^FO220,50^FDIntershipping, Inc.^FS
^CF0,30
^FO220,115^FD1000 Shipping Lane^FS
^FO220,155^FDShelbyville TN 38102^FS
^FO220,195^FDUnited States (USA)^FS
^FO50,250^GB700,3,3^FS
^FX Second section with recipient address and permit information.
^CFA,30
^FO50,300^FDJohn Doe^FS
^FO50,340^FD100 Main Street^FS
^FO50,380^FDSpringfield TN 39021^FS
^FO50,420^FDUnited States (USA)^FS
^CFA,15
^FO600,300^GB150,150,3^FS
^FO638,340^FDPermit^FS
^FO638,390^FD123456^FS
^FO50,500^GB700,3,3^FS
^FX Third section with bar code.
^BY5,2,270
^FO100,550^BC^FD12345678^FS
^FO100,850^BY2,3,60^BQ,,2^FDQA,https://github.com/GOODBOY008/labelize^FS
^XZ</textarea>

    <div class="settings-bar">
      <div class="sg">
        <label for="fmt">Format</label>
        <select id="fmt">
          <option value="zpl" selected>ZPL</option>
          <option value="epl">EPL</option>
        </select>
      </div>

      <div class="sg">
        <label for="size-preset">Size</label>
        <select id="size-preset">
          <option value="4x6"     selected>4 &times; 6 in</option>
          <option value="4x4">4 &times; 4 in</option>
          <option value="4x3">4 &times; 3 in</option>
          <option value="2x4">2 &times; 4 in</option>
          <option value="2x2">2 &times; 2 in</option>
          <option value="3.5x1.5">3.5 &times; 1.5 in</option>
          <option value="custom">Custom&hellip;</option>
        </select>
      </div>

      <div class="sg" id="custom-size" style="display:none">
        <label for="width-in">W</label>
        <input id="width-in" type="number" value="4" min="0.5" max="15" step="0.1">
        <span class="size-sep">&times;</span>
        <label for="height-in">H</label>
        <input id="height-in" type="number" value="6" min="0.5" max="15" step="0.1">
        <span class="unit-label">in</span>
      </div>

      <div class="sg">
        <label for="dpmm">dpmm</label>
        <select id="dpmm">
          <option value="6">6</option>
          <option value="8" selected>8</option>
          <option value="12">12</option>
          <option value="24">24</option>
        </select>
      </div>

      <input id="file-input" type="file" accept=".zpl,.epl">
      <button id="open-file-btn">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        Open ZPL
      </button>

      <button id="render-btn">
        &#9654; Preview <span class="shortcut">Ctrl+&#9166;</span>
      </button>
    </div>
  </div>

  <!-- ── Right: Preview ── -->
  <div class="preview-panel">
    <div class="panel-header">
      <span class="dot" style="background:var(--lagoon)"></span>Label Preview
      <span id="status-text" style="margin-left:auto;font-size:11px;text-transform:none;letter-spacing:0;font-weight:400"></span>
    </div>

    <div id="loading">
      <div class="spinner"></div>
      <span class="loading-text">Rendering&hellip;</span>
    </div>

    <div id="preview-scroll">
      <div class="empty-state" id="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <line x1="3" y1="9" x2="21" y2="9"/>
          <line x1="9" y1="21" x2="9" y2="9"/>
        </svg>
        <p>Press <strong>Preview</strong> to render the label</p>
        <p style="font-size:11px;margin-top:6px;opacity:0.55">Ctrl+Enter</p>
      </div>

      <div id="error-banner"></div>
      <img id="preview-img" alt="Rendered label">

      <!-- Download bar — appears after a successful render -->
      <div id="download-bar">
        <a id="dl-png" class="dl-btn dl-btn-png" href="#" download="label.png">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Save PNG
        </a>

        <button id="dl-pdf" class="dl-btn dl-btn-pdf">
          <svg class="dl-icon-pdf" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          <span class="mini-spinner"></span>
          Save PDF
        </button>
      </div>
    </div><!-- /preview-scroll -->

    <div class="status-bar">
      <span id="status-size"></span>
      <span id="status-time"></span>
    </div>
  </div>
</div>
</section>

<section class="module-panel" data-panel="data">
  <div class="module-dashboard">
    <div class="module-title">
      <div>
        <h2>数据源处理</h2>
        <p>把业务数据变成模板可用字段，支持人工确认字段来源、复用客户/供应商/仓库/流程配置，以及后续批量生成标签。</p>
      </div>
      <span class="status-pill">Field Mapping</span>
    </div>
    <div class="context-bar">
      <div><strong id="current-template-name">No template imported</strong><br><span id="current-template-meta">Open a ZPL file in Label 制作 to start configuration.</span></div>
      <span id="current-template-id" class="chip warn">not saved</span>
    </div>
    <div class="form-grid">
      <div class="form-field"><label for="config-name">Config Name</label><input id="config-name" value="Default workflow mapping"></div>
      <div class="form-field"><label for="config-customer">Customer</label><input id="config-customer" placeholder="SAIC USA"></div>
      <div class="form-field"><label for="config-warehouse">Warehouse</label><input id="config-warehouse" placeholder="WH-A"></div>
      <div class="form-field"><label for="config-process">Business Process</label><input id="config-process" placeholder="Inbound / Shipping"></div>
    </div>
    <div class="action-row">
      <button id="extract-fields-btn" class="action-btn dark">Extract Fields From ZPL</button>
      <button id="add-field-btn" class="action-btn">Add Field Row</button>
      <button id="apply-fields-btn" class="action-btn">Apply Field Edits To ZPL</button>
      <button id="preview-fields-btn" class="action-btn dark">Preview With Fields</button>
      <button id="save-config-btn" class="action-btn primary">Save Data Source Config</button>
      <button id="load-config-btn" class="action-btn">Reload Saved Configs</button>
      <span id="config-status" class="chip">waiting</span>
    </div>
    <div class="two-col data-workspace">
      <div>
        <h3>字段确认表单</h3>
        <div class="scroll-box field-map-scroll">
          <table class="module-table field-map-table">
            <thead><tr><th>Field</th><th>Value</th><th>Source</th><th>API Name</th><th>Action</th></tr></thead>
            <tbody id="field-map-body"><tr><td colspan="5">No extracted fields yet.</td></tr></tbody>
          </table>
        </div>
      </div>
      <div>
        <div class="data-preview-card">
          <div class="data-preview-head">
            <h3>Label 预览</h3>
            <span id="data-preview-status" class="chip">waiting</span>
          </div>
          <div id="data-preview-frame">
            <div id="data-preview-empty">Edit field values, then preview the generated label.</div>
            <img id="data-preview-img" alt="Data mapped label preview">
          </div>
        </div>
        <h3 style="margin-top:12px">可复用配置</h3>
        <div class="scroll-box" style="max-height:190px">
          <table class="module-table">
            <thead><tr><th>Name</th><th>Scope</th><th>Fields</th><th>Status</th></tr></thead>
            <tbody id="config-list-body"><tr><td colspan="4">No saved configs.</td></tr></tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="module-panel" data-panel="api">
  <div class="module-dashboard">
    <div class="module-title">
      <div>
        <h2>API 接口封装测试</h2>
        <p>面向业务系统提供统一 RESTful 接口，支持请求示例生成、PDF 测试输出、ZPL/PNG/PDF 返回和打印任务创建。</p>
      </div>
      <span class="status-pill">API Console</span>
    </div>
    <div class="context-bar">
      <div><strong id="api-template-name">No template selected</strong><br><span>Use current imported template and saved data source config.</span></div>
      <span id="api-mode-chip" class="chip ok">pdf_preview</span>
    </div>
    <div class="two-col">
      <div class="module-card compact">
        <h3>Request JSON</h3>
        <div class="form-field">
          <textarea id="api-request-json" spellcheck="false">{
  "template_id": "",
  "delivery_mode": "pdf_preview",
  "data": {}
}</textarea>
        </div>
        <div class="action-row">
          <button id="build-api-request-btn" class="action-btn">Build From Mapping</button>
          <button id="api-pdf-btn" class="action-btn primary">POST PDF Test</button>
          <button id="api-print-btn" class="action-btn dark">POST Print Task</button>
        </div>
      </div>
      <div class="module-card compact">
        <h3>Response</h3>
        <div id="api-response-box" class="response-box">Waiting for request...</div>
        <div class="action-row">
          <a id="api-output-link" class="action-btn" href="#" target="_blank" style="display:none;text-decoration:none">Open Output</a>
          <button id="copy-curl-btn" class="action-btn">Generate cURL</button>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="module-panel" data-panel="print">
  <div class="module-dashboard">
    <div class="module-title">
      <div>
        <h2>打印配置网关</h2>
        <p>管理打印机、模板绑定、路由规则和打印任务队列。物理设备无响应时不能阻塞 API 服务，必须醒目标识待处理任务。</p>
      </div>
      <span class="status-pill">Queue Control</span>
    </div>
    <div class="alert-strip">
      <strong>打印队列需要优先可见</strong>
      批量任务积压、设备离线、任务失败或多次重试时，界面顶部要显示红/橙告警，并允许一键筛选到受影响任务后重试、改派、暂停、取消或补打。
    </div>
    <div class="metric-row">
      <div class="metric"><strong id="metric-queue-depth">0</strong><span>Queued</span></div>
      <div class="metric"><strong id="metric-attention">0</strong><span>Need Attention</span></div>
      <div class="metric"><strong id="metric-retry">0</strong><span>Retry Pending</span></div>
      <div class="metric"><strong id="metric-health">Healthy</strong><span>Queue Health</span></div>
    </div>
    <div class="form-grid">
      <div class="form-field"><label for="printer-id">Printer ID</label><input id="printer-id" value="warehouse_a_01"></div>
      <div class="form-field"><label for="printer-ip">IP</label><input id="printer-ip" value="192.168.1.50"></div>
      <div class="form-field"><label for="printer-port">Port</label><input id="printer-port" value="9100"></div>
      <div class="form-field"><label for="printer-site">Site / Warehouse</label><input id="printer-site" value="WH-A"></div>
    </div>
    <div class="action-row">
      <button id="save-printer-btn" class="action-btn dark">Save Printer</button>
      <button id="enqueue-print-btn" class="action-btn primary">Create Print Task From Current Label</button>
      <button id="refresh-tasks-btn" class="action-btn">Refresh Queue</button>
      <span id="print-action-status" class="chip">independent service</span>
    </div>
    <div class="two-col">
      <div>
        <h3>打印机配置</h3>
        <div class="scroll-box">
          <table class="module-table">
            <thead><tr><th>Printer</th><th>Endpoint</th><th>Site</th><th>Status</th></tr></thead>
            <tbody id="printer-list-body"><tr><td colspan="4">No printers saved.</td></tr></tbody>
          </table>
        </div>
      </div>
      <div>
        <h3>打印任务队列</h3>
        <div class="scroll-box">
          <table class="module-table">
            <thead><tr><th>Task</th><th>Template</th><th>Printer</th><th>Status</th></tr></thead>
            <tbody id="print-task-body"><tr><td colspan="4">No queued tasks.</td></tr></tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="module-panel" data-panel="monitor">
  <div class="module-dashboard">
    <div class="module-title">
      <div>
        <h2>接口日志监控</h2>
        <p>集中查看 API 请求日志、渲染日志、打印任务日志和错误重试记录，并在 Dashboard 中展示成功率、失败率、耗时和调用量。</p>
      </div>
      <span class="status-pill">Observability</span>
    </div>
    <div class="metric-row">
      <div class="metric"><strong id="metric-total-calls">0</strong><span>Total Calls</span></div>
      <div class="metric"><strong id="metric-success-rate">0%</strong><span>Success Rate</span></div>
      <div class="metric"><strong id="metric-failure-rate">0%</strong><span>Failure Rate</span></div>
      <div class="metric"><strong id="metric-render-count">0</strong><span>Render Count</span></div>
    </div>
    <div id="monitor-alerts" class="alert-strip" style="display:none"></div>
    <div class="action-row">
      <button id="refresh-logs-btn" class="action-btn dark">Refresh Logs</button>
      <span class="chip ok">logs run independently</span>
    </div>
    <div class="dashboard-grid">
      <div>
        <h3>API 请求日志</h3>
        <div class="scroll-box">
          <table class="module-table">
            <thead><tr><th>Request</th><th>Endpoint</th><th>Status</th></tr></thead>
            <tbody id="api-log-body"><tr><td colspan="3">No API logs.</td></tr></tbody>
          </table>
        </div>
      </div>
      <div>
        <h3>渲染日志</h3>
        <div class="scroll-box">
          <table class="module-table">
            <thead><tr><th>Request</th><th>Template</th><th>Output</th></tr></thead>
            <tbody id="render-log-body"><tr><td colspan="3">No render logs.</td></tr></tbody>
          </table>
        </div>
      </div>
      <div>
        <h3>打印任务日志</h3>
        <div class="scroll-box">
          <table class="module-table">
            <thead><tr><th>Task</th><th>Printer</th><th>Status</th></tr></thead>
            <tbody id="monitor-print-body"><tr><td colspan="3">No print logs.</td></tr></tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</section>
</main>

<script>
(function () {
  "use strict";

  var btn        = document.getElementById("render-btn");
  var openFileBtn = document.getElementById("open-file-btn");
  var fileInput   = document.getElementById("file-input");
  var input      = document.getElementById("zpl-input");
  var loading    = document.getElementById("loading");
  var emptyState = document.getElementById("empty-state");
  var errBanner  = document.getElementById("error-banner");
  var previewImg = document.getElementById("preview-img");
  var dlBar      = document.getElementById("download-bar");
  var dlPng      = document.getElementById("dl-png");
  var dlPdf      = document.getElementById("dl-pdf");
  var sizePreset = document.getElementById("size-preset");
  var customSize = document.getElementById("custom-size");
  var widthIn    = document.getElementById("width-in");
  var heightIn   = document.getElementById("height-in");
  var statusText = document.getElementById("status-text");
  var statusSize = document.getElementById("status-size");
  var statusTime = document.getElementById("status-time");

  var pngBlobUrl = null;
  var currentTemplate = { id: "", name: "", content: "" };
  var currentFields = [];

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function slugify(value) {
    return String(value || "template")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "_")
      .replace(/^_+|_+$/g, "")
      .slice(0, 54) || "template";
  }

  function readStore(key, fallback) {
    try {
      var raw = localStorage.getItem(key);
      return raw ? JSON.parse(raw) : fallback;
    } catch (_) {
      return fallback;
    }
  }

  function writeStore(key, value) {
    localStorage.setItem(key, JSON.stringify(value));
  }

  function updateTemplateContext() {
    var name = currentTemplate.name || "No template imported";
    var id = currentTemplate.id || "not saved";
    setText("current-template-name", name);
    setText("current-template-id", id);
    setText("api-template-name", name);
    setText("current-template-meta", currentTemplate.id
      ? "Template is saved and available for data mapping, API tests, and print tasks."
      : "Open a ZPL file in Label 制作 to start configuration.");
  }

  function setStatus(id, text, className) {
    var node = document.getElementById(id);
    if (!node) return;
    node.textContent = text;
    node.className = className || "chip";
  }

  function importTemplate(name, content) {
    var params = getParams();
    var id = "tpl_" + slugify(name);
    return fetch("/api/v1/templates/import", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: id,
        name: name,
        content: content,
        width_mm: params.w_mm,
        height_mm: params.h_mm,
        dpmm: params.dpmm
      })
    })
    .then(function (res) {
      if (!res.ok) return res.text().then(function (txt) { throw new Error(txt || res.status); });
      return res.json();
    })
    .then(function (body) {
      currentTemplate = { id: body.id || id, name: name, content: content };
      updateTemplateContext();
      extractFields();
      buildApiRequest("pdf_preview");
      refreshDashboard();
      refreshLogs();
      return currentTemplate;
    });
  }

  function ensureTemplateSaved() {
    var content = input.value.trim();
    if (!content) return Promise.reject(new Error("ZPL content is empty."));
    if (currentTemplate.id && currentTemplate.content === content) {
      return Promise.resolve(currentTemplate);
    }
    var fallbackName = currentTemplate.name || "Working ZPL Template";
    return importTemplate(fallbackName, content);
  }

  function fieldName(value, index) {
    var cleaned = String(value || "")
      .replace(/\{\{|\}\}/g, "")
      .replace(/[^a-zA-Z0-9]+/g, "_")
      .replace(/^_+|_+$/g, "")
      .toLowerCase();
    if (!cleaned || cleaned.length > 28) cleaned = "field_" + (index + 1);
    return cleaned;
  }

  function extractFields() {
    var zpl = input.value;
    var found = [];
    var seen = {};
    var re = /\^FD([\s\S]*?)\^FS/g;
    var match;
    while ((match = re.exec(zpl)) && found.length < 80) {
      var value = match[1].trim();
      if (!value) continue;
      found.push({
        label: "field_" + (found.length + 1),
        originalValue: value,
        value: value,
        source: value.indexOf("{{") >= 0 ? "api_field" : "fixed",
        apiName: fieldName(value, found.length),
        start: match.index + 3,
        end: match.index + 3 + match[1].length,
        deleted: false,
        manual: false
      });
    }
    currentFields = found;
    renderFieldRows();
    return found;
  }

  function renderFieldRows() {
    var body = document.getElementById("field-map-body");
    if (!body) return;
    var visibleFields = currentFields.filter(function (field) { return !field.deleted; });
    if (!visibleFields.length) {
      body.innerHTML = '<tr><td colspan="5">No extracted fields yet.</td></tr>';
      return;
    }
    body.innerHTML = currentFields.map(function (field, index) {
      if (field.deleted) return "";
      return '<tr>' +
        '<td>' + escapeHtml(field.label) + '</td>' +
        '<td><textarea class="field-value-input" data-index="' + index + '">' + escapeHtml(field.value) + '</textarea></td>' +
        '<td><select class="field-source" data-index="' + index + '">' +
          ["manual", "api_field", "fixed", "ignored"].map(function (source) {
            return '<option value="' + source + '"' + (field.source === source ? " selected" : "") + '>' + source + '</option>';
          }).join("") +
        '</select></td>' +
        '<td><input class="field-api-name" data-index="' + index + '" value="' + escapeHtml(field.apiName) + '"></td>' +
        '<td><button class="row-action delete-field-btn" data-index="' + index + '">Delete</button></td>' +
      '</tr>';
    }).join("");
  }

  function collectFieldMappings() {
    currentFields.forEach(function (field, index) {
      if (field.deleted) return;
      var source = document.querySelector('.field-source[data-index="' + index + '"]');
      var apiName = document.querySelector('.field-api-name[data-index="' + index + '"]');
      var value = document.querySelector('.field-value-input[data-index="' + index + '"]');
      if (source) field.source = source.value;
      if (apiName) field.apiName = apiName.value.trim() || field.apiName;
      if (value) field.value = value.value;
    });
    return currentFields;
  }

  function saveDataConfig() {
    collectFieldMappings();
    var configs = readStore("label_platform_data_configs", []);
    var activeFields = currentFields.filter(function (field) { return !field.deleted; });
    var cfg = {
      id: "cfg_" + Date.now(),
      template_id: currentTemplate.id,
      template_name: currentTemplate.name,
      name: document.getElementById("config-name").value || "Default mapping",
      customer: document.getElementById("config-customer").value,
      warehouse: document.getElementById("config-warehouse").value,
      process: document.getElementById("config-process").value,
      fields: activeFields,
      status: "active"
    };
    configs.unshift(cfg);
    writeStore("label_platform_data_configs", configs.slice(0, 20));
    renderConfigList();
    buildApiRequest("pdf_preview");
    setStatus("config-status", "saved", "chip ok");
  }

  function renderConfigList() {
    var body = document.getElementById("config-list-body");
    if (!body) return;
    var configs = readStore("label_platform_data_configs", []);
    if (!configs.length) {
      body.innerHTML = '<tr><td colspan="4">No saved configs.</td></tr>';
      return;
    }
    body.innerHTML = configs.map(function (cfg) {
      var scope = [cfg.customer, cfg.warehouse, cfg.process].filter(Boolean).join(" / ") || "general";
      return '<tr><td>' + escapeHtml(cfg.name) + '</td><td>' + escapeHtml(scope) + '</td><td>' + cfg.fields.length + '</td><td><span class="chip ok">' + cfg.status + '</span></td></tr>';
    }).join("");
  }

  function dataFromMappings() {
    var data = {};
    collectFieldMappings().forEach(function (field) {
      if (!field.deleted && field.source === "api_field") data[field.apiName] = field.value;
    });
    return data;
  }

  function buildMappedZpl() {
    collectFieldMappings();
    var base = input.value;
    var replacements = currentFields
      .filter(function (field) {
        return !field.deleted && !field.manual && typeof field.start === "number" && field.source !== "ignored";
      })
      .sort(function (a, b) { return b.start - a.start; });
    replacements.forEach(function (field) {
      base = base.slice(0, field.start) + field.value + base.slice(field.end);
    });
    return base;
  }

  function addManualField() {
    collectFieldMappings();
    var index = currentFields.length;
    currentFields.push({
      label: "field_" + (index + 1),
      originalValue: "",
      value: "",
      source: "api_field",
      apiName: "field_" + (index + 1),
      start: null,
      end: null,
      deleted: false,
      manual: true
    });
    renderFieldRows();
    setStatus("config-status", "field row added", "chip ok");
  }

  function deleteField(index) {
    if (!currentFields[index]) return;
    currentFields[index].deleted = true;
    renderFieldRows();
    setStatus("config-status", "field row deleted", "chip warn");
  }

  function applyFieldEditsToTemplate() {
    var mapped = buildMappedZpl();
    input.value = mapped;
    currentTemplate.content = "";
    extractFields();
    setStatus("config-status", "field edits applied to ZPL", "chip ok");
    return ensureTemplateSaved();
  }

  function previewFieldMappedLabel() {
    var params = getParams();
    var img = document.getElementById("data-preview-img");
    var empty = document.getElementById("data-preview-empty");
    setStatus("data-preview-status", "rendering", "chip warn");
    fetch(buildUrl(params, null), {
      method: "POST",
      headers: { "Content-Type": ctFor(params.fmt) },
      body: buildMappedZpl()
    })
    .then(function (res) {
      if (!res.ok) return res.text().then(function (txt) { throw new Error(txt || res.status); });
      return res.blob();
    })
    .then(function (blob) {
      var url = URL.createObjectURL(blob);
      img.src = url;
      img.classList.add("visible");
      empty.style.display = "none";
      setStatus("data-preview-status", "preview ready", "chip ok");
    })
    .catch(function (err) {
      img.classList.remove("visible");
      empty.style.display = "block";
      empty.textContent = "Preview failed: " + err.message;
      setStatus("data-preview-status", "preview failed", "chip bad");
    });
  }

  function buildApiRequest(mode) {
    mode = mode || "pdf_preview";
    var body = {
      template_id: currentTemplate.id,
      delivery_mode: mode,
      data: dataFromMappings(),
      manual_values: {}
    };
    if (mode === "device_print") {
      body.printer_id = document.getElementById("printer-id").value || "warehouse_a_01";
      body.copies = 1;
    }
    var node = document.getElementById("api-request-json");
    if (node) node.value = JSON.stringify(body, null, 2);
    setText("api-mode-chip", mode);
    return body;
  }

  function postApiRequest(mode) {
    ensureTemplateSaved()
      .then(function () {
        var body = buildApiRequest(mode);
        var responseBox = document.getElementById("api-response-box");
        var outputLink = document.getElementById("api-output-link");
        if (responseBox) responseBox.textContent = "Posting...";
        if (outputLink) outputLink.style.display = "none";
        return fetch("/api/v1/labels/print", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body)
        });
      })
      .then(function (res) { return res.json().then(function (body) { return { status: res.status, body: body }; }); })
      .then(function (result) {
        document.getElementById("api-response-box").textContent = JSON.stringify(result.body, null, 2);
        if (result.body.output_url) {
          var outputLink = document.getElementById("api-output-link");
          outputLink.href = result.body.output_url;
          outputLink.style.display = "inline-flex";
        }
        refreshDashboard();
        refreshTasks();
        refreshLogs();
      })
      .catch(function (err) {
        document.getElementById("api-response-box").textContent = "Error: " + err.message;
      });
  }

  function renderPrinters() {
    var body = document.getElementById("printer-list-body");
    if (!body) return;
    var printers = readStore("label_platform_printers", []);
    if (!printers.length) {
      body.innerHTML = '<tr><td colspan="4">No printers saved.</td></tr>';
      return;
    }
    body.innerHTML = printers.map(function (printer) {
      return '<tr><td>' + escapeHtml(printer.id) + '</td><td>' + escapeHtml(printer.ip + ":" + printer.port) + '</td><td>' + escapeHtml(printer.site) + '</td><td><span class="chip ok">ready</span></td></tr>';
    }).join("");
  }

  function savePrinter() {
    var printers = readStore("label_platform_printers", []);
    var printer = {
      id: document.getElementById("printer-id").value || "warehouse_a_01",
      ip: document.getElementById("printer-ip").value || "192.168.1.50",
      port: document.getElementById("printer-port").value || "9100",
      site: document.getElementById("printer-site").value || "WH-A"
    };
    printers = printers.filter(function (item) { return item.id !== printer.id; });
    printers.unshift(printer);
    writeStore("label_platform_printers", printers);
    renderPrinters();
    setStatus("print-action-status", "printer saved", "chip ok");
  }

  function enqueuePrintTask() {
    savePrinter();
    setStatus("print-action-status", "enqueueing", "chip warn");
    ensureTemplateSaved()
      .then(function () {
        return fetch("/api/v1/labels/print", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(buildApiRequest("device_print"))
        });
      })
      .then(function (res) { return res.json(); })
      .then(function (body) {
        setStatus("print-action-status", body.print_task_id ? "queued " + body.print_task_id : "submitted", "chip ok");
        refreshDashboard();
        refreshTasks();
        refreshLogs();
      })
      .catch(function (err) {
        setStatus("print-action-status", err.message, "chip bad");
      });
  }

  function refreshTasks() {
    fetch("/api/v1/logs/print-tasks")
      .then(function (res) { return res.ok ? res.json() : { items: [] }; })
      .then(function (body) {
        var rows = (body.items || []).slice().reverse();
        var printBody = document.getElementById("print-task-body");
        var monitorBody = document.getElementById("monitor-print-body");
        var html = rows.length ? rows.map(function (task) {
          return '<tr><td>' + escapeHtml(task.id) + '</td><td>' + escapeHtml(task.template_id) + '</td><td>' + escapeHtml(task.printer_id || "") + '</td><td><span class="chip warn">' + escapeHtml(task.status) + '</span></td></tr>';
        }).join("") : '<tr><td colspan="4">No queued tasks.</td></tr>';
        if (printBody) printBody.innerHTML = html;
        if (monitorBody) {
          monitorBody.innerHTML = rows.length ? rows.map(function (task) {
            return '<tr><td>' + escapeHtml(task.id) + '</td><td>' + escapeHtml(task.printer_id || "") + '</td><td>' + escapeHtml(task.status) + '</td></tr>';
          }).join("") : '<tr><td colspan="3">No print logs.</td></tr>';
        }
      })
      .catch(function () {});
  }

  function refreshLogs() {
    refreshTasks();
    fetch("/api/v1/logs/api-requests")
      .then(function (res) { return res.ok ? res.json() : { items: [] }; })
      .then(function (body) {
        var rows = (body.items || []).slice().reverse();
        var target = document.getElementById("api-log-body");
        if (target) target.innerHTML = rows.length ? rows.map(function (log) {
          return '<tr><td>' + escapeHtml(log.request_id) + '</td><td>' + escapeHtml(log.endpoint) + '</td><td>' + escapeHtml(log.status_code + " " + log.status) + '</td></tr>';
        }).join("") : '<tr><td colspan="3">No API logs.</td></tr>';
      })
      .catch(function () {});

    fetch("/api/v1/logs/renders")
      .then(function (res) { return res.ok ? res.json() : { items: [] }; })
      .then(function (body) {
        var rows = (body.items || []).slice().reverse();
        var target = document.getElementById("render-log-body");
        if (target) target.innerHTML = rows.length ? rows.map(function (log) {
          return '<tr><td>' + escapeHtml(log.request_id) + '</td><td>' + escapeHtml(log.template_id) + '</td><td>' + escapeHtml(log.output_type) + '</td></tr>';
        }).join("") : '<tr><td colspan="3">No render logs.</td></tr>';
      })
      .catch(function () {});
  }

  function setupModules() {
    var tabs = Array.prototype.slice.call(document.querySelectorAll(".module-tab"));
    var panels = Array.prototype.slice.call(document.querySelectorAll(".module-panel"));
    tabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        var module = tab.getAttribute("data-module");
        tabs.forEach(function (item) { item.classList.toggle("active", item === tab); });
        panels.forEach(function (panel) {
          panel.classList.toggle("active", panel.getAttribute("data-panel") === module);
        });
        if (module === "print" || module === "monitor") refreshDashboard();
      });
    });
  }

  function setText(id, value) {
    var node = document.getElementById(id);
    if (node) node.textContent = value;
  }

  function percent(value) {
    return Math.round((Number(value) || 0) * 100) + "%";
  }

  function titleCase(value) {
    value = String(value || "healthy");
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function refreshDashboard() {
    fetch("/api/v1/dashboard/summary")
      .then(function (res) { return res.ok ? res.json() : null; })
      .then(function (summary) {
        if (!summary) return;
        var health = summary.print_queue_health || {};
        setText("metric-queue-depth", health.queue_depth || 0);
        setText("metric-attention", health.needs_attention_count || 0);
        setText("metric-retry", health.retry_pending_count || 0);
        setText("metric-health", titleCase(health.status));
        setText("metric-total-calls", summary.total_calls || 0);
        setText("metric-success-rate", percent(summary.success_rate));
        setText("metric-failure-rate", percent(summary.failure_rate));
        setText("metric-render-count", summary.render_count || 0);

        var alerts = health.alerts || [];
        var alertBox = document.getElementById("monitor-alerts");
        if (alertBox) {
          alertBox.style.display = alerts.length ? "block" : "none";
          alertBox.innerHTML = alerts.length
            ? "<strong>" + alerts.length + " queue alert(s)</strong>" + alerts.map(function (a) {
                return "<div>" + a.severity + " - " + a.title + " - " + a.action + "</div>";
              }).join("")
            : "";
        }
      })
      .catch(function () {});
  }

  var SIZE_PRESETS = {
    "4x6":     [4,   6],
    "4x4":     [4,   4],
    "4x3":     [4,   3],
    "2x4":     [2,   4],
    "2x2":     [2,   2],
    "3.5x1.5": [3.5, 1.5]
  };

  sizePreset.addEventListener("change", function () {
    var isCustom = this.value === "custom";
    customSize.style.display = isCustom ? "flex" : "none";
    if (!isCustom) {
      var wh = SIZE_PRESETS[this.value];
      widthIn.value  = wh[0];
      heightIn.value = wh[1];
    }
  });

  function getParams() {
    var fmt   = document.getElementById("fmt").value;
    var dpmm  = parseInt(document.getElementById("dpmm").value, 10) || 8;
    var w_in  = parseFloat(widthIn.value)  || 4;
    var h_in  = parseFloat(heightIn.value) || 6;
    var w_mm  = +(w_in * 25.4).toFixed(2);
    var h_mm  = +(h_in * 25.4).toFixed(2);
    return { fmt: fmt, dpmm: dpmm, w_mm: w_mm, h_mm: h_mm };
  }

  function buildUrl(p, output) {
    var u = "/convert?width=" + p.w_mm + "&height=" + p.h_mm + "&dpmm=" + p.dpmm;
    if (output) u += "&output=" + output;
    return u;
  }

  function ctFor(fmt) {
    return fmt === "epl" ? "application/epl" : "application/zpl";
  }

  function clearPreview() {
    errBanner.classList.remove("visible");
    previewImg.classList.remove("visible");
    dlBar.classList.remove("visible");
    emptyState.style.display = "none";
    if (pngBlobUrl) { URL.revokeObjectURL(pngBlobUrl); pngBlobUrl = null; }
  }

  function showError(msg) {
    clearPreview();
    errBanner.textContent = msg;
    errBanner.classList.add("visible");
    statusText.textContent = "Error";
    statusText.className = "status-err";
  }

  /* ── Render (PNG) ── */
  function render() {
    var zpl = input.value.trim();
    if (!zpl) { showError("Editor is empty — paste some ZPL or EPL first."); return; }

    var params = getParams();
    clearPreview();
    loading.classList.add("active");
    btn.disabled = true;
    statusText.textContent = "Rendering\u2026";
    statusText.className = "";
    statusSize.textContent = "";
    statusTime.textContent = "";

    var t0 = performance.now();

    fetch(buildUrl(params, null), {
      method: "POST",
      headers: { "Content-Type": ctFor(params.fmt) },
      body: zpl
    })
    .then(function (res) {
      var elapsed = Math.round(performance.now() - t0);
      loading.classList.remove("active");
      btn.disabled = false;
      if (!res.ok) {
        return res.text().then(function (txt) {
          showError("Server error " + res.status + ": " + txt);
        });
      }
      return res.blob().then(function (blob) {
        pngBlobUrl     = URL.createObjectURL(blob);
        dlPng.href     = pngBlobUrl;
        previewImg.src = pngBlobUrl;
        previewImg.classList.add("visible");
        dlBar.classList.add("visible");
        statusSize.textContent = "PNG  " + (blob.size / 1024).toFixed(1) + " KB";
        statusTime.textContent = elapsed + " ms";
        statusText.textContent = "OK";
        statusText.className   = "status-ok";
      });
    })
    .catch(function (err) {
      loading.classList.remove("active");
      btn.disabled = false;
      showError("Network error: " + err.message);
    });
  }

  /* ── PDF download (lazy) ── */
  dlPdf.addEventListener("click", function () {
    var zpl = input.value.trim();
    if (!zpl) return;
    var params = getParams();
    dlPdf.classList.add("loading");
    dlPdf.disabled = true;

    fetch(buildUrl(params, "pdf"), {
      method: "POST",
      headers: { "Content-Type": ctFor(params.fmt) },
      body: zpl
    })
    .then(function (res) {
      dlPdf.classList.remove("loading");
      dlPdf.disabled = false;
      if (!res.ok) {
        return res.text().then(function (txt) {
          showError("PDF error " + res.status + ": " + txt);
        });
      }
      return res.blob().then(function (blob) {
        var url = URL.createObjectURL(blob);
        var a   = document.createElement("a");
        a.href     = url;
        a.download = "label.pdf";
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        setTimeout(function () { URL.revokeObjectURL(url); }, 10000);
      });
    })
    .catch(function (err) {
      dlPdf.classList.remove("loading");
      dlPdf.disabled = false;
      showError("PDF download error: " + err.message);
    });
  });

  /* ── Open File ── */
  openFileBtn.addEventListener("click", function () { fileInput.click(); });

  fileInput.addEventListener("change", function () {
    var file = this.files && this.files[0];
    if (!file) return;
    var ext = file.name.split(".").pop().toLowerCase();
    var fmtSel = document.getElementById("fmt");
    if (ext === "epl") fmtSel.value = "epl";
    else               fmtSel.value = "zpl";
    var reader = new FileReader();
    reader.onload = function (e) {
      input.value = e.target.result;
      input.focus();
      importTemplate(file.name.replace(/\.(zpl|epl)$/i, ""), input.value)
        .then(function () {
          setStatus("config-status", "template imported", "chip ok");
        })
        .catch(function (err) {
          setStatus("config-status", "import failed", "chip bad");
          showError("Template import failed: " + err.message);
        });
    };
    reader.readAsText(file);
    this.value = "";
  });

  btn.addEventListener("click", render);

  document.getElementById("extract-fields-btn").addEventListener("click", function () {
    extractFields();
    setStatus("config-status", currentFields.length + " fields extracted", "chip ok");
    previewFieldMappedLabel();
  });

  document.getElementById("add-field-btn").addEventListener("click", addManualField);

  document.getElementById("apply-fields-btn").addEventListener("click", function () {
    applyFieldEditsToTemplate()
      .then(function () {
        previewFieldMappedLabel();
        buildApiRequest("pdf_preview");
      })
      .catch(function (err) { setStatus("config-status", err.message, "chip bad"); });
  });

  document.getElementById("preview-fields-btn").addEventListener("click", previewFieldMappedLabel);

  document.getElementById("save-config-btn").addEventListener("click", function () {
    ensureTemplateSaved()
      .then(function () {
        saveDataConfig();
        previewFieldMappedLabel();
      })
      .catch(function (err) { setStatus("config-status", err.message, "chip bad"); });
  });

  document.getElementById("load-config-btn").addEventListener("click", renderConfigList);

  document.getElementById("build-api-request-btn").addEventListener("click", function () {
    ensureTemplateSaved()
      .then(function () { buildApiRequest("pdf_preview"); })
      .catch(function (err) { document.getElementById("api-response-box").textContent = "Error: " + err.message; });
  });

  document.getElementById("api-pdf-btn").addEventListener("click", function () {
    postApiRequest("pdf_preview");
  });

  document.getElementById("api-print-btn").addEventListener("click", function () {
    postApiRequest("device_print");
  });

  document.getElementById("copy-curl-btn").addEventListener("click", function () {
    var body = document.getElementById("api-request-json").value;
    var curl = "curl -X POST http://127.0.0.1:8081/api/v1/labels/print -H 'Content-Type: application/json' -d '" + body.replace(/'/g, "'\\''") + "'";
    document.getElementById("api-response-box").textContent = curl;
  });

  document.getElementById("save-printer-btn").addEventListener("click", savePrinter);
  document.getElementById("enqueue-print-btn").addEventListener("click", enqueuePrintTask);
  document.getElementById("refresh-tasks-btn").addEventListener("click", function () {
    refreshDashboard();
    refreshTasks();
  });
  document.getElementById("refresh-logs-btn").addEventListener("click", function () {
    refreshDashboard();
    refreshLogs();
  });

  document.getElementById("field-map-body").addEventListener("click", function (event) {
    if (event.target && event.target.classList.contains("delete-field-btn")) {
      deleteField(parseInt(event.target.getAttribute("data-index"), 10));
    }
  });

  document.getElementById("field-map-body").addEventListener("input", function (event) {
    if (event.target && (event.target.classList.contains("field-value-input") || event.target.classList.contains("field-api-name"))) {
      setStatus("data-preview-status", "needs preview", "chip warn");
      setStatus("config-status", "edited", "chip warn");
    }
  });

  document.getElementById("field-map-body").addEventListener("change", function (event) {
    if (event.target && event.target.classList.contains("field-source")) {
      setStatus("data-preview-status", "needs preview", "chip warn");
      setStatus("config-status", "edited", "chip warn");
    }
  });

  input.addEventListener("keydown", function (e) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      render();
    }
  });

  setupModules();
  updateTemplateContext();
  extractFields();
  renderConfigList();
  renderPrinters();
  buildApiRequest("pdf_preview");
  previewFieldMappedLabel();
  refreshDashboard();
  refreshLogs();
  setInterval(function () {
    refreshDashboard();
    refreshTasks();
  }, 10000);
})();
</script>
</body>
</html>
"##;
