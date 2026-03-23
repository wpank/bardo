# UI component system [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14
**Package:** `packages/ui/`

---

> **Reader orientation:** This document specifies `@bardo/ui`, the shared React component library for all Bardo web surfaces (Portal dashboard, debug UI, browser SPA). It belongs to the **interfaces layer** and implements the ROSEDUST design language (rose on violet-black surfaces with restrained bone accents). This is NOT the Golem's primary interface; that is the TUI (`bardo-terminal`), a separate Rust/ratatui application. Key concepts: Golem (a mortal autonomous DeFi agent), the Portal (web dashboard for agent setup and analytics, see `00-portal.md`), and the `bardo-sprites` WASM module for creature rendering. For unfamiliar terms, see `prd2/shared/glossary.md`.

## Overview

`@bardo/ui` is the shared React component library for all Bardo web surfaces. It implements the **ROSEDUST** design language -- rose on violet-black surfaces with restrained bone accents that communicate safety, precision, and sophistication.

This library serves the Portal (web dashboard), debug UI, and browser SPA. It is not the Golem's primary UI. The Golem's primary interface is the TUI (`bardo-terminal`), a separate Rust/ratatui application. The React components here are for web surfaces where TypeScript and React are the right tools: onboarding wizards, analytics dashboards, reputation profiles.

The package is internal (no npm publish), consumed via direct TypeScript source imports.

---

## Scope boundaries

| Surface              | UI System            | Where Defined                      |
| -------------------- | -------------------- | ---------------------------------- |
| **Portal**           | `@bardo/ui` (React)  | This document                      |
| **Debug UI**         | `@bardo/ui` (React)  | This document                      |
| **TUI (primary)**    | ratatui (Rust)       | `../../13-runtime/14-creature-system.md` (specifies the Golem creature sprite system and runtime rendering) |
| **Creature viewport**| `bardo-sprites` WASM | `../../13-runtime/14-creature-system.md` (procedural creature generation and animation) |

The web Portal can render creatures by embedding the `bardo-sprites` WASM module in an HTML5 Canvas -- same procedural generation as the TUI, different rendering target. But the creature interaction model (pet, steer, dream visualization, death cinematic) lives in the TUI, not the web components.

---

## Design philosophy

### Core principles

1. **ROSEDUST** -- Near-black surfaces (`bg_void #060608`) with rose as the primary color and restrained bone accents. Bone is earned, not decorative. Every bone element signals something meaningful: the single most important element on screen.

2. **Engineering aesthetic** -- PCB traces, hexagonal connectors, octagonal clip-paths, monospaced data. Hardware metaphors ground abstract financial concepts.

3. **Glass over void** -- Layered transparency creates depth without color. Glass cards float above the void-dark base. Backdrop blur separates content layers.

4. **Typography as hierarchy** -- Three fonts, three purposes. Serif for authority. Mono for data and truth. Sans for comfortable reading.

5. **Motion as meaning** -- Luxury-eased animations with deliberate pacing. Nothing snaps. Nothing bounces. Elements glide into position.

6. **Restraint** -- Dark surfaces need empty space. Gold loses its power when overused.

---

## Color system

### Foundation palette

| Token         | Value     | Usage                                 |
| ------------- | --------- | ------------------------------------- |
| `--black`     | `#0A0A0A` | Page background, deepest surface      |
| `--surface`   | `#111111` | Elevated sections                     |
| `--surface-2` | `#161616` | Secondary elevation (sidebar, panels) |

Never use pure `#000000`. The `0A0A0A` black has enough warmth to prevent the "hole in the screen" effect.

### Gold system

| Token         | Value                      | Usage                                           |
| ------------- | -------------------------- | ----------------------------------------------- |
| `--gold`      | `#C9A84C`                  | Primary brand -- headlines, active states, CTAs |
| `--gold-dim`  | `rgba(201, 168, 76, 0.20)` | Borders at rest, subtle dividers                |
| `--gold-mid`  | `rgba(201, 168, 76, 0.40)` | Borders on hover, scrollbar thumb               |
| `--gold-glow` | `rgba(201, 168, 76, 0.05)` | Hover backgrounds, ghost button fills           |

### Tier color system

| Tier | Name       | Color  | Hex       |
| ---- | ---------- | ------ | --------- |
| I    | Unverified | Gray   | `#4A4A4A` |
| II   | Basic      | Copper | `#8B5E3C` |
| III  | Verified   | Silver | `#A0A0A8` |
| IV   | Trusted    | Gold   | `#C9A84C` |
| V    | Sovereign  | Amber  | `#E8A020` |

### Debug UI adaptations

The debug UI (developer-facing) uses a blue-shifted color variant:

| Token            | Debug Value              | Purpose                  |
| ---------------- | ------------------------ | ------------------------ |
| `--black`        | `#0D1117`                | Blue-shifted base        |
| `--surface`      | `#161A20`                | Cooler surface elevation |
| `--surface-2`    | `#1E2430`                | Secondary elevation      |

---

## Typography

| Token          | Font             | Purpose                                      |
| -------------- | ---------------- | -------------------------------------------- |
| `--font-serif` | Instrument Serif | Display headlines, hero text, section titles |
| `--font-sans`  | General Sans     | Body copy, UI text, descriptions, navigation |
| `--font-mono`  | JetBrains Mono   | Addresses, numbers, code, data values        |

### Scale

| Level   | Size | Weight | Font  | Usage               |
| ------- | ---- | ------ | ----- | ------------------- |
| Display | 72px | 400    | Serif | Hero headlines only |
| H1      | 48px | 500    | Serif | Page titles         |
| H2      | 32px | 500    | Serif | Section headers     |
| H3      | 24px | 500    | Sans  | Subsection headers  |
| Body    | 16px | 400    | Sans  | Paragraph text      |
| Small   | 14px | 400    | Sans  | Labels, metadata    |
| Mono    | 14px | 400    | Mono  | Addresses, values   |

---

## Glass morphism system

Three elevation levels:

| Level | Background | Border | Blur |
|-------|-----------|--------|------|
| Glass-1 (Subtle) | `rgba(255,255,255,0.02)` | `rgba(255,255,255,0.05)` | 12px |
| Glass-2 (Standard) | `rgba(255,255,255,0.04)` | `rgba(255,255,255,0.08)` | 20px |
| Glass-3 (Prominent) | `rgba(255,255,255,0.06)` | `rgba(255,255,255,0.12)` | 30px |

---

## Component library

### Layout

| Component    | Purpose                                              |
| ------------ | ---------------------------------------------------- |
| `PageLayout` | Full-page wrapper with sidebar, header, content area |
| `GlassCard`  | Glass morphism container with configurable elevation |
| `PcbCard`    | PCB-traced card for hero statistics                  |
| `Section`    | Page section with optional title and gold divider    |
| `Sidebar`    | Navigation sidebar with active state indicators      |

### Data display

| Component        | Purpose                                               |
| ---------------- | ----------------------------------------------------- |
| `DataTable`      | Column-aligned table with sort, filter, pagination    |
| `StatCard`       | Key metric display with label, value, trend indicator |
| `AddressDisplay` | Truncated address with copy button and explorer link  |
| `TokenAmount`    | Formatted token amount with symbol and decimals       |
| `TierBadge`      | Reputation tier indicator with metal color            |
| `PriceChart`     | Recharts line chart with rose accent styling          |
| `LiquidityChart` | Tick-range bar chart for V3/V4 position visualization |

### Input

| Component         | Purpose                                              |
| ----------------- | ---------------------------------------------------- |
| `TokenInput`      | Amount input with token selector and balance display |
| `AddressInput`    | Address input with validation and ENS resolution     |
| `RangeSlider`     | Tick range selector for LP positions                 |
| `ChainSelector`   | Chain dropdown with chain icons                      |

### Feedback

| Component       | Purpose                                         |
| --------------- | ----------------------------------------------- |
| `Spinner`       | Animated loading indicator (gold dots)          |
| `Toast`         | Notification system with auto-dismiss           |
| `ConfirmDialog` | Transaction confirmation with detail breakdown  |
| `ProgressBar`   | Scenario progress with step count               |
| `AlertBanner`   | Full-width status banner (info, warning, error) |

---

## Hooks

| Hook                 | Purpose                                                           |
| -------------------- | ----------------------------------------------------------------- |
| `useRecharts`        | Lazy-loads recharts via dynamic import to avoid bundle bloat      |
| `useTheme`           | Theme context (production vs debug UI color variants)             |
| `useTierColor`       | Returns metal color for a given reputation tier                   |
| `useTokenFormat`     | Formats BigInt amounts with correct decimals and symbol           |
| `useAddressTruncate` | Truncates address with configurable prefix/suffix length          |
| `useAutoRefresh`     | Polling hook with configurable interval and `watchBlocks` support |
| `useClipboard`       | Copy-to-clipboard with toast feedback                             |
| `useMediaQuery`      | Responsive breakpoint detection                                   |

---

## Animation system

### Timing functions

| Token           | Value                                  | Usage                                      |
| --------------- | -------------------------------------- | ------------------------------------------ |
| `--ease-luxury` | `cubic-bezier(0.22, 1, 0.36, 1)`      | Primary easing -- all standard transitions |
| `--ease-subtle` | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | Micro-interactions, hover states          |
| `--ease-bounce` | `cubic-bezier(0.34, 1.56, 0.64, 1)`   | Emphasis (used sparingly)                  |

### Motion patterns

- **Page fade-in**: 250ms opacity transition on mount
- **Stagger delays**: 50ms per item in lists
- **Card hover**: 200ms scale(1.01) + border color transition
- **Scroll reveal**: IntersectionObserver with 100px threshold, 400ms fade-up

---

## Tech stack

| Library      | Version | Purpose                                  |
| ------------ | ------- | ---------------------------------------- |
| React        | ^19.0   | UI framework                             |
| Tailwind CSS | ^4.0    | Utility-first styling (CSS-first config) |
| Radix UI     | latest  | Accessible primitive components          |
| recharts     | ^2.0    | Charts (lazy-loaded via `useRecharts`)   |
| shadcn/ui    | latest  | Component templates (Radix + Tailwind)   |

### Package configuration

```jsonc
{
  "name": "@bardo/ui",
  "private": true,
  "type": "module",
  "exports": {
    ".": { "types": "./src/index.ts", "default": "./src/index.ts" },
    "./*": { "types": "./src/*.ts", "default": "./src/*.ts" }
  },
  "peerDependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }
}
```

Internal package -- no build step. Consumers import TypeScript source directly. Test environment uses `happy-dom`.

---

## Context-specific adaptations

| Feature       | Landing Page | Portal            | Debug UI                 |
| ------------- | ------------ | ----------------- | ------------------------ |
| Gold system   | Full         | Full              | Full                     |
| Glass cards   | Full         | Full              | Full                     |
| PCB cards     | Hero stats   | Hero stats        | Hero stats only          |
| Film grain    | Yes          | No                | No                       |
| Custom cursor | Yes          | No                | No                       |
| Color base    | `#0A0A0A`    | `#0A0A0A`         | `#0D1117` (blue-shifted) |

---

## Accessibility

- All interactive elements have visible focus indicators (gold ring)
- Color contrast ratio >= 4.5:1 for text, >= 3:1 for large text
- Radix UI primitives provide keyboard navigation and ARIA attributes
- `prefers-reduced-motion` disables all animations except opacity transitions
- Screen reader text provided for icon-only buttons
