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
- Let business users submit test data and generate a PDF label without sending anything to a physical printer.
- Let technical users switch the same request from PDF test output to configured printer dispatch.
- Run test requests against the local API service.
- Show response headers, status code, response body, output preview, and request ID.
- Show a PDF preview/download link when the test output is `pdf`.
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
- `pdf_test`: generate a PDF from the same label data for business validation without printer dispatch.

Delivery modes:
- `render_only`: generate PNG, PDF, or ZPL without creating a print task.
- `pdf_preview`: generate a PDF for business testing and approval.
- `device_print`: route the generated raw ZPL through the print configuration gateway.

Error code standard:
- Use stable application error codes in addition to HTTP status codes.
- Return one consistent error response shape for validation, authentication, rendering, data mapping, and printing failures.
- Include `request_id` in every error response for log lookup.
- Keep error messages user-readable while preserving technical detail in server logs.

Print Service component boundary:
- Print Service is the business-system-facing print entrypoint.
- It receives RESTful print requests from external business systems.
- It accepts JSON first and should support XML input through a conversion layer.
- It normalizes incoming JSON/XML into the confirmed template field model.
- It performs data conversion, field mapping, default value filling, and transformation rules.
- It validates required fields, field types, lengths, allowed values, and business rules before creating print work.
- It selects the correct template from request parameters, route context, or configured defaults.
- It selects a target printer or printer route candidate using warehouse, site, business type, customer, supplier, and template.
- It creates either a PDF preview response or a print task request.
- It passes validated print work to Print Configuration Gateway for routing, queueing, device dispatch, retry, and status management.
- It records API request logs, render logs, validation errors, and the resulting print task ID.

Print Service should not:
- Store low-level printer connection details directly.
- Own printer health checks or queue worker scheduling.
- Dispatch raw ZPL to devices by itself.
- Replace Print Configuration Gateway routing and task lifecycle management.

### 4. Print Configuration Gateway

Print Configuration Gateway manages printers, print routing, and print jobs.

Responsibilities:
- Store printer network and client-side printing configuration.
- Manage printer IP, port, DPI/DPMM, paper size, model, and supported formats.
- Bind templates to one or more printers.
- Route print jobs by warehouse, site, business type, template, and priority.
- Track print queue status and print task lifecycle.
- Support retry, pause, resume, cancel, and reprint.
- Manage print status from queued to completed or failed.

Primary users:
- IT or warehouse administrators maintaining printers.
- Operations users monitoring print execution.

Initial scope:
- Register network printers with IP, port, name, site, model, DPI/DPMM, and paper size.
- Register optional QZ Tray client printers for browser-connected workstations.
- Bind templates to allowed printers and default printers.
- Define route rules by warehouse, site, business type, and template.
- Allow the same print request to produce a PDF preview instead of dispatching to a printer.
- Send raw ZPL to a configured printer through direct network printing or QZ Tray.
- Queue print tasks and track status.
- Record print task status and retry count.
- Support retry, pause, resume, cancel, and manual reprint from task history.

Printer configuration:
- Required fields: printer name, connection type, site, warehouse, model, DPI/DPMM, paper width, paper height, status.
- Direct network printers require IP and port.
- QZ Tray printers require workstation/client identity and local printer name.
- Supported formats should include raw ZPL first, with PDF/PNG print support as future options.
- Printer health should be checked manually in the minimum version and automatically in later versions.

Template-printer binding:
- A template can define allowed printers and one default printer.
- Bindings can override paper size, DPI/DPMM, copies, and print mode for a specific template.
- The print API should reject printers that are not allowed for the selected template unless an administrator override is used.

Routing rules:
- Rules can match warehouse, site, business type, template, customer, supplier, and priority.
- The highest-priority active rule wins.
- If no rule matches, fall back to the template default printer.
- If no default printer exists, the print request should fail with a routing error.
- Route decisions should be recorded on the print task for troubleshooting.

Print queue:
- Print tasks should be created before dispatch.
- Initial statuses: `queued`, `dispatching`, `sent`, `completed`, `failed`, `paused`, `canceled`.
- Queue workers should select tasks by priority, creation time, and printer availability.
- Failed tasks should store error details and retry eligibility.
- Pausing a printer should stop dispatching new tasks to that printer without losing queued tasks.

Print task operations:
- Retry: create a new attempt on the same task when retry limits allow.
- Pause: pause a queued task or pause dispatch for a printer.
- Resume: make paused tasks eligible for dispatch again.
- Cancel: prevent a queued or paused task from printing.
- Reprint: create a new print task from a completed or failed historical task.

PDF preview mode:
- PDF preview mode uses the same template, data mapping, validation, and rendering path as real printing.
- It should not create a physical print dispatch attempt.
- It should create request and render logs so business test activity remains traceable.
- It should return a PDF URL that business users can download and review.
- An approved PDF preview can be used as the last test case before enabling device printing.

QZ Tray integration option:
- QZ Tray can be used as an optional local workstation print bridge for browser-based printing.
- The platform should treat QZ Tray as one print channel beside direct network socket printing.
- QZ Tray mode is useful when printers are only available from a user's workstation or local network.
- The API should still create and track print tasks on the server; the browser/client reports dispatch result back to the server.
- QZ Tray support should include client registration, local printer selection, signed request handling if required, and result callback logging.

### 5. Interface Logs And Monitoring

Interface Logs And Monitoring tracks API calls, render results, and print execution.

Responsibilities:
- Record API requests and responses.
- Record render duration, template version, output format, and render errors.
- Record print task lifecycle events, attempts, retries, and final status.
- Provide filtering by template, API endpoint, printer, status, time range, and request ID.
- Surface error details and retry history for troubleshooting.
- Provide dashboard metrics for success rate, failure rate, duration, latency, and call volume.

Primary users:
- Support users troubleshooting failed labels.
- Developers debugging integration issues.
- Administrators watching service health.

Initial scope:
- Persist request logs for render and print APIs.
- Persist render logs for every label render attempt.
- Persist print task logs and retry attempts.
- Show request status, duration, error message, related render ID, and related print task ID.
- Provide searchable log list pages and detail pages.
- Provide a dashboard for success rate, failure rate, average duration, p95 duration, and call volume.

API request logs:
- Capture method, path, endpoint name, request ID, caller, API key ID, status code, duration, request payload summary, response summary, and error code.
- Redact sensitive values before storing request and response details.
- Link each API log to render logs, batch jobs, and print tasks when applicable.

Render logs:
- Capture template ID, template version, output type, data source configuration, merged field values summary, duration, output path, status, and error details.
- Store render errors separately from API validation errors so rendering engine issues can be diagnosed.
- Record the `labelize` options used for rendering, including width, height, and DPMM.

Print task logs:
- Capture queue status transitions, selected printer, route rule, channel, retries, attempts, dispatch result, and final status.
- Link every retry to a print task attempt.
- Preserve the original print task and create clear history for reprint tasks.

Search and filtering:
- Common filters: time range, status, request ID, template, endpoint, printer, warehouse, site, business type, output type, and error code.
- API logs should filter by endpoint, caller, API key, status code, and duration range.
- Render logs should filter by template, output type, render status, and duration range.
- Print logs should filter by printer, route rule, task status, retry count, and channel.

Dashboard:
- Show total calls, render count, print count, success rate, failure rate, average duration, p95 duration, and call volume trend.
- Break down metrics by template, endpoint, printer, warehouse, and error code.
- Show recent failures and top recurring errors.
- Show queue depth and failed print task count.
- Allow time ranges such as last 15 minutes, 1 hour, 24 hours, 7 days, and custom range.

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
Print Service
        |
        +--> Data Source Processing
        |
        +--> Label Design / Template Store
        |
        +--> labelize Rendering Driver
        |
        +--> PNG/PDF Preview
        |
        v
Print Configuration Gateway
        |
        v
Network Printer / QZ Tray Client
```

Detailed print flow:

```text
Business System
        |
        v
Print Service REST API
        |
        +--> JSON/XML normalization
        +--> field mapping and validation
        +--> template selection
        +--> printer or route candidate selection
        +--> labelize render / PDF preview when requested
        |
        v
Print Configuration Gateway
        |
        +--> route rule resolution
        +--> print queue
        +--> retry / pause / cancel / reprint
        |
        v
Printer Device / QZ Tray
```

General render flow:

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
Network Printer / QZ Tray Client
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
- Generate a PDF label from POSTed test data for business validation.
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
  "output": "pdf",
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
  "output_type": "pdf",
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
  "delivery_mode": "device_print",
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

Supported `delivery_mode` values:
- `pdf_preview`: generate a PDF label for business testing and do not dispatch to a printer.
- `device_print`: dispatch the generated label through printer routing/configuration.

When `delivery_mode` is `pdf_preview`, `printer_id` is optional and the response returns `output_type: "pdf"` plus `output_url` instead of a `print_task_id`.

### Test Label PDF

`POST /api/v1/labels/test-pdf`

This endpoint is optimized for API testing pages and business validation. It validates the request, generates a label from POSTed data, converts it to PDF, and returns a downloadable PDF without printing.

Request:

```json
{
  "template_id": "shipping_label_v1",
  "data_source_config_id": "dsc_warehouse_a_shipping",
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
  "request_id": "req_01H00000000000000000000004",
  "status": "success",
  "output_type": "pdf",
  "output_url": "/api/v1/requests/req_01H00000000000000000000004/output"
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

### Manage Printers

Endpoints:
- `GET /api/v1/printers`
- `POST /api/v1/printers`
- `GET /api/v1/printers/{id}`
- `PUT /api/v1/printers/{id}`
- `POST /api/v1/printers/{id}/enable`
- `POST /api/v1/printers/{id}/disable`
- `POST /api/v1/printers/{id}/pause`
- `POST /api/v1/printers/{id}/resume`
- `POST /api/v1/printers/{id}/test-print`
- `DELETE /api/v1/printers/{id}`

Printer search filters:
- `q`
- `site`
- `warehouse`
- `model`
- `connection_type`
- `status`

### Manage Template Printer Bindings

Endpoints:
- `GET /api/v1/templates/{template_id}/printer-bindings`
- `POST /api/v1/templates/{template_id}/printer-bindings`
- `PUT /api/v1/templates/{template_id}/printer-bindings/{binding_id}`
- `DELETE /api/v1/templates/{template_id}/printer-bindings/{binding_id}`

### Manage Print Routing Rules

Endpoints:
- `GET /api/v1/print-route-rules`
- `POST /api/v1/print-route-rules`
- `GET /api/v1/print-route-rules/{id}`
- `PUT /api/v1/print-route-rules/{id}`
- `POST /api/v1/print-route-rules/{id}/enable`
- `POST /api/v1/print-route-rules/{id}/disable`
- `DELETE /api/v1/print-route-rules/{id}`
- `POST /api/v1/print-route-rules/test`

Route test request:

```json
{
  "template_id": "shipping_label_v1",
  "warehouse": "Warehouse A",
  "site": "Detroit",
  "business_type": "shipping",
  "customer": "SAIC USA",
  "supplier": "Relyans Max Inc."
}
```

### Manage Print Tasks

Endpoints:
- `GET /api/v1/print-tasks`
- `GET /api/v1/print-tasks/{id}`
- `POST /api/v1/print-tasks/{id}/retry`
- `POST /api/v1/print-tasks/{id}/pause`
- `POST /api/v1/print-tasks/{id}/resume`
- `POST /api/v1/print-tasks/{id}/cancel`
- `POST /api/v1/print-tasks/{id}/reprint`

### QZ Tray Client Callbacks

Endpoints:
- `POST /api/v1/qz/clients/register`
- `POST /api/v1/qz/clients/{client_id}/printers/sync`
- `POST /api/v1/qz/print-tasks/{task_id}/dispatch-result`

### Logs And Monitoring

Endpoints:
- `GET /api/v1/logs/api-requests`
- `GET /api/v1/logs/api-requests/{id}`
- `GET /api/v1/logs/renders`
- `GET /api/v1/logs/renders/{id}`
- `GET /api/v1/logs/print-tasks`
- `GET /api/v1/logs/print-tasks/{id}`
- `GET /api/v1/logs/errors`
- `GET /api/v1/dashboard/summary`
- `GET /api/v1/dashboard/timeseries`

Common log filters:
- `request_id`
- `template_id`
- `endpoint`
- `printer_id`
- `warehouse`
- `site`
- `business_type`
- `status`
- `error_code`
- `from`
- `to`

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
- `warehouse`
- `connection_type`: `network_socket` or `client_qz_tray`
- `ip`
- `port`
- `client_id`
- `local_printer_name`
- `model`
- `dpmm`
- `paper_width_mm`
- `paper_height_mm`
- `supported_formats`
- `status`
- `paused`
- `last_health_check_at`
- `last_health_status`
- `created_at`
- `updated_at`

### Template Printer Binding

Fields:
- `id`
- `template_id`
- `printer_id`
- `is_default`
- `allowed`
- `override_dpmm`
- `override_paper_width_mm`
- `override_paper_height_mm`
- `default_copies`
- `print_mode`
- `status`
- `created_at`
- `updated_at`

### Print Route Rule

Fields:
- `id`
- `name`
- `priority`
- `warehouse`
- `site`
- `business_type`
- `template_id`
- `customer`
- `supplier`
- `printer_id`
- `status`
- `created_at`
- `updated_at`

### API Request Log

Fields:
- `id`
- `request_id`
- `method`
- `path`
- `endpoint`
- `caller`
- `api_key_id`
- `status_code`
- `duration_ms`
- `request_summary`
- `response_summary`
- `error_code`
- `error_message`
- `related_render_id`
- `related_batch_job_id`
- `related_print_task_id`
- `created_at`

### Render Log

Fields:
- `id`
- `request_id`
- `template_id`
- `template_version`
- `output_type`
- `data_source_config_id`
- `render_options`
- `merged_values_summary`
- `output_path`
- `status`
- `duration_ms`
- `error_code`
- `error_message`
- `created_at`

### Error Log

Fields:
- `id`
- `request_id`
- `source`: `api`, `render`, `print`, `data_mapping`, or `system`
- `code`
- `message`
- `details`
- `template_id`
- `printer_id`
- `print_task_id`
- `render_log_id`
- `created_at`

### Print Task

Fields:
- `id`
- `request_id`
- `template_id`
- `printer_id`
- `route_rule_id`
- `channel`: `network_socket` or `client_qz_tray`
- `raw_payload`
- `copies`
- `priority`
- `status`
- `retry_count`
- `max_retries`
- `error_message`
- `queued_at`
- `dispatched_at`
- `completed_at`
- `canceled_at`
- `created_at`
- `updated_at`

### Print Task Attempt

Fields:
- `id`
- `print_task_id`
- `attempt_no`
- `status`
- `channel`
- `printer_id`
- `client_id`
- `started_at`
- `finished_at`
- `error_message`
- `created_at`
- `updated_at`

### QZ Tray Client

Fields:
- `id`
- `name`
- `site`
- `warehouse`
- `workstation_name`
- `user_name`
- `status`
- `last_seen_at`
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
- `label-print-service`: business-system-facing print entrypoint, JSON/XML normalization, field validation, template selection, and printer route candidate selection.
- `label-platform-service`: template, data mapping, render, print, and logging services.
- `label-print-gateway`: printer configuration, route resolution, print queue, device dispatch, QZ Tray integration, retry, and print status lifecycle.
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
- Add POST-to-PDF label testing for business validation.
- Add request example generation for cURL, JavaScript, Python, and raw JSON.
- Add API key authentication and scope checks.
- Add unified request validation.
- Add stable error response format and error codes.
- Add API test case save and rerun.

### Phase 4: Print Gateway

- Add printer configuration management for IP, port, DPI/DPMM, paper size, model, site, warehouse, and status.
- Add template-printer binding.
- Add routing rules by warehouse, site, business type, template, customer, and supplier.
- Add PDF preview delivery mode that uses print configuration but does not dispatch to a device.
- Add raw ZPL print dispatch through direct network printing.
- Add optional QZ Tray client print channel.
- Add print task queue with priority and printer availability checks.
- Add retry, pause, resume, cancel, and reprint workflows.
- Add print status management and task attempt history.

### Phase 5: Monitoring Console

- Add searchable API request logs.
- Add render logs with template, output type, duration, and error detail.
- Add print task logs with attempts, retries, and status history.
- Add error detail pages linked by request ID.
- Add filters by template, endpoint, printer, warehouse, site, time range, status, and error code.
- Add dashboard metrics for success rate, failure rate, average duration, p95 duration, and call volume.
- Add print queue depth and failed print task widgets.
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
