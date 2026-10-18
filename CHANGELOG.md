# Changelog

All notable changes to **Lumis AI Gateway** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2026-10-18

### Added
- **Phase 15: Native Intelligent Generative UI & OpenUI Engine**:
  - Client-side Pratt recursive descent AST parser and tokenizer for `openui-lang 1.0` in Svelte 5.
  - Reactive two-way state binding with `$variables` and automatic hoisted expression re-evaluation.
  - Zero-server client execution of actions (`@Set`, `@Reset`, `@CopyToClipboard`, `@ToAssistant`) and 50+ built-in mathematical, string, array, and formatting functions (`@Round`, `@Sum`, `@Percent`, `@Each`, etc.).
  - Official `shadcn-svelte` styled primitives (`Card`, `Col`, `Row`, `Grid`, `Slider`, `Input`, `SegmentedControl`, `Checkbox`, `Table`, `Stat`, `Button`) with GPU-composited 120Hz/144Hz animations.
  - Dual-view mode (Live Interactive UI vs OpenUI DSL Code view), 1-click clipboard copy, and Expand to Fullscreen Canvas popout modal.
- **Strict LLM Prompt Guard & Tool Calling Constraint**:
  - Added directive in chat assembly and system prompts prohibiting LLMs from calling backend execution tools (`execute_code`, `gateway_plan_tasks`) when users request inline calculators, steppers, or widgets.
  - Injected concrete split bill calculator syntax into LLM system prompts for zero-shot accuracy.

## [1.0.0] - 2026-10-13

### Added
- **Decoupled Identity Federation Pattern**:
  - `user_identities` table supporting 1-to-N federated account linking (SAML, OIDC, Ingress Proxy, Local Password, SCIM).
  - Immutable subject mapping (`provider`, `provider_subject_id`) ensuring persistence across username/email updates.
- **Relational SCIM 2.0 Persistence Engine (RFC 7643 & RFC 7644)**:
  - Relational tables `scim_users` and `scim_groups` in SQLite WAL and PostgreSQL.
  - Auto-generated UUIDs on user creation when client omits `id`.
  - RFC 7644 `PATCH` operations support (`{"Operations": [{"op": "replace", "path": "active", "value": false}]}`).
  - Cascading account deactivation: deactivating a user in SCIM immediately sets `users.is_active = false`.
- **Policy Enforcement Point (PEP) Invariants**:
  - Immediate `HTTP 403 Forbidden` response across `/api/auth/login`, `/api/auth/me`, `/api/sessions`, and message streams for deactivated accounts.
  - Role hierarchy gating (`SuperAdmin` down to `Guest`) with clearance tier enforcement.
  - Dedicated SuperAdmin status management endpoint `/api/admin/users/status`.
- **High-Performance Architecture Baseline**:
  - Pure async Rust Axum 0.8 backend (< 25MB RAM consumption).
  - Pluggable relational database driver supporting SQLite WAL (mode=rwc) and PostgreSQL connection pool.
  - Svelte 5 frontend with Runes reactivity, Google Sans typography, shadcn-svelte components, and GPU-composited 120Hz/144Hz animations.
  - SIMD Aho-Corasick DLP scanner and prompt injection guardrails.
  - Multi-vendor resilient LLM router with automatic failover and token budget quota enforcement.

### Changed
- Re-architected user authentication to rely solely on the database as single source of truth with SHA-256 salted hashing.
- Modularized password management into dedicated `routes/password.rs` ensuring strict adherence to `< 350 lines` limit.
- Generified upstream defaults in `routes/saml.rs` and `docker-compose.coolify.yml` to standard enterprise fallbacks.

### Security
- SOC emergency killswitch with sub-millisecond atomic flag enforcement.
- Strict `#![deny(unsafe_code)]` enabled across all workspace crates.
- Verified 100% Zero-Mock & Silent Mock Static Compliance via AST analysis (`detect-mocks .`).
