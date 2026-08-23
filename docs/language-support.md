# Workshop Language Support Matrix

This document is the **single authoritative index for declared `workshop-rs` support status** across the Overwatch Workshop language surface.

It provides a centralized overview of language support and links directly to individual component inventories:

## Support Semantics

Every capability in this document and its referenced sub-documents uses one of three visible support states:

- `✅ Supported`: The capability is part of the currently supported `workshop-rs` contract for its stated scope.
- `🚧 Coming soon`: The capability is known and intended for support, but the current release does not yet provide the complete user-visible behavior.
- `❌ Unsupported`: The capability is intentionally outside the supported contract or has no planned support under the current project scope.

> [!NOTE]
> `workshop-rs` is a compiler, parser, validator, emitter, and semantic core for the Overwatch Workshop language. Live runtime execution/simulation within the Overwatch game client is an engine concern and is classified as `❌ Unsupported`.

## Language Surface Overview

| Surface Area | Status | Scope / Capabilities | Specification & Inventory |
| --- | --- | --- | --- |
| [Program Structure & Variables](language-support/structure.md) | ✅ Supported | `settings`, `variables`, `subroutines`, `rule`, `disabled` modifiers, global & player variables | [`structure.md`](language-support/structure.md) |
| [Events & Event Filters](language-support/events.md) | ✅ Supported | 14 canonical rule events (`global`, `eachPlayer`, knockback, damage, healing, etc.), Team & Player filters | [`events.md`](language-support/events.md) |
| [Conditions & Control Flow](language-support/control-flow.md) | ✅ Supported | Conditions, `If`, `Else If`, `Else`, `End`, `While`, `For`, `Loop`, `Skip`, `Wait`, `Abort`, `Return` | [`control-flow.md`](language-support/control-flow.md) |
| [Operators & Variable Modifications](language-support/operators.md) | ✅ Supported | Comparison operators (`==`, `!=`, etc.), arithmetic (`Add`, `Subtract`, `Raise To Power`), array modifications | [`operators.md`](language-support/operators.md) |
| [Actions Inventory](language-support/actions.md) | ✅ Supported | Complete inventory of all 219 canonical Workshop actions with parameter signatures | [`actions.md`](language-support/actions.md) |
| [Values Inventory](language-support/values.md) | ✅ Supported | 258 supported values & expressions | [`values.md`](language-support/values.md) |
| [Enumerated Domains](language-support/enums.md) | ✅ Supported | Complete inventory of all 52 canonical enum domains (`Button`, `Color`, `Hero`, `Map`, `Gamemode`, etc.) | [`enums.md`](language-support/enums.md) |
| [Custom-Game Settings](language-support/settings.md) | ✅ Supported | `main`, `lobby`, `modes`, `heroes`, `extensions`, and `workshop` settings blocks | [`settings.md`](language-support/settings.md) |
| [Strings & Localization](language-support/strings.md) | ✅ Supported | `Custom String`, `String`, preset strings, `en-US`, `zh-CN`, bidirectional conversion (additional locales remain `🚧 Coming soon`) | [`strings.md`](language-support/strings.md) |
| [Tooling & Semantic Capabilities](language-support/tooling.md) | ✅ Supported | Parsing, validation, deterministic emission, conversion, hero gameplay query APIs, offline census | [`tooling.md`](language-support/tooling.md) |
| [Intentionally Out-of-Scope Capabilities](#intentionally-out-of-scope-capabilities) | ❌ Unsupported | Live client simulation/VM, source-language syntax (OverPy/DEL), dynamic runtime `eval` | [See below](#intentionally-out-of-scope-capabilities) |

## Intentionally Out-of-Scope Capabilities

| Feature | Status | Notes |
| --- | --- | --- |
| Live Workshop runtime / VM simulation | ❌ Unsupported | `workshop-rs` is a compiler and semantic analysis engine; executing gameplay simulation in real-time is an Overwatch engine function. |
| Source-language syntax (OverPy / DEL / OSTW) | ❌ Unsupported | OverPy syntax and macros belong to `opy-rs`; DEL/OSTW syntax and features belong to `del-rs`. |
| Dynamic script evaluation (`eval`) | ❌ Unsupported | Workshop language semantics do not include dynamic code evaluation or runtime code generation. |
