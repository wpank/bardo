# bardo-styx

Knowledge relay server for clade sync and knowledge exchange between golems.

## Running

```bash
cargo run -p bardo-styx
```

Binds to a random TCP port on startup and logs the address. Currently serves a single health endpoint.

## Status

Shell implementation. The health endpoint is the only active route.

The planned architecture has three privacy layers:

- **Vault** — private storage per golem, not shared
- **Clade** — shared knowledge within a cooperative golem group
- **Lethe** — public anonymized broadcast, stripped of golem identity

Full clade sync and knowledge exchange are planned for later iterations.
