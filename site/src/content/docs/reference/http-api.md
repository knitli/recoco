---
title: HTTP API
---
# ReCoco HTTP API Documentation

This document describes the HTTP API provided by the ReCoco library when the `server` feature is enabled. This API is primarily used for inspecting, monitoring, and querying running data flows.

## 1. Overview

- **Purpose**: Monitor flow status, inspect schemas, debugging keys/data, and execute queries against configured Query Handlers.
- **Base URL**: `http://{address}/cocoindex/api`
  - The `{address}` is configured via `ServerSettings` (e.g., `127.0.0.1:3000`).
- **Version**: Varies with library version (e.g., `0.2.x`).
- **Content-Type**: `application/json`

## 2. Authentication

- **Authentication Method**: None.
- **Security Note**: This API is intended for internal use within a private network or a secured environment (e.g., behind a reverse proxy or VPN). It does not implement built-in authentication mechanisms.

## 3. Endpoints

### 3.1. General

#### Health Check
**Method**: `GET`
**Path**: `/healthz`
**Description**: Returns the server status and library version.
**Response**:
```json
{
  "status": "ok",
  "version": "0.2.0"
}
```

#### Service Check
**Method**: `GET`
**Path**: `/cocoindex`
**Description**: Simple text check to verify the service is running.
**Response**: `CocoIndex is running!` (Text)

---

### 3.2. Flows

#### List Flows
**Method**: `GET`
**Path**: `/cocoindex/api/flows`
**Description**: Lists the names of all active flow instances in the current library context.
**Response**:
```json
[
  "my_flow_1",
  "knowledge_base_flow"
]
```

#### Get Flow Details
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}`
**Description**: Retrieves detailed configuration, schema, and registered query handlers for a specific flow.
**Parameters**:
- `flowInstName` (Path): The name of the flow instance.
**Response**:
```json
{
  "flow_spec": { ... },
  "data_schema": { ... },
  "query_handlers_spec": {
    "search": {
      "result_fields": { "embedding": [], "score": null }
    }
  },
  "fingerprint": "..."
}
```

#### Get Flow Schema
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}/schema`
**Description**: Returns the data schema for the flow.
**Response**:
```json
{
  "fields": [
    { "name": "id", "value_type": { "type": "Str" } },
    { "name": "content", "value_type": { "type": "Str" } }
  ]
}
```

#### Trigger Flow Update
**Method**: `POST`
**Path**: `/cocoindex/api/flows/{flowInstName}/update`
**Description**: Manually triggers an update cycle for the flow. This forces the flow to check for new data and process pending changes.
**Response**:
```json
{
  "added": 10,
  "updated": 5,
  "deleted": 0
}
```

---

### 3.3. Data Inspection

#### Get Source Keys
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}/keys`
**Description**: Lists all primary keys available for a specific source field in the flow.
**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| field | string | Yes | The name of the source field (operation) to list keys from. |

**Example Request**:
`GET /cocoindex/api/flows/my_flow/keys?field=my_source_file`

**Response**:
```json
{
  "key_schema": [ { "type": "Str" } ],
  "keys": [
    [ ["file1.txt"], null ],
    [ ["file2.txt"], null ]
  ]
}
```

#### Evaluate Source Data
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}/data`
**Description**: Evaluates and returns the data scope (variables) for a specific row in a source. Useful for debugging how a row is processed.
**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| field | string | Yes | The name of the source field. |
| key | string (multi) | Yes | The primary key components. Use multiple times for composite keys. |
| key_aux | string | No | JSON string containing auxiliary key info (optional). |

**Example Request**:
`GET /cocoindex/api/flows/my_flow/data?field=my_source&key=doc_1`

**Response**:
```json
{
  "schema": { ... },
  "data": {
    "field_name": "value",
    "computed_field": "computed_value"
  }
}
```

#### Get Row Indexing Status
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}/rowStatus`
**Description**: Checks the indexing status of a specific source row (e.g., is it fully processed, pending, or failed?).
**Query Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| field | string | Yes | The name of the source field. |
| key | string (multi) | Yes | The primary key components. Use multiple times for composite keys. |
| key_aux | string | No | JSON string containing auxiliary key info (optional). |

**Response**:
```json
{
  "status": "Synced",
  "last_updated": "2026-01-27T10:00:00Z"
}
```

---

### 3.4. Querying

#### Execute Query
**Method**: `GET`
**Path**: `/cocoindex/api/flows/{flowInstName}/queryHandlers/{queryHandlerName}`
**Description**: Executes a search/query against a specific Query Handler defined in the flow (e.g., vector search).
**Parameters**:
- `flowInstName` (Path): Flow name.
- `queryHandlerName` (Path): Name of the query handler (e.g., "search").

**Query Parameters**:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| query | string | Yes | The query string (e.g., search keywords). |

**Example Request**:
`GET /cocoindex/api/flows/kb_flow/queryHandlers/vector_search?query=rust+async`

**Response**:
```json
{
  "results": [
    [ ["url", "https://rust-lang.org"], ["title", "Rust Homepage"] ]
  ],
  "query_info": {
    "embedding": [0.1, 0.2, ...],
    "similarity_metric": "Cosine"
  }
}
```

## 4. Error Handling

The API uses standard HTTP status codes:

- **200 OK**: Request succeeded.
- **400 Bad Request**: Invalid parameters (e.g., missing field, unknown flow name).
- **500 Internal Server Error**: Server-side processing error.

**Error Response Format**:
```json
{
  "error": "Description of what went wrong"
}
```
*(Note: The exact JSON structure for errors depends on the `ApiError` serialization, typically a simple message or object).*