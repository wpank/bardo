# bardo-gateway

bardo-gateway is the external API gateway for the Bardo system. It is the single ingress point for clients that want to observe or interact with the golem fleet from outside the internal network.

## Features

- Expose a public HTTP/WebSocket API for external clients
- Route requests to the appropriate internal service (golem surfaces, styx relay, etc.)
- Handle authentication and rate limiting at the boundary
- Serve the WebSocket and SSE event streams from `golem-surfaces` to external consumers

## Architecture

bardo-gateway sits at the edge of the system. Internal golem services publish events and state through `golem-surfaces`. bardo-gateway picks those up and makes them available to external callers — dashboards, bots, monitoring tools — without giving external clients direct access to internal golem processes.
