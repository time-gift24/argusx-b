# LLM SDK Crate Design

## Overview

Create a new `llm-sdk` crate that implements the traits and domain models from the design doc, with BigModel as the first provider.

## Project Structure

```
llm-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports
    ├── domain.rs       # Core domain models (from design doc)
    ├── traits.rs       # 10 trait definitions
    ├── error.rs       # Error types
    └── providers/
        └── bigmodel/
            ├── mod.rs
            └── client.rs
```

## Components

### 1. Domain (domain.rs)

From `docs/plans/2026-02-16-llm-sdk-traits-core-domain-only.md`:

- `Role` - User, Assistant, Tool
- `Part` enum - Text, Image, Audio, Source, ToolCall, ToolResult, Reasoning
- `Message` - User, Assistant, Tool variants
- `ModelUsage`, `ModelResponse`, `PartialModelResponse`
- `LanguageModelInput`, `ToolDefinition`, `ToolChoice`, `ResponseFormat`
- `AgentItem`, `AgentResponse`, `AgentStreamEvent`
- `MemoryBlock`, `MemorySearchHit`, `PlanSnapshot`
- `RunCheckpoint`, etc.

### 2. Traits (traits.rs)

10 traits from design doc:

- `LanguageModelTrait` - generate(), stream()
- `RunSessionTrait` - init(), run(), run_stream(), close()
- `RunStateView`, `RunStateTrait`
- `AgentToolTrait`
- `DelegationToolTrait`
- `CoreMemoryStoreTrait`
- `ArchivalMemoryStoreTrait`
- `PlanStoreTrait`
- `ApprovalGateTrait`
- `InterruptionResumeTrait`

### 3. BigModel Provider

- Uses existing `bigmodel-api` crate as HTTP client
- Implements all 10 traits
- Maps BigModel API responses to domain types

### 4. Error Types

```rust
pub enum ModelError {
    InvalidRequest(String),
    AuthenticationError(String),
    RateLimitError(String),
    ServerError(String),
    NetworkError(String),
    ParseError(String),
}
```

## Dependencies

```toml
[dependencies]
bigmodel-api = { path = "../bigmodel-api" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["rt", "macros"] }
futures = "0.3"
```
