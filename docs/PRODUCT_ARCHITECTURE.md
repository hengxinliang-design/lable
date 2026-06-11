# Label Platform Product Architecture

## Purpose

This project will evolve from a ZPL/EPL rendering engine into a label platform for template design, data preparation, API integration, print routing, and operational monitoring.

The existing `labelize` crate remains the rendering driver. Higher-level modules should call it for parsing, preview rendering, PNG/PDF generation, and format validation instead of reimplementing label rendering logic.

## Product Modules

### 1. Label Design

Label Design manages templates and preview workflows.

Responsibilities:
- Store ZPL/EPL templates and metadata.
- Preview labels as PNG/PDF through `labelize`.
- Bind template variables to sample data.
- Keep template versions and change history.
- Validate whether a template can be parsed and rendered.
- Provide a future path toward visual or semi-visual editing.

Primary users:
- Operations users who maintain shipping, product, and warehouse labels.
- Developers who need to test template changes quickly.

Initial scope:
- Upload or paste a ZPL template.
- Save template name, label size, DPI/DPMM, and description.
- Render preview with JSON sample data.
- Keep a small set of test samples per template.

### 2. Data Source Processing

Data Source Processing converts business data into template-ready payloads.

Responsibilities:
- Import CSV, Excel, JSON, or API response data.
- Map business fields to template variables.
- Validate required fields and data types.
- Apply formatting rules, default values, and transformations.
- Generate batch label payloads.

Primary users:
- Operations users who prepare batch print jobs.
- Integration developers mapping external systems to labels.

Initial scope:
- Accept JSON and CSV as input.
- Define field mapping rules.
- Validate required fields before rendering.
- Produce one render payload per data row.

### 3. API Encapsulation And Testing

API Encapsulation exposes label rendering and printing as stable service APIs.

Responsibilities:
- Provide REST APIs for rendering, template lookup, and print requests.
- Offer an API test console for internal users.
- Normalize request/response formats and error codes.
- Support authentication and request tracing.
- Return PNG, PDF, raw ZPL, or print task IDs.

Primary users:
- Business systems that need label rendering or printing.
- Developers validating integration behavior.

Initial scope:
- `POST /api/v1/labels/render`
- `POST /api/v1/labels/print`
- `GET /api/v1/templates`
- `GET /api/v1/requests/{request_id}`
- A simple API test page that can submit JSON and preview output.

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
- Create and store a ZPL template.
- Submit JSON sample data.
- Render preview using `labelize`.
- Return PNG or PDF through API.
- Record request logs.
- Keep generated output linked to the request ID.

Out of scope for MVP:
- Full visual designer.
- Complex Excel mapping.
- Advanced print queue management.
- Real-time dashboards.
- Role-based permissions beyond basic API authentication.

## Candidate API Design

### Render Label

`POST /api/v1/labels/render`

Request:

```json
{
  "template_id": "shipping_label_v1",
  "output": "png",
  "size": {
    "width_mm": 102,
    "height_mm": 152,
    "dpmm": 8
  },
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
  "request_id": "req_01H00000000000000000000000",
  "status": "success",
  "output_type": "png",
  "output_url": "/api/v1/requests/req_01H00000000000000000000000/output"
}
```

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

## Core Data Model Draft

### Template

Fields:
- `id`
- `name`
- `format`: `zpl` or `epl`
- `content`
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
- `mapping_rules`
- `validation_rules`
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
- Add render API using template ID and JSON data.
- Add request logging.
- Add sample template management.
- Add smoke tests for render API.

### Phase 2: Template And Data Management

- Add template versioning.
- Add JSON/CSV data source import.
- Add field mapping and validation.
- Add batch render jobs.

### Phase 3: Print Gateway

- Add printer configuration.
- Add raw ZPL print dispatch.
- Add print task queue and retry.
- Add reprint workflow.

### Phase 4: Monitoring Console

- Add searchable request logs.
- Add print task detail pages.
- Add metrics dashboard.
- Add alert hooks for repeated failures.

### Phase 5: Advanced Designer

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
