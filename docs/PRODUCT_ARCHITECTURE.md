# Label Platform Product Architecture

## Purpose

This project will evolve from a ZPL/EPL rendering engine into a label platform for template design, data preparation, API integration, print routing, and operational monitoring.

The existing `labelize` crate remains the rendering driver. Higher-level modules should call it for parsing, preview rendering, PNG/PDF generation, and format validation instead of reimplementing label rendering logic.

## Product Modules

### 1. Label Design

Label Design manages templates and preview workflows.

Responsibilities:
- Store ZPL/EPL templates and metadata.
- Create labels from imported ZPL first, then support brand-new visual drawing later.
- Preview labels as PNG/PDF through `labelize`.
- Bind template variables to sample data.
- Keep template versions and change history.
- Validate whether a template can be parsed and rendered.
- Provide a future path toward visual or semi-visual editing.

Primary users:
- Operations users who maintain shipping, product, and warehouse labels.
- Developers who need to test template changes quickly.

Initial scope:
- Upload or paste an existing ZPL template.
- Normalize the imported ZPL as the first editable template version.
- Support light secondary editing on the imported ZPL.
- Save template name, label size, DPI/DPMM, and description.
- Render preview with JSON sample data.
- Keep a small set of test samples per template.

Creation modes:
- Existing ZPL import: primary mode for the minimum version.
- New visual drawing: future mode after the imported-template workflow is stable.

Minimum editing model:
- Keep the original ZPL content intact as the source snapshot.
- Create a working copy for secondary edits.
- Use `labelize` to parse and render the working copy after every save or preview.
- Record each saved edit as a new template version.
- Treat variable extraction and field binding as an enhancement on top of imported ZPL, not a blocker for initial preview.

### 2. Data Source Processing

Data Source Processing converts parsed ZPL content and business data into template-ready payloads.

For the minimum version, this module starts from the imported ZPL template itself. The system should use `labelize` to read and parse the ZPL structure, extract editable data-bearing elements, and generate a formatted field confirmation form. A user then confirms which values are manually keyed and which values should be supplied by external form/API fields.

Responsibilities:
- Parse imported ZPL through `labelize` and extract candidate data fields.
- Build a formatted field confirmation form from parsed ZPL content.
- Let users classify each field as manual input, API/form input, fixed text, or ignored.
- Map confirmed API/form fields to template variables.
- Save reusable data source configurations for repeated label creation.
- Import CSV, Excel, JSON, or API response data after the field model is confirmed.
- Validate required fields and data types.
- Apply formatting rules, default values, and transformations.
- Generate batch label payloads and render/print jobs.

Primary users:
- Operations users who prepare batch print jobs.
- Integration developers mapping external systems to labels.

Initial scope:
- Extract candidate fields from imported ZPL text and barcode elements.
- Display normalized field rows for human confirmation.
- Mark fields as `manual`, `api_field`, `fixed`, or `ignored`.
- Define field mapping rules after confirmation.
- Accept JSON input for API-driven fields.
- Save field mappings as a reusable data source configuration.
- Generate labels in bulk from repeated data rows.
- Validate required fields before rendering.
- Produce one render payload from manually keyed values plus API-provided values.

Field confirmation workflow:
- Parse the imported ZPL working copy with `labelize`.
- Extract candidate values from text fields, barcodes, and 2D codes.
- Normalize each candidate into a field row with label, current value, element type, position, and suggested source.
- Let the user confirm field source and field name.
- For manual fields, store the confirmed value in the template configuration or require an operator to key it before rendering.
- For API/form fields, expose the confirmed field name in the render/print API request schema.
- Re-render the label after applying confirmed values to verify the output.

Field source types:
- `manual`: value is entered by an operator in the management UI.
- `api_field`: value is supplied by the caller through API payload.
- `fixed`: value stays unchanged as part of the template.
- `ignored`: parsed candidate is not part of the business field model.

Reusable data source configuration:
- A data source configuration belongs to one template or one template family.
- It stores field mapping, validation rules, default values, transformation rules, and batch import settings.
- Users should be able to select an existing configuration when creating labels from the same customer, supplier, warehouse, or business process.
- Reusing a configuration should skip repeated field confirmation unless the template version changed.
- If a template version changes, the system should compare previous fields with newly parsed fields and ask the user to resolve only changed or missing mappings.

Data source configuration management UI:
- Provide a searchable list page for saved configurations.
- Search and filter by configuration name, template, customer, supplier, warehouse, business process, status, creator, and updated time.
- Show key columns: name, template, customer, supplier, warehouse, business process, input type, status, updated time, and last used time.
- Provide create, edit, view detail, copy, enable, disable, archive/delete, and test actions.
- The detail page should show field mappings, validation rules, default values, transformation rules, batch settings, and recent usage.
- The edit page should allow users to adjust mappings and immediately test with sample rows.
- Copying a configuration should let users create a new configuration for a similar customer, supplier, warehouse, or process without repeating all setup.
- Disabling a configuration should prevent new batch jobs from using it while preserving history.

Batch label generation:
- Accept multiple data rows from JSON, CSV, Excel, or an API result.
- Validate every row against the confirmed template field model.
- Produce one label payload per valid row.
- Support partial failure: valid rows can render while invalid rows are returned with row-level errors.
- Store a batch job record with total count, success count, failure count, output type, and related request IDs.
- Allow users to download generated outputs individually or as a bundled file.
- Allow batch output to flow into the print queue when a printer is selected.

### 3. API Encapsulation And Testing

API Encapsulation exposes label rendering and printing as stable service APIs.

Responsibilities:
- Provide REST APIs for rendering, template lookup, and print requests.
- Offer an API test console for internal users.
- Generate request examples from confirmed template fields and data source configurations.
- Normalize request/response formats and error codes.
- Support authentication and request tracing.
- Return PNG, PDF, raw ZPL, or print task IDs.
- Validate request parameters before rendering or printing.

Primary users:
- Business systems that need label rendering or printing.
- Developers validating integration behavior.

Initial scope:
- `POST /api/v1/labels/render`
- `POST /api/v1/labels/print`
- `POST /api/v1/labels/batch-render`
- `GET /api/v1/templates`
- `GET /api/v1/requests/{request_id}`
- A simple API test page that can submit JSON and preview output.

API test console:
- Let users select a template and data source configuration.
- Auto-generate a request body from confirmed `api_field` definitions.
- Show editable request JSON with defaults, required fields, and field descriptions.
- Let users choose output type: `png`, `pdf`, `zpl`, or `print_task`.
- Run test requests against the local API service.
- Show response headers, status code, response body, output preview, and request ID.
- Show validation errors beside the matching request fields.
- Save successful test cases as reusable integration examples.

Request example generation:
- Generate cURL, JavaScript `fetch`, Python `requests`, and raw JSON examples.
- Include authentication headers and request ID header examples.
- Use confirmed template fields and data source configuration defaults.
- Highlight required fields and optional fields separately.
- Allow users to copy examples from the API test console.

Authentication:
- Support API keys for business system calls in the minimum version.
- Require `Authorization: Bearer <api_key>` for protected endpoints.
- Support an optional `X-Request-Id` header supplied by callers; generate one when missing.
- Store API key owner, status, allowed scopes, last used time, and expiration time.
- Define scopes such as `labels:render`, `labels:print`, `templates:read`, and `logs:read`.

Parameter validation:
- Validate required fields, field types, max length, allowed values, numeric ranges, and output type.
- Validate template status and data source configuration status before rendering.
- Validate printer availability before creating print tasks.
- Return row-level errors for batch requests.
- Never start rendering or printing when top-level request validation fails.

Output behavior:
- `png`: return an output URL and optionally inline binary download endpoint.
- `pdf`: return an output URL and optionally inline binary download endpoint.
- `zpl`: return the final merged ZPL after field replacement.
- `print_task`: enqueue a print task and return the task ID.

Error code standard:
- Use stable application error codes in addition to HTTP status codes.
- Return one consistent error response shape for validation, authentication, rendering, data mapping, and printing failures.
- Include `request_id` in every error response for log lookup.
- Keep error messages user-readable while preserving technical detail in server logs.

### 4. Print Configuration Gateway

Print Configuration Gateway manages printers, print routing, and print jobs.

Responsibilities:
- Store printer network configuration.
- Manage printer type, DPI, paper size, and supported formats.
- Route print jobs by warehouse, station, business type, or template.
- Track print queue status.
- Support retry, cancel, pause, resume, and reprint.

Primary users:
- IT or warehouse administrators maintaining printers.
- Operations users monitoring print execution.

Initial scope:
- Register network printers with IP, port, name, site, and DPI.
- Send raw ZPL to a configured printer.
- Record print task status and retry count.
- Support manual reprint from task history.

### 5. Interface Logs And Monitoring

Interface Logs And Monitoring tracks API calls, render results, and print execution.

Responsibilities:
- Record API requests and responses.
- Record render duration, template version, and output format.
- Record print task lifecycle events.
- Provide filtering by time, template, printer, status, and request ID.
- Surface error details for troubleshooting.
- Provide dashboard metrics for success rate, failure rate, and latency.

Primary users:
- Support users troubleshooting failed labels.
- Developers debugging integration issues.
- Administrators watching service health.

Initial scope:
- Persist request logs for render and print APIs.
- Show request status, duration, error message, and related task ID.
- Provide basic search and detail pages.

## System Data Flow

```text
Business System / User
        |
        v
API Encapsulation And Testing
        |
        +--> Interface Logs And Monitoring
        |
        v
Data Source Processing
        |
        v
Label Design / Template Store
        |
        v
labelize Rendering Driver
        |
        +--> PNG/PDF Preview
        |
        v
Print Configuration Gateway
        |
        v
Network Printer
```

## Suggested MVP

The first milestone should prove the complete render path before expanding print operations.

MVP capabilities:
- Import and store an existing ZPL template.
- Preserve the original uploaded ZPL as a source snapshot.
- Create an editable working copy for secondary processing.
- Submit JSON sample data.
- Render preview using `labelize`.
- Return PNG or PDF through API.
- Record request logs.
- Keep generated output linked to the request ID.

Out of scope for MVP:
- Full visual designer.
- Brand-new label drawing from a blank canvas.
- Complex Excel mapping.
- Advanced print queue management.
- Real-time dashboards.
- Role-based permissions beyond basic API authentication.

Initial Label Design workflow:
- Upload ZPL file.
- Detect format and basic label settings.
- Save template metadata and original ZPL.
- Render baseline preview.
- Apply secondary edits to the working ZPL copy.
- Save as a new template version.
- Attach sample data and expected preview output for regression checks.

## Candidate API Design

### Render Label

`POST /api/v1/labels/render`

Request:

```json
{
  "template_id": "shipping_label_v1",
  "output": "png",
  "response_mode": "url",
  "size": {
    "width_mm": 102,
    "height_mm": 152,
    "dpmm": 8
  },
  "data": {
    "order_no": "SO-10001",
    "sku": "ABC-001",
    "barcode": "1234567890"
  },
  "manual_values": {
    "operator_note": "checked"
  }
}
```

Response:

```json
{
  "request_id": "req_01H00000000000000000000000",
  "status": "success",
  "output_type": "png",
  "output_url": "/api/v1/requests/req_01H00000000000000000000000/output"
}
```

Supported `output` values:
- `png`
- `pdf`
- `zpl`

Supported `response_mode` values:
- `url`: return an output URL.
- `inline`: return the output directly when practical.

### Print Label

`POST /api/v1/labels/print`

Request:

```json
{
  "template_id": "shipping_label_v1",
  "printer_id": "printer_warehouse_a_01",
  "copies": 1,
  "data": {
    "order_no": "SO-10001",
    "sku": "ABC-001",
    "barcode": "1234567890"
  }
}
```

Response:

```json
{
  "request_id": "req_01H00000000000000000000001",
  "print_task_id": "pt_01H00000000000000000000001",
  "status": "queued"
}
```

### API Example Generation

`POST /api/v1/api-examples`

Request:

```json
{
  "template_id": "shipping_label_v1",
  "data_source_config_id": "dsc_warehouse_a_shipping",
  "operation": "render",
  "language": "curl",
  "output": "png"
}
```

Response:

```json
{
  "template_id": "shipping_label_v1",
  "operation": "render",
  "language": "curl",
  "example": "curl -X POST https://label.example.com/api/v1/labels/render -H 'Authorization: Bearer <api_key>' -H 'Content-Type: application/json' -d '{...}'"
}
```

### API Test Run

`POST /api/v1/api-tests/run`

Request:

```json
{
  "operation": "render",
  "request_body": {
    "template_id": "shipping_label_v1",
    "output": "png",
    "data": {
      "order_no": "SO-10001"
    }
  }
}
```

Response:

```json
{
  "request_id": "req_01H00000000000000000000002",
  "status_code": 200,
  "status": "success",
  "output_preview_url": "/api/v1/requests/req_01H00000000000000000000002/output"
}
```

### Error Response Format

All API errors should use one response shape:

```json
{
  "request_id": "req_01H00000000000000000000003",
  "status": "error",
  "error": {
    "code": "VALIDATION_REQUIRED_FIELD",
    "message": "Required field is missing: order_no",
    "details": [
      {
        "field": "data.order_no",
        "reason": "required"
      }
    ]
  }
}
```

Initial error codes:
- `AUTH_MISSING_TOKEN`
- `AUTH_INVALID_TOKEN`
- `AUTH_SCOPE_DENIED`
- `VALIDATION_REQUIRED_FIELD`
- `VALIDATION_INVALID_TYPE`
- `VALIDATION_INVALID_OUTPUT`
- `TEMPLATE_NOT_FOUND`
- `TEMPLATE_INACTIVE`
- `DATA_SOURCE_CONFIG_NOT_FOUND`
- `DATA_MAPPING_FAILED`
- `RENDER_FAILED`
- `PRINTER_NOT_FOUND`
- `PRINTER_UNAVAILABLE`
- `PRINT_TASK_FAILED`
- `BATCH_ROW_VALIDATION_FAILED`

### Batch Render Labels

`POST /api/v1/labels/batch-render`

Request:

```json
{
  "template_id": "shipping_label_v1",
  "data_source_config_id": "dsc_warehouse_a_shipping",
  "output": "png",
  "rows": [
    {
      "order_no": "SO-10001",
      "sku": "ABC-001",
      "barcode": "1234567890"
    },
    {
      "order_no": "SO-10002",
      "sku": "ABC-002",
      "barcode": "1234567891"
    }
  ],
  "manual_values": {
    "operator_note": "checked"
  }
}
```

Response:

```json
{
  "batch_job_id": "batch_01H00000000000000000000001",
  "status": "processing",
  "total_rows": 2,
  "accepted_rows": 2,
  "rejected_rows": 0
}
```

### Search Data Source Configurations

`GET /api/v1/data-source-configs`

Query parameters:
- `q`
- `template_id`
- `customer`
- `supplier`
- `warehouse`
- `business_process`
- `status`
- `updated_from`
- `updated_to`

Response:

```json
{
  "items": [
    {
      "id": "dsc_warehouse_a_shipping",
      "name": "Warehouse A shipping labels",
      "template_id": "shipping_label_v1",
      "customer": "SAIC USA",
      "supplier": "Relyans Max Inc.",
      "warehouse": "Warehouse A",
      "business_process": "shipping",
      "type": "csv",
      "status": "active",
      "updated_at": "2026-06-11T15:00:00Z",
      "last_used_at": "2026-06-11T15:30:00Z"
    }
  ],
  "total": 1
}
```

### Manage Data Source Configuration

Endpoints:
- `POST /api/v1/data-source-configs`
- `GET /api/v1/data-source-configs/{id}`
- `PUT /api/v1/data-source-configs/{id}`
- `POST /api/v1/data-source-configs/{id}/copy`
- `POST /api/v1/data-source-configs/{id}/enable`
- `POST /api/v1/data-source-configs/{id}/disable`
- `DELETE /api/v1/data-source-configs/{id}`
- `POST /api/v1/data-source-configs/{id}/test`

## Core Data Model Draft

### Template

Fields:
- `id`
- `name`
- `format`: `zpl` or `epl`
- `content`
- `source_content`
- `creation_mode`: `imported_zpl` or `visual_drawing`
- `width_mm`
- `height_mm`
- `dpmm`
- `version`
- `status`
- `created_at`
- `updated_at`

### Template Sample

Fields:
- `id`
- `template_id`
- `name`
- `sample_data`
- `expected_output_type`
- `created_at`
- `updated_at`

### Data Source

Fields:
- `id`
- `name`
- `type`: `json`, `csv`, `excel`, or `api`
- `template_id`
- `template_version`
- `customer`
- `supplier`
- `warehouse`
- `business_process`
- `description`
- `mapping_rules`
- `validation_rules`
- `default_values`
- `transform_rules`
- `batch_settings`
- `status`
- `created_by`
- `updated_by`
- `last_used_at`
- `created_at`
- `updated_at`

### API Key

Fields:
- `id`
- `name`
- `owner`
- `key_hash`
- `scopes`
- `status`
- `expires_at`
- `last_used_at`
- `created_at`
- `updated_at`

### API Test Case

Fields:
- `id`
- `name`
- `template_id`
- `data_source_config_id`
- `operation`
- `request_body`
- `expected_output_type`
- `last_status`
- `last_request_id`
- `created_by`
- `created_at`
- `updated_at`

### Template Field

Fields:
- `id`
- `template_id`
- `template_version`
- `field_name`
- `display_name`
- `source_type`: `manual`, `api_field`, `fixed`, or `ignored`
- `element_type`: `text`, `barcode`, `datamatrix`, `qrcode`, or `unknown`
- `zpl_command`
- `original_value`
- `default_value`
- `required`
- `position`
- `format_rules`
- `created_at`
- `updated_at`

### Printer

Fields:
- `id`
- `name`
- `site`
- `ip`
- `port`
- `model`
- `dpmm`
- `status`
- `created_at`
- `updated_at`

### Render Request Log

Fields:
- `id`
- `template_id`
- `template_version`
- `output_type`
- `request_payload`
- `status`
- `duration_ms`
- `error_message`
- `created_at`

### Print Task

Fields:
- `id`
- `request_id`
- `template_id`
- `printer_id`
- `raw_payload`
- `copies`
- `status`
- `retry_count`
- `error_message`
- `created_at`
- `updated_at`

### Batch Label Job

Fields:
- `id`
- `template_id`
- `template_version`
- `data_source_config_id`
- `output_type`
- `status`
- `total_rows`
- `success_rows`
- `failed_rows`
- `request_ids`
- `error_summary`
- `created_at`
- `updated_at`

### Batch Label Row

Fields:
- `id`
- `batch_job_id`
- `row_index`
- `input_data`
- `merged_values`
- `status`
- `request_id`
- `output_path`
- `error_message`
- `created_at`
- `updated_at`

## Technical Direction

Recommended layering:
- `labelize-core`: existing parser, renderer, encoder logic.
- `label-platform-api`: HTTP API, request validation, authentication.
- `label-platform-service`: template, data mapping, render, print, and logging services.
- `label-platform-web`: management UI for design, testing, configuration, and monitoring.
- `label-platform-storage`: database schema and repository layer.

The current repository can start by extending the existing HTTP service gradually. If the platform grows large, split upper-layer services into separate crates or a workspace.

## Development Roadmap

### Phase 1: Render Service Foundation

- Define template storage model.
- Add imported ZPL template storage.
- Preserve original ZPL and editable working ZPL separately.
- Add render API using template ID and JSON data.
- Add request logging.
- Add sample template management.
- Add smoke tests for render API.

### Phase 2: Template And Data Management

- Add template versioning.
- Add ZPL field extraction from imported templates.
- Add field confirmation form.
- Add manual/API/fixed/ignored field source classification.
- Add JSON/CSV data source import.
- Add field mapping and validation.
- Add reusable data source configurations.
- Add searchable data source configuration management UI.
- Add create/edit/copy/enable/disable/test actions for data source configurations.
- Add batch render jobs with row-level validation.
- Add output download for batch label results.

### Phase 3: API Integration Workbench

- Add API test console.
- Add request example generation for cURL, JavaScript, Python, and raw JSON.
- Add API key authentication and scope checks.
- Add unified request validation.
- Add stable error response format and error codes.
- Add API test case save and rerun.

### Phase 4: Print Gateway

- Add printer configuration.
- Add raw ZPL print dispatch.
- Add print task queue and retry.
- Add reprint workflow.

### Phase 5: Monitoring Console

- Add searchable request logs.
- Add print task detail pages.
- Add metrics dashboard.
- Add alert hooks for repeated failures.

### Phase 6: Advanced Designer

- Add brand-new label drawing from a blank canvas.
- Add field-aware template editing.
- Add variable hints and validation.
- Add semi-visual layout assistance.
- Add visual diff against golden samples.

## Open Questions

- Should the management UI live inside this Rust service or as a separate frontend app?
- Which database should be used first: SQLite for local deployment, or PostgreSQL for multi-user deployment?
- Should print dispatch talk directly to printers from the API service, or through a dedicated worker process?
- What authentication model is required for internal users and external business systems?
- Which label formats beyond ZPL/EPL must be supported later?
- How should imported ZPL variables be marked: custom placeholders, ZPL field numbers, or external mapping rules?
- Which parsed ZPL elements should become candidate fields automatically: all `^FD` values, only text/barcodes, or only user-selected elements?
