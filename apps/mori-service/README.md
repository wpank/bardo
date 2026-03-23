# mori-service

HTTP API daemon wrapping the mori build system. Exposes plan orchestration as a network service.

## Running

```bash
cargo run -p mori-service

# with explicit config
MORI_SERVICE_PORT=8080 MORI_SERVICE_BIND=0.0.0.0 cargo run -p mori-service
```

## Configuration

| Flag | Env var | Default |
|------|---------|---------|
| `--port` | `MORI_SERVICE_PORT` | `8080` |
| `--bind` | `MORI_SERVICE_BIND` | `0.0.0.0` |

CLI flags take precedence over environment variables.

## Architecture

The binary (`mori-service`) is a thin entry point. The actual server logic lives in the `mori_service` library crate, split across three modules:

- `mori_service::api` — Axum route handlers
- `mori_service::state::ServiceState` — shared state passed into handlers via `axum::Extension`
- `mori_service::types::ServiceConfig` — config struct populated from CLI args and env vars

HTTP is served by Axum with tower-http providing CORS middleware.

## Status

Early implementation. API routes exist but coverage of the full mori plan orchestration surface is incomplete. Do not depend on the API shape being stable.
