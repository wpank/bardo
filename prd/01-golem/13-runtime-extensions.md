# Runtime Extensions [SPEC] -- Split Notice

> **Reader orientation:** This is a redirect page. The runtime extensions specification -- which defines how every subsystem in the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) is implemented as a native Rust extension with lifecycle hooks -- has been split into two parts. Extensions are the primary organizational unit: 28 extensions across 7 dependency layers, each implementing a subset of 20 lifecycle hooks. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

This document has been split into two parts to stay under the 2000-line threshold:

- **[13a-runtime-extensions.md](./13a-runtime-extensions.md)** -- Extension Architecture, Registry, GolemState, Interventions, Session Tree, Tool Authorization, Model Routing, Adaptive Clock (Sections 1-6 + source primitives)
- **[13b-runtime-extensions.md](./13b-runtime-extensions.md)** -- Type-State Machine, Event Fabric, CorticalState (32-signal atomic shared perception surface), Arena allocator, 10-phase Shutdown, Main Binary (Sections 7-15)

All existing cross-references to `13-runtime-extensions.md` remain valid via this redirect.
