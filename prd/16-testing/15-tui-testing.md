# TUI Testing Specification

> **Reader orientation:** This document specifies the testing strategy for `bardo-terminal`, the TUI (terminal user interface) through which operators observe and interact with their Golems (mortal autonomous agents). It belongs to Section 16 (Testing) and covers unit tests, integration tests, snapshot tests, performance budgets, and property tests for the 29-screen, 6-window interface. See `prd2/shared/glossary.md` for full term definitions.

## Overview

Testing strategy for the 29-screen, 6-window `bardo-terminal` TUI.

## Unit Tests

- **Widget rendering**: Each widget renders correctly given mock state. Use `ratatui::backend::TestBackend`.
- **Focus stack**: Verify Layer 0-5 transitions, Esc behavior (jump to Layer 3), Backspace (back one level).
- **Key dispatch**: Verify key events route to correct handler at each layer depth.
- **State derivation**: Verify `GolemSnapshot` → screen state mapping for each of the 29 screens.

## Integration Tests

- **WebSocket reconnection**: Simulate disconnect/reconnect. Verify state recovery from last snapshot.
- **Progressive disclosure**: Verify tabs activate when data becomes available.
- **Multi-Golem switching**: Verify sidebar state persists across Golem switches.
- **Offline mode**: Verify graceful degradation when WebSocket is unavailable.

## Snapshot Tests

- **Golden frame tests**: Render each screen at 80x24, 120x40, 200x60 and compare against golden frames.
- **Responsive breakpoints**: Verify layout changes at sub-80, 80-119, 120-159, 160-199, 200+ column widths.
- **Theme variants**: Verify rendering under each color theme.

## Performance Tests

- **Frame budget**: Assert < 16.6ms per frame (60fps) under worst-case state.
- **Post-processing budget**: Assert < 1.5ms for bloom/composite pass.
- **Startup time**: Assert < 500ms from launch to first painted frame.

## Property Tests

- **Focus stack invariants**: `back()` at Layer 0 is no-op. `depth()` never exceeds 6.
- **Key conflict detection**: No two handlers claim the same key at the same layer depth.
- **Screen count**: Exactly 29 screens registered in the screen catalog enum.
