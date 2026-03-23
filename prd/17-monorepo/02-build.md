# Build, Test, and Quality [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-15

> **Reader orientation:** This document specifies the build, test, and quality systems for the Bardo codebase, which spans a TypeScript monorepo (`gotts-monorepo`) and a Rust workspace (`bardo-golem-rs`). It belongs to the **Monorepo** section. The key concept before diving in: the two build systems are fully independent (TypeScript uses tsup/Turborepo/Vitest, Rust uses cargo/clippy/rustfmt) and run in parallel in CI. The Rust side produces a single static binary for the Golem (mortal autonomous DeFi agent) runtime. Terms like Golem and Grimoire are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

---

## Overview

The Bardo build system has two distinct layers. The TypeScript monorepo uses **tsup** for published package bundling, **Vitest 4.0+** for TypeScript testing, **Foundry** for Solidity testing, **ESLint 10** for linting, and **Prettier 3.5+** for formatting, with **Turborepo 2.8+** orchestrating all tasks. The Rust workspace (`bardo-golem-rs`) uses **cargo + cargo-nextest** for testing, **clippy** for linting, and **rustfmt** for formatting, producing a single static binary. The two build systems run in parallel in CI and are otherwise independent.

---

## Build: tsup

[tsup](https://github.com/egoist/tsup) v8.5.1 bundles published packages using esbuild under the hood. Only packages distributed to npm require a build step -- internal packages import TypeScript source directly.

> **Note:** tsup is used instead of tsdown due to a pnpm v9 compatibility issue with rolldown's optional native bindings. The API is identical.

### Standard tsup Config

Each published package has a `tsup.config.ts`:

```typescript
import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["esm", "cjs"],
  dts: true,
  clean: true,
  sourcemap: true,
  outDir: "dist",
});
```

| Option      | Value              | Purpose                                      |
| ----------- | ------------------ | -------------------------------------------- |
| `entry`     | `["src/index.ts"]` | Library entry point(s)                       |
| `format`    | `["esm", "cjs"]`   | Dual-format output for maximum compatibility |
| `dts`       | `true`             | Generate `.d.ts` declaration files           |
| `clean`     | `true`             | Remove `dist/` before each build             |
| `sourcemap` | `true`             | Source maps for debugging                    |

### Multi-Entry Packages

Packages with multiple entry points (e.g., vault SDK + vault tools):

```typescript
import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    index: "src/index.ts",
    tools: "src/tools/index.ts",
    client: "sdk/src/index.ts",
  },
  format: ["esm", "cjs"],
  dts: true,
  clean: true,
  sourcemap: true,
  outDir: "dist",
});
```

### CLI Package Build

The CLI bundles lightweight dependencies to minimize cold-start time:

```typescript
import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/cli.ts"],
  format: ["esm"],
  target: "node20",
  outDir: "dist",
  noExternal: ["@bardo/tui", "@bardo/core", "picocolors"],
  banner: { js: "#!/usr/bin/env node" },
});
```

### Package Scripts

Published packages include these scripts:

```jsonc
{
  "scripts": {
    "build": "tsup",
    "dev": "tsup --watch",
    "check-types": "tsc --noEmit",
    "clean": "rm -rf dist .turbo",
    "lint": "eslint .",
    "test": "vitest run",
  },
}
```

---

## Turborepo Pipeline

### `turbo.json`

```jsonc
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": ["**/.env.*local"],
  "ui": "tui",
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "tsconfig.json", "tsup.config.ts", "package.json"],
      "outputs": ["dist/**"],
    },
    "check-types": {
      "dependsOn": ["^check-types"],
      "inputs": ["src/**", "test/**", "tsconfig.json"],
      "outputs": [],
    },
    "dev": {
      "cache": false,
      "persistent": true,
    },
    "lint": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "test/**", "eslint.config.ts"],
      "outputs": [],
    },
    "test": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "test/**", "vitest.config.ts"],
      "outputs": [],
    },
    "test:integration": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "test/**"],
      "outputs": [],
      "cache": false,
    },
    "clean": { "cache": false },
  },
}
```

**Key decisions:**

1. `dependsOn: ["^build"]` -- Tasks wait for upstream workspace dependencies to build first
2. `inputs` arrays -- Only cache-relevant files listed; adding `README.md` would cause unnecessary cache misses
3. `dev` is not cached -- Dev servers are long-running (`persistent: true`)
4. `test:integration` is not cached -- Integration tests hit real RPC/subgraph endpoints

### Filtering

```bash
pnpm turbo run build --filter=@bardo/vault...     # Vault + dependencies
pnpm turbo run test --filter='...[main]'           # Changed since main
pnpm turbo run build --filter='./packages/*'       # All packages, no apps
```

---

## Testing: Vitest 4.0+

### Root Configuration (Projects Mode)

```typescript
// vitest.config.ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: ["packages/*/vitest.config.ts", "apps/*/vitest.config.ts"],
  },
});
```

### Per-Package Configuration

```typescript
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    name: "tools",
    environment: "node",
    include: ["test/**/*.test.ts"],
    globals: false,
    passWithNoTests: true,
    coverage: {
      provider: "v8",
      include: ["src/**/*.ts"],
      exclude: ["src/**/*.d.ts", "src/**/index.ts"],
      thresholds: {
        statements: 80,
        branches: 80,
        functions: 80,
        lines: 80,
      },
    },
  },
});
```

### Test File Conventions

| Convention | Rule                                                                  |
| ---------- | --------------------------------------------------------------------- |
| Location   | `test/` directory at package root                                     |
| Naming     | `*.test.ts` (unit), `*.integration.test.ts` (integration)             |
| Imports    | Explicit `import { describe, it, expect } from "vitest"` (no globals) |
| Mocking    | `vi.mock()` and `vi.fn()` -- avoid jest-style globals                 |
| Assertions | `expect()` API only                                                   |

### Test Patterns

- Use `TOOL_DEF.handler(params, defaultToolContext)` for testing Bardo tools
- Use `vi.fn()` for noop callbacks (not `() => {}`)
- Use type assertions (`as HTMLElement`) instead of non-null assertions
- Always `afterEach(cleanup)` for React 19 + @testing-library/react tests
- Use `Promise.allSettled` for parallel independent async operations

### Coverage Targets

| Metric     | Minimum | Target |
| ---------- | ------- | ------ |
| Statements | 80%     | 90%+   |
| Branches   | 80%     | 90%+   |
| Functions  | 80%     | 90%+   |
| Lines      | 80%     | 90%+   |

Safety-critical packages (`tools`, `vault`, `warden` (deferred)) MUST target 90%+. Coverage is enforced per-package, not monorepo-wide.

---

## Solidity Testing: Foundry

Each Solidity package has a `foundry.toml`:

```toml
[profile.default]
src = "contracts/src"
out = "out"
libs = ["lib"]
solc = "0.8.26"
optimizer = true
optimizer_runs = 200
via_ir = false
ffi = false

[profile.default.fuzz]
runs = 1000
max_test_rejects = 100_000

[profile.ci.fuzz]
runs = 10_000
```

### Test Categories

| Category    | Pattern               | Description                       |
| ----------- | --------------------- | --------------------------------- |
| Unit        | `test_*()`            | Single function behavior          |
| Fuzz        | `testFuzz_*(uint256)` | Property-based with random inputs |
| Invariant   | `invariant_*()`       | Stateful property preservation    |
| Integration | `testIntegration_*()` | Cross-contract interaction        |
| Fork        | `testFork_*()`        | Against mainnet fork              |

### Static Analysis

- **Slither**: Zero high-severity findings required for merge
- **Aderyn**: Complementary static analysis
- **Formal verification**: Certora and Halmos spec files in `contracts/certora/`

### Gas Benchmarking

Forge gas reports generated on every PR. Regressions > 10% flagged as warnings. Regressions > 25% block merge. Snapshots stored in `gas-snapshots/` for trend tracking.

---

## Linting: ESLint 10

### Shared Config Package

`@bardo/eslint-config` provides composable presets:

```typescript
// packages/eslint-config/src/base.ts
import eslint from "@eslint/js";
import perfectionist from "eslint-plugin-perfectionist";
import turboPlugin from "eslint-plugin-turbo";
import tseslint from "typescript-eslint";

export const base: Linter.Config[] = [
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    plugins: { perfectionist, turbo: turboPlugin },
    rules: {
      "perfectionist/sort-imports": [
        "error",
        {
          type: "natural",
          groups: [
            "builtin",
            "external",
            "internal",
            "parent",
            "sibling",
            "index",
            "type",
          ],
          newlinesBetween: "always",
        },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { prefer: "type-imports" },
      ],
      "prefer-const": "error",
      "no-var": "error",
    },
  },
];
```

Three presets: `base` (all packages), `library` (published packages, adds explicit return types), `nextjs` (React apps, relaxes misused-promises for event handlers).

### Per-Package Config

```typescript
import { library } from "@bardo/eslint-config";

export default [
  ...library,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  { ignores: ["dist/", "**/*.config.{ts,js,mjs}"] },
];
```

### Import Sorting

`eslint-plugin-perfectionist` handles deterministic import ordering. Chosen over `eslint-plugin-import` for zero performance overhead and full flat config support.

---

## Formatting: Prettier 3.5+

### `.prettierrc`

```jsonc
{
  "semi": true,
  "singleQuote": false,
  "tabWidth": 2,
  "trailingComma": "all",
  "printWidth": 80,
  "bracketSpacing": true,
  "endOfLine": "lf",
  "plugins": ["prettier-plugin-tailwindcss"],
}
```

Solidity formatting is handled by `forge fmt`, not Prettier. Add `*.sol` to `.prettierignore`.

---

## Pre-Commit Hooks

### Husky 9.x + lint-staged 16.x

```bash
# .husky/pre-commit
pnpm exec lint-staged

# .husky/commit-msg
pnpm exec commitlint --edit $1
```

### lint-staged Config

```javascript
// lint-staged.config.mjs
export default {
  "*.{ts,tsx}": ["eslint --fix", "prettier --write"],
  "*.{json,md,yml,yaml}": ["prettier --write"],
  "*.sol": ["forge fmt"],
};
```

### Commit Messages

commitlint 19.x enforces Conventional Commits:

```text
feat(vault): add ERC-4626 deposit tool
fix(tools): handle missing pool address in get_pool_info
chore(deps): update viem to 2.30.0
```

Allowed scopes: `tools`, `vault`, `warden`, `ui`, `tui`, `cli`, `portal`, `dev`, `testnet`, `core`, `chain`, `crypto`, `policy`, `wallet`, `ci`, `deps`, `monorepo`.

---

## CI Pipeline

### PR CI

```yaml
steps:
  - pnpm install
  - pnpm build
  - pnpm lint
  - pnpm format:check
  - pnpm test
  - forge test
  - forge test --gas-report
```

### Nightly CI

```yaml
steps:
  - All PR CI steps
  - pnpm test:integration # Tier 2 (Anvil required)
  - pnpm test:swarm # Tier 3 (full swarm)
  - forge test --ffi -vvv # Extended fuzz (10K runs)
  - slither contracts/src/ # Static analysis
  - aderyn contracts/ # Complementary analysis
```

### Dependency Maintenance

- **Knip**: Unused dependency detection, run weekly
- **Syncpack**: Cross-workspace version consistency
- **Renovate**: Automated dependency update PRs

---

## Rust Workspace: bardo-golem-rs

The `bardo-golem-rs` Rust workspace compiles to a single static binary (`bardo-golem`). It is a sibling repository to the TypeScript monorepo, not a subdirectory. The TypeScript monorepo handles provisioning, tooling, and web surfaces; the Rust workspace handles the live agent runtime. See `17-monorepo/01-rust-workspace.md` for the full crate DAG and dependency list.

### Build

```bash
cargo build --workspace               # Debug build (fast)
cargo build --workspace --release     # Release build (optimized, ~2 min cold)
cargo build -p golem-binary --release # Single-binary release artifact
```

The release binary is statically linked (`x86_64-unknown-linux-musl`). No dynamic dependencies. Ships as a ~25MB Docker image layer.

### Testing: cargo-nextest

`cargo nextest` replaces the default `cargo test`. It provides parallel test execution, per-test timeouts, and structured output:

```bash
cargo nextest run --workspace         # All tests, parallel
cargo nextest run -p golem-mortality  # Single crate
cargo nextest run --test-threads=4    # Bounded parallelism for RPC tests
```

`nextest.toml` at workspace root:

```toml
[profile.default]
test-threads = "num-cpus"
slow-timeout = { period = "60s", terminate-after = 3 }
fail-fast = false

[profile.ci]
fail-fast = true
slow-timeout = { period = "120s", terminate-after = 2 }
```

### Linting: clippy

```bash
cargo clippy --workspace --all-features -- -D warnings
```

`clippy.toml` at workspace root:

```toml
# Enforce pedantic rules in library crates
[workspace]
# See 17-monorepo/03-conventions.md for per-lint configuration
```

All clippy warnings are errors in CI (`-D warnings`). The `pedantic` lint group is enabled for library crates; `restriction` lints are off by default.

### Formatting: rustfmt

```bash
cargo fmt --all           # Format all crates
cargo fmt --all -- --check  # Check formatting (CI)
```

`rustfmt.toml` at workspace root:

```toml
edition = "2024"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### Coverage

```bash
cargo llvm-cov nextest --workspace --lcov --output-path coverage.lcov
```

`cargo-llvm-cov` produces LCOV reports consumed by CI coverage gates. Safety-critical crates (`golem-safety`, `golem-mortality`, `golem-chain`) require 90%+ line coverage.

### Adding Rust tasks to CI

The CI pipeline (`prd2/16-testing/`) runs TypeScript and Rust in parallel. The Rust pipeline:

```yaml
rust-ci:
  steps:
    - cargo build --workspace
    - cargo fmt --all -- --check
    - cargo clippy --workspace --all-features -- -D warnings
    - cargo nextest run --workspace --profile ci
    - cargo llvm-cov nextest --workspace (coverage gate)
```
