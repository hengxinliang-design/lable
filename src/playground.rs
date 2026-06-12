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
  .metric-row { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-bottom: 12px; }
  .metric { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 12px; }
  .metric strong { display: block; font-size: 24px; color: var(--deep-teal); line-height: 1.1; }
  .metric span { color: var(--text-dim); font-size: 12px; }
  .alert-strip { border: 2px solid var(--coral); background: #fff1ee; border-radius: var(--radius); padding: 13px 14px; margin-bottom: 12px; }
  .alert-strip strong { display: block; color: var(--error); margin-bottom: 4px; }
  .module-table { width: 100%; border-collapse: collapse; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); overflow: hidden; }
  .module-table th, .module-table td { border-bottom: 1px solid var(--border); padding: 9px 10px; text-align: left; font-size: 12px; }
  .module-table th { background: var(--surface2); color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.4px; }
  .module-table tr:last-child td { border-bottom: none; }
  .chip { display: inline-flex; align-items: center; border-radius: 999px; padding: 2px 8px; font-size: 11px; font-weight: 700; background: var(--surface2); color: var(--text-dim); }
  .chip.ok { background: #e4f4f6; color: var(--deep-teal); }
  .chip.warn { background: var(--coral-light); color: #5b241b; }

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
    <div class="dashboard-grid">
      <div class="module-card">
        <h3>字段确认</h3>
        <ul>
          <li>从 ZPL 文本、条码、二维码提取候选字段</li>
          <li>标记 manual、api_field、fixed、ignored</li>
          <li>确认后生成 API 请求字段模型</li>
        </ul>
      </div>
      <div class="module-card">
        <h3>配置复用</h3>
        <ul>
          <li>按客户、供应商、仓库、业务流程保存</li>
          <li>支持搜索、复制、启用、停用和归档</li>
          <li>模板版本变化时只处理差异字段</li>
        </ul>
      </div>
      <div class="module-card">
        <h3>批量生成</h3>
        <ul>
          <li>接收 JSON、CSV、Excel 或 API 结果</li>
          <li>行级校验，支持部分失败</li>
          <li>有效行可进入渲染或打印队列</li>
        </ul>
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
    <div class="dashboard-grid">
      <div class="module-card">
        <h3>调试页面</h3>
        <p>选择模板和数据源配置后自动生成请求 JSON，业务人员可先生成 PDF 标签确认，不触发物理打印。</p>
      </div>
      <div class="module-card">
        <h3>返回类型</h3>
        <p><span class="chip ok">PNG</span> <span class="chip ok">PDF</span> <span class="chip ok">ZPL</span> <span class="chip warn">Print Task ID</span></p>
      </div>
      <div class="module-card">
        <h3>接口规范</h3>
        <p>API Key 鉴权、参数校验、稳定错误码、request_id 贯穿日志和任务链路。</p>
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
    <table class="module-table">
      <thead><tr><th>能力</th><th>当前设计</th><th>人工动作</th></tr></thead>
      <tbody>
        <tr><td>打印机配置</td><td>IP、端口、DPI/DPMM、纸张、型号、站点、仓库、QZ Tray</td><td>测试连接 / 停用</td></tr>
        <tr><td>模板绑定</td><td>模板允许打印机、默认打印机、纸张和份数覆盖</td><td>改派 / 管理绑定</td></tr>
        <tr><td>路由规则</td><td>按仓库、站点、业务类型、客户、供应商、模板、优先级匹配</td><td>启停规则 / 调整优先级</td></tr>
        <tr><td>任务队列</td><td>queued、dispatching、sent、completed、failed、blocked、device_offline</td><td>重试 / 暂停 / 取消 / 补打</td></tr>
      </tbody>
    </table>
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
    <table class="module-table">
      <thead><tr><th>日志类型</th><th>筛选条件</th><th>定位动作</th></tr></thead>
      <tbody>
        <tr><td>API 请求日志</td><td>接口、调用方、状态码、request_id、时间</td><td>查看请求与错误码</td></tr>
        <tr><td>渲染日志</td><td>模板、输出类型、DPMM、耗时、错误</td><td>打开输出或错误详情</td></tr>
        <tr><td>打印任务日志</td><td>打印机、路由、状态、重试次数、通道</td><td>进入任务详情并恢复</td></tr>
      </tbody>
    </table>
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
    };
    reader.readAsText(file);
    this.value = "";
  });

  btn.addEventListener("click", render);

  input.addEventListener("keydown", function (e) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      render();
    }
  });

  setupModules();
  refreshDashboard();
})();
</script>
</body>
</html>
"##;
