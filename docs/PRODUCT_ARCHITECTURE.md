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
- Import CSV, Excel, JSON, or API response data after the field model is confirmed.
- Validate required fields and data types.
- Apply formatting rules, default values, and transformations.
- Generate batch label payloads.

Primary users:
- Operations users who prepare batch print jobs.
- Integration developers mapping external systems to labels.

Initial scope:
- Extract candidate fields from imported ZPL text and barcode elements.
- Display normalized field rows for human confirmation.
- Mark fields as `manual`, `api_field`, `fixed`, or `ignored`.
- Define field mapping rules after confirmation.
- Accept JSON input for API-driven fields.
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
- `mapping_rules`
- `validation_rules`
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
