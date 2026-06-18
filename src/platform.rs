use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{DrawerOptions, Renderer, ZplParser};

#[derive(Clone)]
pub struct PlatformState {
    inner: Arc<PlatformInner>,
}

struct PlatformInner {
    seq: AtomicU64,
    store: Mutex<Store>,
}

#[derive(Default)]
struct Store {
    templates: HashMap<String, Template>,
    outputs: HashMap<String, OutputArtifact>,
    api_logs: Vec<ApiRequestLog>,
    render_logs: Vec<RenderLog>,
    print_tasks: Vec<PrintTask>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub content: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub dpmm: i32,
    pub status: String,
}

#[derive(Clone, Debug, Serialize)]
struct TemplateSummary {
    id: String,
    name: String,
    width_mm: f64,
    height_mm: f64,
    dpmm: i32,
    status: String,
}

#[derive(Clone, Debug)]
struct OutputArtifact {
    content_type: &'static str,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
struct ApiRequestLog {
    id: String,
    request_id: String,
    endpoint: String,
    status: String,
    status_code: u16,
    duration_ms: u64,
    error_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct RenderLog {
    id: String,
    request_id: String,
    template_id: String,
    output_type: String,
    status: String,
    duration_ms: u64,
    error_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct PrintTask {
    id: String,
    request_id: String,
    template_id: String,
    printer_id: Option<String>,
    delivery_mode: String,
    connection_mode: String,
    copies: u32,
    label_data: Value,
    field_schema: Value,
    status: String,
    retry_count: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct QueueAlert {
    severity: String,
    code: String,
    title: String,
    message: String,
    action: String,
    task_filter: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct QueueHealth {
    status: String,
    queue_depth: usize,
    needs_attention_count: usize,
    retry_pending_count: usize,
    alerts: Vec<QueueAlert>,
}

#[derive(Debug, Deserialize)]
pub struct ImportTemplateRequest {
    pub id: Option<String>,
    pub name: String,
    pub content: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub dpmm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    pub template_id: String,
    pub output: Option<String>,
    pub response_mode: Option<String>,
    pub size: Option<RenderSize>,
    #[serde(default)]
    pub field_schema: Value,
    #[serde(default)]
    pub data: Value,
    #[serde(default)]
    pub manual_values: Value,
}

#[derive(Debug, Deserialize)]
pub struct RenderSize {
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub dpmm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PrintRequest {
    pub template_id: String,
    pub delivery_mode: Option<String>,
    pub printer_id: Option<String>,
    pub connection_mode: Option<String>,
    pub copies: Option<u32>,
    #[serde(default)]
    pub field_schema: Value,
    #[serde(default)]
    pub data: Value,
    #[serde(default)]
    pub manual_values: Value,
}

#[derive(Debug, Deserialize)]
struct ExportDataConfigRequest {
    output_dir: Option<String>,
    config_name: String,
    template_id: Option<String>,
    template_name: Option<String>,
    zpl_content: String,
    config: Value,
}

#[derive(Debug, Deserialize)]
struct ApiExampleRequest {
    template_id: String,
    operation: String,
    language: Option<String>,
    output: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    request_id: String,
    status: &'static str,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
    details: Vec<Value>,
}

impl PlatformState {
    pub fn new() -> Self {
        let mut store = Store::default();
        seed_sample_templates(&mut store);
        Self {
            inner: Arc::new(PlatformInner {
                seq: AtomicU64::new(1),
                store: Mutex::new(store),
            }),
        }
    }

    fn next_id(&self, prefix: &str) -> String {
        let id = self.inner.seq.fetch_add(1, Ordering::SeqCst);
        format!("{}_{:016}", prefix, id)
    }
}

impl Default for PlatformState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router(state: PlatformState) -> Router {
    Router::new()
        .route(
            "/api/v1/templates",
            get(list_templates).post(import_template),
        )
        .route("/api/v1/templates/import", post(import_template))
        .route("/api/v1/labels/render", post(render_label_handler))
        .route("/api/v1/labels/test-pdf", post(test_pdf_handler))
        .route("/api/v1/labels/print", post(print_label_handler))
        .route("/api/v1/data-configs/export", post(export_data_config))
        .route("/api/v1/api-examples", post(api_example_handler))
        .route("/api/v1/requests/{request_id}/output", get(get_output))
        .route("/api/v1/logs/api-requests", get(list_api_logs))
        .route("/api/v1/logs/renders", get(list_render_logs))
        .route("/api/v1/logs/print-tasks", get(list_print_tasks))
        .route("/api/v1/dashboard/summary", get(dashboard_summary))
        .with_state(state)
}

async fn list_templates(State(state): State<PlatformState>) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    let items: Vec<_> = store
        .templates
        .values()
        .map(|template| TemplateSummary {
            id: template.id.clone(),
            name: template.name.clone(),
            width_mm: template.width_mm,
            height_mm: template.height_mm,
            dpmm: template.dpmm,
            status: template.status.clone(),
        })
        .collect();
    Json(json!({ "items": items, "total": items.len() }))
}

async fn import_template(
    State(state): State<PlatformState>,
    Json(req): Json<ImportTemplateRequest>,
) -> impl IntoResponse {
    let request_id = state.next_id("req");
    if req.content.trim().is_empty() {
        return api_error(
            &request_id,
            StatusCode::BAD_REQUEST,
            "VALIDATION_REQUIRED_FIELD",
            "content is required",
        );
    }

    let id = req.id.unwrap_or_else(|| slugify(&req.name));
    let template = Template {
        id: id.clone(),
        name: req.name,
        content: req.content,
        width_mm: req.width_mm.unwrap_or(102.0),
        height_mm: req.height_mm.unwrap_or(152.0),
        dpmm: req.dpmm.unwrap_or(12),
        status: "active".to_string(),
    };
    let mut store = state.inner.store.lock().expect("store lock");
    store.templates.insert(id.clone(), template);
    log_api(
        &mut store,
        request_id.clone(),
        "templates.import",
        "success",
        200,
        None,
    );

    (
        StatusCode::CREATED,
        Json(json!({ "request_id": request_id, "id": id })),
    )
        .into_response()
}

async fn render_label_handler(
    State(state): State<PlatformState>,
    headers: HeaderMap,
    Json(req): Json<RenderRequest>,
) -> impl IntoResponse {
    let request_id = request_id(&state, &headers);
    let output = req.output.as_deref().unwrap_or("pdf");
    match render_from_request(&state, &request_id, &req, output) {
        Ok(artifact) => {
            let output_url = store_output(&state, &request_id, artifact);
            let mut store = state.inner.store.lock().expect("store lock");
            log_api(
                &mut store,
                request_id.clone(),
                "labels.render",
                "success",
                200,
                None,
            );
            Json(json!({
                "request_id": request_id,
                "status": "success",
                "output_type": output,
                "response_mode": req.response_mode.unwrap_or_else(|| "url".to_string()),
                "output_url": output_url
            }))
            .into_response()
        }
        Err((status, code, message)) => api_error(&request_id, status, code, message),
    }
}

async fn test_pdf_handler(
    State(state): State<PlatformState>,
    headers: HeaderMap,
    Json(mut req): Json<RenderRequest>,
) -> impl IntoResponse {
    req.output = Some("pdf".to_string());
    let request_id = request_id(&state, &headers);
    match render_from_request(&state, &request_id, &req, "pdf") {
        Ok(artifact) => {
            let output_url = store_output(&state, &request_id, artifact);
            let mut store = state.inner.store.lock().expect("store lock");
            log_api(
                &mut store,
                request_id.clone(),
                "labels.test_pdf",
                "success",
                200,
                None,
            );
            Json(json!({
                "request_id": request_id,
                "status": "success",
                "output_type": "pdf",
                "output_url": output_url
            }))
            .into_response()
        }
        Err((status, code, message)) => api_error(&request_id, status, code, message),
    }
}

async fn export_data_config(
    State(state): State<PlatformState>,
    headers: HeaderMap,
    Json(req): Json<ExportDataConfigRequest>,
) -> impl IntoResponse {
    let request_id = request_id(&state, &headers);
    if req.zpl_content.trim().is_empty() {
        return api_error(
            &request_id,
            StatusCode::BAD_REQUEST,
            "VALIDATION_REQUIRED_FIELD",
            "zpl_content is required",
        );
    }

    match write_data_config_export(&req) {
        Ok((config_path, zpl_path)) => {
            let mut store = state.inner.store.lock().expect("store lock");
            log_api(
                &mut store,
                request_id.clone(),
                "data-configs.export",
                "success",
                200,
                None,
            );
            Json(json!({
                "request_id": request_id,
                "status": "success",
                "config_path": config_path,
                "zpl_path": zpl_path
            }))
            .into_response()
        }
        Err(message) => api_error_owned(
            &request_id,
            StatusCode::INTERNAL_SERVER_ERROR,
            "EXPORT_FAILED",
            message,
        ),
    }
}

async fn print_label_handler(
    State(state): State<PlatformState>,
    headers: HeaderMap,
    Json(req): Json<PrintRequest>,
) -> impl IntoResponse {
    let request_id = request_id(&state, &headers);
    let delivery_mode = req
        .delivery_mode
        .clone()
        .unwrap_or_else(|| "device_print".to_string());
    let connection_mode = req
        .connection_mode
        .clone()
        .unwrap_or_else(|| "print_server".to_string());

    if !is_allowed_connection_mode(&connection_mode) {
        return api_error(
            &request_id,
            StatusCode::BAD_REQUEST,
            "VALIDATION_INVALID_CONNECTION_MODE",
            "unsupported connection_mode",
        );
    }

    if delivery_mode == "pdf_preview" {
        let render_req = RenderRequest {
            template_id: req.template_id,
            output: Some("pdf".to_string()),
            response_mode: Some("url".to_string()),
            size: None,
            field_schema: req.field_schema,
            data: req.data,
            manual_values: req.manual_values,
        };
        return match render_from_request(&state, &request_id, &render_req, "pdf") {
            Ok(artifact) => {
                let output_url = store_output(&state, &request_id, artifact);
                let mut store = state.inner.store.lock().expect("store lock");
                log_api(
                    &mut store,
                    request_id.clone(),
                    "labels.print.preview",
                    "success",
                    200,
                    None,
                );
                Json(json!({
                    "request_id": request_id,
                    "status": "success",
                    "delivery_mode": "pdf_preview",
                    "output_type": "pdf",
                    "output_url": output_url
                }))
                .into_response()
            }
            Err((status, code, message)) => api_error(&request_id, status, code, message),
        };
    }

    if req.printer_id.as_deref().unwrap_or("").is_empty() {
        return api_error(
            &request_id,
            StatusCode::BAD_REQUEST,
            "VALIDATION_REQUIRED_FIELD",
            "printer_id is required for device_print",
        );
    }

    let print_task_id = state.next_id("pt");
    let mut store = state.inner.store.lock().expect("store lock");
    store.print_tasks.push(PrintTask {
        id: print_task_id.clone(),
        request_id: request_id.clone(),
        template_id: req.template_id,
        printer_id: req.printer_id,
        delivery_mode,
        connection_mode: connection_mode.clone(),
        copies: req.copies.unwrap_or(1),
        label_data: req.data,
        field_schema: req.field_schema,
        status: "queued".to_string(),
        retry_count: 0,
    });
    log_api(
        &mut store,
        request_id.clone(),
        "labels.print",
        "success",
        202,
        None,
    );

    (
        StatusCode::ACCEPTED,
        Json(json!({
            "request_id": request_id,
            "print_task_id": print_task_id,
            "status": "queued",
            "connection_mode": connection_mode
        })),
    )
        .into_response()
}

fn is_allowed_connection_mode(mode: &str) -> bool {
    matches!(mode, "print_server" | "direct_ip" | "qz_tray" | "pdf_only")
}

async fn api_example_handler(Json(req): Json<ApiExampleRequest>) -> impl IntoResponse {
    let language = req.language.unwrap_or_else(|| "curl".to_string());
    let output = req.output.unwrap_or_else(|| "pdf".to_string());
    let body = json!({
        "template_id": req.template_id,
        "output": output,
        "data": {
            "order_no": "SO-10001",
            "barcode": "1234567890"
        }
    });
    let example = match language.as_str() {
        "python" => format!(
            "requests.post('/api/v1/labels/render', headers={{'Authorization':'Bearer <api_key>'}}, json={})",
            body
        ),
        "javascript" | "js" => format!(
            "fetch('/api/v1/labels/render', {{ method: 'POST', headers: {{ 'Authorization': 'Bearer <api_key>', 'Content-Type': 'application/json' }}, body: JSON.stringify({}) }})",
            body
        ),
        _ => format!(
            "curl -X POST /api/v1/labels/render -H 'Authorization: Bearer <api_key>' -H 'Content-Type: application/json' -d '{}'",
            body
        ),
    };

    Json(json!({
        "operation": req.operation,
        "language": language,
        "example": example
    }))
}

async fn get_output(
    State(state): State<PlatformState>,
    Path(request_id): Path<String>,
) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    match store.outputs.get(&request_id) {
        Some(output) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, output.content_type)],
            Bytes::from(output.bytes.clone()),
        )
            .into_response(),
        None => api_error(
            &request_id,
            StatusCode::NOT_FOUND,
            "OUTPUT_NOT_FOUND",
            "output not found",
        ),
    }
}

async fn list_api_logs(State(state): State<PlatformState>) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    Json(json!({ "items": store.api_logs, "total": store.api_logs.len() }))
}

async fn list_render_logs(State(state): State<PlatformState>) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    Json(json!({ "items": store.render_logs, "total": store.render_logs.len() }))
}

async fn list_print_tasks(State(state): State<PlatformState>) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    Json(json!({ "items": store.print_tasks, "total": store.print_tasks.len() }))
}

async fn dashboard_summary(State(state): State<PlatformState>) -> impl IntoResponse {
    let store = state.inner.store.lock().expect("store lock");
    let total_calls = store.api_logs.len();
    let failures = store
        .api_logs
        .iter()
        .filter(|l| l.status != "success")
        .count();
    let success = total_calls.saturating_sub(failures);
    let avg_duration_ms = if total_calls == 0 {
        0
    } else {
        store.api_logs.iter().map(|l| l.duration_ms).sum::<u64>() / total_calls as u64
    };

    let queue_health = print_queue_health(&store.print_tasks);

    Json(json!({
        "total_calls": total_calls,
        "success_count": success,
        "failure_count": failures,
        "success_rate": if total_calls == 0 { 0.0 } else { success as f64 / total_calls as f64 },
        "failure_rate": if total_calls == 0 { 0.0 } else { failures as f64 / total_calls as f64 },
        "avg_duration_ms": avg_duration_ms,
        "render_count": store.render_logs.len(),
        "print_task_count": store.print_tasks.len(),
        "queue_depth": queue_health.queue_depth,
        "print_queue_health": queue_health
    }))
}

fn print_queue_health(tasks: &[PrintTask]) -> QueueHealth {
    let queue_depth = tasks.iter().filter(|t| t.status == "queued").count();
    let needs_attention_count = tasks
        .iter()
        .filter(|t| matches!(t.status.as_str(), "failed" | "blocked" | "device_offline"))
        .count();
    let retry_pending_count = tasks
        .iter()
        .filter(|t| t.retry_count > 0 && matches!(t.status.as_str(), "queued" | "retrying"))
        .count();

    let mut alerts = Vec::new();
    if needs_attention_count > 0 {
        alerts.push(QueueAlert {
            severity: "critical".to_string(),
            code: "PRINT_TASKS_NEED_ATTENTION".to_string(),
            title: "Print tasks need manual attention".to_string(),
            message: format!(
                "{} print task(s) are failed, blocked, or waiting on an offline device.",
                needs_attention_count
            ),
            action: "Open the print queue filtered by failed, blocked, and device_offline tasks."
                .to_string(),
            task_filter: "status:failed,blocked,device_offline".to_string(),
        });
    }

    if queue_depth >= 100 {
        alerts.push(QueueAlert {
            severity: "critical".to_string(),
            code: "PRINT_QUEUE_BACKLOG_CRITICAL".to_string(),
            title: "Print queue backlog is critical".to_string(),
            message: format!("{} print task(s) are waiting in queue.", queue_depth),
            action: "Pause non-urgent routes, verify printer health, and dispatch recovery work."
                .to_string(),
            task_filter: "status:queued".to_string(),
        });
    } else if queue_depth >= 25 {
        alerts.push(QueueAlert {
            severity: "warning".to_string(),
            code: "PRINT_QUEUE_BACKLOG_WARNING".to_string(),
            title: "Print queue backlog is growing".to_string(),
            message: format!("{} print task(s) are waiting in queue.", queue_depth),
            action: "Review route capacity and printer availability before the backlog grows."
                .to_string(),
            task_filter: "status:queued".to_string(),
        });
    }

    if retry_pending_count > 0 {
        alerts.push(QueueAlert {
            severity: "warning".to_string(),
            code: "PRINT_RETRY_PENDING".to_string(),
            title: "Print retries are pending".to_string(),
            message: format!(
                "{} queued or retrying task(s) have already attempted printing.",
                retry_pending_count
            ),
            action: "Open retry tasks, check the last error, then retry or reroute them."
                .to_string(),
            task_filter: "retry_count:>0,status:queued,retrying".to_string(),
        });
    }

    let status = if alerts.iter().any(|a| a.severity == "critical") {
        "critical"
    } else if alerts.iter().any(|a| a.severity == "warning") {
        "warning"
    } else {
        "healthy"
    };

    QueueHealth {
        status: status.to_string(),
        queue_depth,
        needs_attention_count,
        retry_pending_count,
        alerts,
    }
}

fn render_from_request(
    state: &PlatformState,
    request_id: &str,
    req: &RenderRequest,
    output: &str,
) -> Result<OutputArtifact, (StatusCode, &'static str, &'static str)> {
    let template = {
        let store = state.inner.store.lock().expect("store lock");
        store.templates.get(&req.template_id).cloned()
    }
    .ok_or((
        StatusCode::NOT_FOUND,
        "TEMPLATE_NOT_FOUND",
        "template not found",
    ))?;

    if template.status != "active" {
        return Err((
            StatusCode::BAD_REQUEST,
            "TEMPLATE_INACTIVE",
            "template is inactive",
        ));
    }
    if !matches!(output, "png" | "pdf" | "zpl") {
        return Err((
            StatusCode::BAD_REQUEST,
            "VALIDATION_INVALID_OUTPUT",
            "output must be png, pdf, or zpl",
        ));
    }

    let content = merge_values(
        &merge_schema_values(
            &template.content,
            &req.field_schema,
            &req.data,
            &req.manual_values,
        ),
        &req.data,
        &req.manual_values,
    );
    if output == "zpl" {
        return Ok(OutputArtifact {
            content_type: "text/plain; charset=utf-8",
            bytes: content.into_bytes(),
        });
    }

    let labels = ZplParser::new().parse(content.as_bytes()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "RENDER_FAILED",
            "failed to parse ZPL",
        )
    })?;
    let label =
        labels
            .first()
            .ok_or((StatusCode::BAD_REQUEST, "RENDER_FAILED", "no labels found"))?;
    let options = DrawerOptions {
        label_width_mm: req
            .size
            .as_ref()
            .and_then(|s| s.width_mm)
            .unwrap_or(template.width_mm),
        label_height_mm: req
            .size
            .as_ref()
            .and_then(|s| s.height_mm)
            .unwrap_or(template.height_mm),
        dpmm: req
            .size
            .as_ref()
            .and_then(|s| s.dpmm)
            .unwrap_or(template.dpmm),
        ..Default::default()
    };

    let renderer = Renderer::new();
    let mut png_buf = Cursor::new(Vec::new());
    renderer
        .draw_label_as_png(label, &mut png_buf, options.clone())
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "RENDER_FAILED",
                "failed to render label",
            )
        })?;

    let artifact = if output == "pdf" {
        let img = image::load_from_memory(&png_buf.into_inner())
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "RENDER_FAILED",
                    "failed to decode rendered PNG",
                )
            })?
            .to_rgba8();
        let mut pdf_buf = Cursor::new(Vec::new());
        crate::encode_pdf(&img, &options, &mut pdf_buf).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "RENDER_FAILED",
                "failed to encode PDF",
            )
        })?;
        OutputArtifact {
            content_type: "application/pdf",
            bytes: pdf_buf.into_inner(),
        }
    } else {
        OutputArtifact {
            content_type: "image/png",
            bytes: png_buf.into_inner(),
        }
    };

    let mut store = state.inner.store.lock().expect("store lock");
    let render_id = state.next_id("render");
    store.render_logs.push(RenderLog {
        id: render_id,
        request_id: request_id.to_string(),
        template_id: template.id,
        output_type: output.to_string(),
        status: "success".to_string(),
        duration_ms: 0,
        error_code: None,
    });
    Ok(artifact)
}

fn store_output(state: &PlatformState, request_id: &str, artifact: OutputArtifact) -> String {
    let mut store = state.inner.store.lock().expect("store lock");
    store.outputs.insert(request_id.to_string(), artifact);
    format!("/api/v1/requests/{}/output", request_id)
}

fn merge_values(content: &str, data: &Value, manual_values: &Value) -> String {
    let mut merged = content.to_string();
    for source in [data, manual_values] {
        if let Value::Object(map) = source {
            for (key, value) in map {
                let replacement = value
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| value.to_string());
                merged = merged.replace(&format!("{{{{{}}}}}", key), &replacement);
            }
        }
    }
    merged
}

fn merge_schema_values(
    content: &str,
    field_schema: &Value,
    data: &Value,
    manual_values: &Value,
) -> String {
    let mut merged = content.to_string();
    let Value::Array(fields) = field_schema else {
        return merged;
    };

    for field in fields {
        let Some(name) = field.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(sample_value) = field.get("sample_value").and_then(Value::as_str) else {
            continue;
        };
        if sample_value.is_empty() {
            continue;
        }
        let replacement = data
            .get(name)
            .or_else(|| manual_values.get(name))
            .map(json_value_to_label_value);
        if let Some(replacement) = replacement {
            merged = merged.replace(sample_value, &replacement);
        }
    }

    merged
}

fn json_value_to_label_value(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn write_data_config_export(req: &ExportDataConfigRequest) -> Result<(String, String), String> {
    let output_dir = export_output_dir(req.output_dir.as_deref())?;
    fs::create_dir_all(&output_dir).map_err(|err| format!("create export directory: {err}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("clock error: {err}"))?
        .as_secs();
    let base_name = sanitize_export_name(&format!(
        "{}_{}_{}",
        req.config_name,
        req.template_name.as_deref().unwrap_or("template"),
        timestamp
    ));
    let config_path = output_dir.join(format!("{base_name}.json"));
    let zpl_path = output_dir.join(format!("{base_name}.zpl"));

    let export_payload = json!({
        "config_name": req.config_name,
        "template_id": req.template_id,
        "template_name": req.template_name,
        "config": req.config,
        "zpl_file": zpl_path.to_string_lossy()
    });
    let config_bytes = serde_json::to_vec_pretty(&export_payload)
        .map_err(|err| format!("serialize config: {err}"))?;
    fs::write(&config_path, config_bytes).map_err(|err| format!("write config: {err}"))?;
    fs::write(&zpl_path, req.zpl_content.as_bytes()).map_err(|err| format!("write zpl: {err}"))?;

    Ok((
        config_path.to_string_lossy().to_string(),
        zpl_path.to_string_lossy().to_string(),
    ))
}

fn export_output_dir(raw: Option<&str>) -> Result<PathBuf, String> {
    let value = raw.unwrap_or("").trim();
    if value.is_empty() {
        return std::env::current_dir()
            .map(|dir| dir.join("exports").join("data-source-configs"))
            .map_err(|err| format!("resolve current directory: {err}"));
    }
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join(rest));
        }
    }
    Ok(PathBuf::from(value))
}

fn sanitize_export_name(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('_');
    if trimmed.is_empty() {
        "data_source_config".to_string()
    } else {
        trimmed.chars().take(96).collect()
    }
}

fn api_error(
    request_id: &str,
    status: StatusCode,
    code: &'static str,
    message: &'static str,
) -> axum::response::Response {
    (
        status,
        Json(ErrorBody {
            request_id: request_id.to_string(),
            status: "error",
            error: ErrorDetail {
                code,
                message: message.to_string(),
                details: Vec::new(),
            },
        }),
    )
        .into_response()
}

fn api_error_owned(
    request_id: &str,
    status: StatusCode,
    code: &'static str,
    message: String,
) -> axum::response::Response {
    (
        status,
        Json(ErrorBody {
            request_id: request_id.to_string(),
            status: "error",
            error: ErrorDetail {
                code,
                message,
                details: Vec::new(),
            },
        }),
    )
        .into_response()
}

fn request_id(state: &PlatformState, headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| state.next_id("req"))
}

fn log_api(
    store: &mut Store,
    request_id: String,
    endpoint: &str,
    status: &str,
    status_code: u16,
    error_code: Option<String>,
) {
    let id = format!("log_{:04}", store.api_logs.len() + 1);
    store.api_logs.push(ApiRequestLog {
        id,
        request_id,
        endpoint: endpoint.to_string(),
        status: status.to_string(),
        status_code,
        duration_ms: 0,
        error_code,
    });
}

fn seed_sample_templates(store: &mut Store) {
    let content = include_str!("../testdata/samples/z5z_01_gm_300_master.zpl");
    store.templates.insert(
        "z5z_01_gm_300_master".to_string(),
        Template {
            id: "z5z_01_gm_300_master".to_string(),
            name: "Z5Z 01 GM 300 master".to_string(),
            content: content.to_string(),
            width_mm: 102.0,
            height_mm: 152.0,
            dpmm: 12,
            status: "active".to_string(),
        },
    );
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_values_replaces_data_and_manual_placeholders() {
        let out = merge_values(
            "^FD{{order_no}} {{operator}}^FS",
            &json!({ "order_no": "SO-1" }),
            &json!({ "operator": "Ann" }),
        );
        assert_eq!(out, "^FDSO-1 Ann^FS");
    }

    #[test]
    fn merge_schema_values_replaces_sample_values_with_api_data() {
        let out = merge_schema_values(
            "^FDP06512515AA^FS^FDQ380^FS",
            &json!([
                { "name": "barcode_b3_2", "sample_value": "P06512515AA" },
                { "name": "barcode_b3_6", "sample_value": "Q380" }
            ]),
            &json!({
                "barcode_b3_2": "P99999999AA",
                "barcode_b3_6": "Q888"
            }),
            &json!({}),
        );

        assert_eq!(out, "^FDP99999999AA^FS^FDQ888^FS");
    }

    #[test]
    fn data_config_export_writes_json_and_zpl_files() {
        let dir = std::env::temp_dir().join(format!(
            "labelize_export_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let req = ExportDataConfigRequest {
            output_dir: Some(dir.to_string_lossy().to_string()),
            config_name: "Default workflow mapping".to_string(),
            template_id: Some("tpl_dcx".to_string()),
            template_name: Some("Z5Z_01_DCX_300 single".to_string()),
            zpl_content: "^XA^FD1000000BB^FS^XZ".to_string(),
            config: json!({ "fields": [{ "apiName": "field_1", "value": "1000000BB" }] }),
        };

        let (config_path, zpl_path) = write_data_config_export(&req).expect("export writes files");
        let config_text = fs::read_to_string(config_path).expect("config file readable");
        let zpl_text = fs::read_to_string(zpl_path).expect("zpl file readable");

        assert!(config_text.contains("Default workflow mapping"));
        assert_eq!(zpl_text, "^XA^FD1000000BB^FS^XZ");
    }

    #[test]
    fn seeded_template_renders_pdf() {
        let state = PlatformState::new();
        let req = RenderRequest {
            template_id: "z5z_01_gm_300_master".to_string(),
            output: Some("pdf".to_string()),
            response_mode: Some("url".to_string()),
            size: None,
            field_schema: json!([]),
            data: json!({}),
            manual_values: json!({}),
        };
        let artifact = render_from_request(&state, "req_test", &req, "pdf").unwrap();
        assert_eq!(artifact.content_type, "application/pdf");
        assert!(artifact.bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn invalid_output_is_rejected_before_rendering() {
        let state = PlatformState::new();
        let req = RenderRequest {
            template_id: "z5z_01_gm_300_master".to_string(),
            output: Some("docx".to_string()),
            response_mode: Some("url".to_string()),
            size: None,
            field_schema: json!([]),
            data: json!({}),
            manual_values: json!({}),
        };
        let err = render_from_request(&state, "req_test", &req, "docx").unwrap_err();
        assert_eq!(err.1, "VALIDATION_INVALID_OUTPUT");
    }

    #[test]
    fn print_request_accepts_client_schema_and_connection_mode() {
        let req: PrintRequest = serde_json::from_value(json!({
            "template_id": "z5z_01_gm_300_master",
            "delivery_mode": "device_print",
            "printer_id": "warehouse_a_01",
            "connection_mode": "direct_ip",
            "field_schema": [
                {
                    "name": "barcode_bc_11",
                    "kind": "barcode",
                    "barcode_type": "Barcode - Code 128",
                    "barcode_command": "^BC",
                    "sample_value": "12345678"
                }
            ],
            "data": { "barcode_bc_11": "987654321" },
            "manual_values": {}
        }))
        .expect("print request should deserialize");

        assert_eq!(req.connection_mode.as_deref(), Some("direct_ip"));
        assert_eq!(req.field_schema[0]["kind"], "barcode");
        assert_eq!(req.field_schema[0]["barcode_command"], "^BC");
        assert_eq!(req.data["barcode_bc_11"], "987654321");
    }

    #[test]
    fn print_task_log_keeps_label_data_snapshot() {
        let task = PrintTask {
            id: "pt_1".to_string(),
            request_id: "req_1".to_string(),
            template_id: "tpl_1".to_string(),
            printer_id: Some("warehouse_a_01".to_string()),
            delivery_mode: "device_print".to_string(),
            connection_mode: "print_server".to_string(),
            copies: 2,
            label_data: json!({ "hu": "578896087", "part": "06512515AA" }),
            field_schema: json!([{ "name": "hu", "sample_value": "578896087" }]),
            status: "queued".to_string(),
            retry_count: 0,
        };
        let value = serde_json::to_value(task).expect("print task serializes");

        assert_eq!(value["copies"], 2);
        assert_eq!(value["label_data"]["hu"], "578896087");
        assert_eq!(value["field_schema"][0]["name"], "hu");
    }

    #[test]
    fn print_connection_mode_is_whitelisted() {
        for mode in ["print_server", "direct_ip", "qz_tray", "pdf_only"] {
            assert!(
                is_allowed_connection_mode(mode),
                "{mode} should be accepted"
            );
        }
        assert!(!is_allowed_connection_mode("browser_socket"));
        assert!(!is_allowed_connection_mode(""));
    }

    #[test]
    fn rendered_output_can_be_stored_and_retrieved_by_request_id() {
        let state = PlatformState::new();
        let url = store_output(
            &state,
            "req_abc",
            OutputArtifact {
                content_type: "text/plain; charset=utf-8",
                bytes: b"^XA^XZ".to_vec(),
            },
        );
        assert_eq!(url, "/api/v1/requests/req_abc/output");
        let store = state.inner.store.lock().expect("store lock");
        let artifact = store.outputs.get("req_abc").expect("stored output");
        assert_eq!(artifact.bytes, b"^XA^XZ");
    }

    #[test]
    fn print_queue_health_surfaces_backlog_and_recovery_alerts() {
        let mut tasks = Vec::new();
        for idx in 0..25 {
            tasks.push(PrintTask {
                id: format!("pt_{}", idx),
                request_id: format!("req_{}", idx),
                template_id: "z5z_01_gm_300_master".to_string(),
                printer_id: Some("warehouse_a_01".to_string()),
                delivery_mode: "device_print".to_string(),
                connection_mode: "print_server".to_string(),
                copies: 1,
                label_data: json!({ "hu": format!("HU{}", idx) }),
                field_schema: json!([]),
                status: "queued".to_string(),
                retry_count: if idx == 0 { 1 } else { 0 },
            });
        }
        tasks.push(PrintTask {
            id: "pt_blocked".to_string(),
            request_id: "req_blocked".to_string(),
            template_id: "z5z_01_gm_300_master".to_string(),
            printer_id: Some("warehouse_a_01".to_string()),
            delivery_mode: "device_print".to_string(),
            connection_mode: "print_server".to_string(),
            copies: 1,
            label_data: json!({ "hu": "blocked" }),
            field_schema: json!([]),
            status: "device_offline".to_string(),
            retry_count: 3,
        });

        let health = print_queue_health(&tasks);
        assert_eq!(health.status, "critical");
        assert_eq!(health.queue_depth, 25);
        assert_eq!(health.needs_attention_count, 1);
        assert_eq!(health.retry_pending_count, 1);
        assert!(health
            .alerts
            .iter()
            .any(|alert| alert.code == "PRINT_TASKS_NEED_ATTENTION"));
        assert!(health
            .alerts
            .iter()
            .any(|alert| alert.code == "PRINT_QUEUE_BACKLOG_WARNING"));
        assert!(health
            .alerts
            .iter()
            .any(|alert| alert.code == "PRINT_RETRY_PENDING"));
    }
}
