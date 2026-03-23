# bardo-compute

Compute provisioning and fleet management service.

## Running

```bash
cargo run -p bardo-compute
```

Starts a TCP server and responds to health checks:

```json
{"status":"ok","service":"bardo-compute"}
```

## Status

Shell implementation. Health endpoint only. Fleet provisioning and compute management are planned for later iterations.
