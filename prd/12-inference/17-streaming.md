# Streaming UX for Provider-Specific Features

> **Document Type**: SPEC (normative) | **Version**: 3.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-14
>
> **Package**: `@bardo/golem` (`bardo-ui-bridge`), `@bardo/runtime`
>
> **Depends on**: `prd2-streaming-ux.md`, `prd2-bardo-inference-architecture.md`, `prd2-reasoning-chain-integration.md`, `prd2-bankr-self-funding.md`
>
> **Purpose**: How Bardo Inference's multi-backend responses -- each with different streaming formats, reasoning shapes, and metadata -- are parsed into unified BardoEventBus events and rendered across web/TUI/Telegram/Discord surfaces.

---

> **Reader orientation:** This document specifies the streaming UX layer of Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and describes how provider-specific SSE streaming formats (Anthropic content_block_delta, OpenAI choices delta, DeepSeek think tags, Gemini alt=sse) are parsed into unified BardoEventBus events and rendered across web, TUI, Telegram, and Discord surfaces. The key concept is that users see consistent streaming behavior regardless of which backend provider is handling the request. For term definitions, see `prd2/shared/glossary.md`.

## The Thesis

Bardo Inference routes requests to different backends, and each backend returns responses in different streaming formats. Claude streams `thinking_delta` events. DeepSeek embeds `<think>` tags inline. OpenAI streams `reasoning.summary_text.delta`. Gemini streams partial function arguments. The `bardo-ui-bridge` must parse all of these into a single event protocol that surfaces can render -- without exposing backend implementation details to the user.

The user sees **what the Golem is doing**, not which backend is doing it. But power users who want to know can see provider metadata in tooltips and debug panels.

---

## Part 1: Bardo Inference Response Metadata

Every response from Bardo Inference includes metadata about which backend handled the request:

```typescript
// X-Bardo-Backend header in responses
interface BardoResponseMeta {
  backend: string;            // "blockrun", "openrouter", "venice", "bankr", "direct_anthropic", etc.
  model: string;              // Actual model used
  securityClass: "standard" | "confidential" | "private";
  contextEngineering: {
    cacheHit: boolean;        // Semantic or hash cache hit
    promptCacheTokens: number; // Tokens served from provider cache
    tokensSaved: number;       // Total tokens saved by pipeline
    compressionApplied: boolean;
  };
  cost: {
    backendCostUsd: number;   // What Bardo paid the backend
    userCostUsd: number;      // What the user pays (backend + spread)
    savingsVsDirect: number;  // Percentage saved vs. direct call
  };
  flags: {
    wasCompacted: boolean;
    wasPrivate: boolean;
    wasSelfFunded: boolean;
    wasDiemFunded: boolean;
    wasCrossVerified: boolean;
    wasBatchProcessed: boolean;
  };
}
```

---

## Part 2: Stream Parsing by Backend

### 2.1 Unified Stream Parser

```typescript
class BardoStreamParser {
  /**
   * Parse streaming events from any Bardo Inference backend.
   * Bardo Inference normalizes most differences, but some provider-specific
   * event types (thinking, <think> tags) need special handling.
   */
  *parse(event: SSEEvent, meta: BardoResponseMeta): Generator<BardoUIEvent> {
    // Anthropic-format responses (from BlockRun/OpenRouter/Bankr/Direct routing to Claude)
    if (event.type?.startsWith("content_block")) {
      yield* this.parseAnthropicEvent(event, meta);
    }
    // OpenAI-format responses (from any backend routing to GPT/Gemini/etc.)
    else if (event.choices || event.type?.startsWith("response.")) {
      yield* this.parseOpenAIEvent(event, meta);
    }

    // <think> tag detection applies to ALL backends (DeepSeek/Qwen can come from any)
    if (this.isInlineThinkingStream(event)) {
      yield* this.parseThinkTags(event, meta);
    }
  }

  private *parseAnthropicEvent(event: SSEEvent, meta: BardoResponseMeta): Generator<BardoUIEvent> {
    if (event.type === "content_block_start" && event.content_block?.type === "thinking") {
      yield this.emit("reasoning:start", {
        provider: meta.model,
        visibility: "summarized",
        subsystem: this.currentSubsystem,
        backend: meta.backend,
      });
    }
    else if (event.delta?.type === "thinking_delta") {
      yield this.emit("reasoning:phase", {
        provider: meta.model,
        visibility: "summarized",
        content: event.delta.thinking,
      });
    }
    else if (event.delta?.type === "text_delta") {
      yield this.emit("stream:chunk", {
        content: event.delta.text,
        done: false,
        requestId: this.requestId,
      });
    }
    else if (event.delta?.type === "input_json_delta") {
      yield this.emit("tool:progress", {
        toolCallId: this.currentToolId,
        message: "Building parameters...",
        phase: "processing",
        data: { partialJson: event.delta.partial_json },
      });
    }
  }

  private *parseOpenAIEvent(event: SSEEvent, meta: BardoResponseMeta): Generator<BardoUIEvent> {
    // OpenAI Responses API format
    if (event.type === "response.reasoning.summary_text.delta") {
      yield this.emit("reasoning:phase", {
        provider: meta.model,
        visibility: "summarized",
        content: event.delta,
      });
    }
    // Standard chat completions format
    else if (event.choices?.[0]?.delta?.content) {
      // Check for inline <think> tags (DeepSeek/Qwen via OpenAI-compatible format)
      // The think tag parser handles this separately
      yield this.emit("stream:chunk", {
        content: event.choices[0].delta.content,
        done: false,
        requestId: this.requestId,
      });
    }
  }

  /**
   * Parse <think>...</think> tags from inline content stream.
   * Applies to DeepSeek R1 and Qwen3 models from ANY backend.
   */
  private *parseThinkTags(event: SSEEvent, meta: BardoResponseMeta): Generator<BardoUIEvent> {
    const content = this.extractContent(event);

    for (const char of content) {
      this.tagBuffer += char;

      if (this.tagBuffer.endsWith("<think>")) {
        this.isInThinkTag = true;
        this.thinkContent = "";
        this.tagBuffer = "";
        yield this.emit("reasoning:start", {
          provider: meta.model,
          visibility: "visible",
          backend: meta.backend,
        });
      }
      else if (this.tagBuffer.endsWith("</think>")) {
        this.isInThinkTag = false;
        this.tagBuffer = "";
        yield this.emit("reasoning:end", {
          provider: meta.model,
          visibility: "visible",
          content: this.thinkContent,
          reasoningTokens: Math.ceil(this.thinkContent.length / 4),
        });
        this.thinkContent = "";
      }
      else if (this.isInThinkTag) {
        this.thinkContent += char;
        // Emit chunks periodically
        if (this.thinkContent.length % 80 === 0) {
          yield this.emit("reasoning:phase", {
            provider: meta.model,
            visibility: "visible",
            content: this.thinkContent.slice(-150),
          });
        }
      }
    }
  }
}
```

---

## Part 3: Surface-Specific Rendering

### 3.1 Provider Badge (All Surfaces)

Every surface shows what's handling the current operation -- but at the **model** level, not the backend level. Users see "Claude Opus 4.6" not "BlockRun":

**Web Chat**:
```
+- Provider ------------------------------------------+
| Claude Opus 4.6     |  Cache: 92% hit              |
| DeepSeek R1         |  Private | DIEM              |  <- Venice backend (shown as "Private")
| Claude Opus 4.6     |  Self-funded                 |  <- Bankr backend
| Qwen Plus           |  /think mode                 |
| Qwen3-7B            |  Local | Free                |
+-----------------------------------------------------+
```

**TUI**:
```
+------------------------------------------------------+
| Claude/Opus | v0.67 | $142.50 | Cache:92%            |
+------------------------------------------------------+
```

**Telegram**: Footer: `via Claude` or `via DeepSeek (private)`
**Discord**: Bot presence: `Thinking with Claude` or `Dreaming privately`

### 3.2 Reasoning Rendering

| Visibility | Web | TUI | Telegram | Discord |
|---|---|---|---|---|
| **Visible** (`<think>`) | Collapsible panel, scrolling text, phase indicators | Dedicated reasoning pane | Omitted | Spoiler embed |
| **Summarized** (Claude/OpenAI) | Brief card | One-line status | In message | Embed field |
| **Opaque** (redacted) | Lock indicator | Lock icon | Omitted | "Thinking privately" |
| **None** | No indicator | No indicator | No indicator | No indicator |

### 3.3 Context Engineering UX

Bardo Inference's context engineering savings are shown as a subtle indicator:

**Web Chat**: "Context: 92% cached - 15K tokens pruned - $0.14 saved" in footer
**TUI**: `Cache:92% | Saved:$0.14` in status bar

### 3.4 Compaction UX

```
+--- SESSION COMPACTED -------------------------------------------+
| Context summarized: 142K -> 8.2K tokens                         |
| Preserved: vault state, 3 permits, risk tier                    |
| Quality: 60%                                                     |
| [View Summary] [Start New Session]                               |
+-----------------------------------------------------------------+
```

### 3.5 Bankr Dashboard Events

When Bankr backend is active:

```typescript
type BankrDashboardEvent =
  | { type: "bankr:balance"; balance: number; projectedDays: number }
  | { type: "bankr:inference_cost"; model: string; cost: number }
  | { type: "bankr:revenue"; source: string; amount: number }
  | { type: "bankr:sustainability"; ratio: number; trend: string }
  | { type: "bankr:cross_verified"; models: string[]; agreement: number };
```

**TUI**:
```
+--- METABOLISM ---------------------------------------------------+
| Revenue: $52.30/d | Costs: $15.70/d | Ratio: 3.3x               |
| SELF-SUSTAINING                                                   |
+-----------------------------------------------------------------+
```

### 3.6 Dream Mode

When dreaming, all surfaces switch to dream rendering:

**Web**: Background dims. Dream journal panel appears with visible reasoning (if R1/Qwen) or summary cards (if Claude).

**TUI**: Terminal dims. "DREAM MODE" banner. Reasoning pane shows dream narration.

**Telegram**: "Dreaming... insights when I wake."

**Discord**: Bot status -> "Sleeping". Dream thread for followers.

---

## Part 4: Graceful UX Degradation

Missing features are invisible. Present features are celebrated.

| Missing | UX Impact |
|---|---|
| No visible reasoning | Reasoning panel hidden |
| No Compaction | Session length warning at 80% capacity instead |
| No Citations | No provenance links on Grimoire entries |
| No Bankr | No metabolism dashboard |
| No Venice | No privacy indicator |
| No local | T0 uses cheapest remote model (slightly slower) |
| No cross-model verify | Risk shows single-model result |

---

## References

- [VAN-DE-MERWE-2024] Van de Merwe, D. et al. "Four Transparency Levels for Agent UX." J. Cog. Eng., 2024. Defines a framework for how much agent reasoning to expose to users; informs Bardo's streaming transparency settings (show thinking, show cost, show provider).
- [A2UI-2025] A2UI Protocol. "Declarative Streaming JSON for Agent UX." a2ui.org, 2025. Proposes a standard format for agent-to-UI streaming that separates content, metadata, and control signals; informs the BardoEventBus event structure.
- [VERCEL-AI-SDK-2025] Vercel. "Data Stream Protocol." 2025. Documents Vercel's approach to streaming LLM responses through middleware; informs the SSE normalization layer that handles provider-specific chunk formats.
- [REYES-2025] Reyes, M. et al. "Uncertainty Visualization and Trust." Frontiers in CS, 2025. Shows that exposing model uncertainty in UIs increases user trust and decision quality; informs the reasoning trace rendering that shows confidence alongside outputs.
- [FACTORY-COMPRESSION-2026] Factory.ai. "Evaluating Context Compression." 2026. Evaluates compression techniques for long conversations; informs the streaming compaction that summarizes prior turns for display.
