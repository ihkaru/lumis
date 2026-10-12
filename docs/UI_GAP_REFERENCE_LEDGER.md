# 🏛️ Lumis vs Llumen UI Gap Reference Ledger
> **Authority**: October 2026 Enterprise UI Engineering Standards  
> **Repository**: `ipds6104/lumis` & `ipds6104/lumis-enterprise-stack`  
> **Comparative Baseline**: Llumen Frontend (`/root/projects/llumen/frontend`)  
> **Status**: ACTIVE & DETERMINISTICALLY ENFORCED

Ledger ini adalah catatan kanonikal permanen dari seluruh gap UI yang telah dianalisis, diriset, dan diselesaikan. Seluruh agen AI **WAJIB** membaca ledger ini terlebih dahulu sebelum melakukan analisis ulang terhadap Llumen agar tidak melakukan pekerjaan berulang (*anti-duplikasi & zero redundant re-research*).

---

## 📋 Ledger Registry Summary

| Range ID | Domain / Kategori | Jumlah Gap | Status |
| :--- | :--- | :---: | :---: |
| `UI-GAP-001` .. `UI-GAP-003` | Routing & Error Boundaries | 3 | **RESOLVED** |
| `UI-GAP-004` .. `UI-GAP-013` | Message Viewport & AST Rendering | 10 | **RESOLVED** |
| `UI-GAP-014` .. `UI-GAP-018` | Input Composer & Multimodal Ingress | 5 | **RESOLVED** |
| `UI-GAP-019` .. `UI-GAP-022` | Sidebar & Session Lifecycle | 4 | **RESOLVED** |
| `UI-GAP-023` | Settings, Themes & Administration | 1 | **RESOLVED** |
| `UI-GAP-024` .. `UI-GAP-026` | Iteration 1: Message Branching, Deep Research & Lightbox | 3 | **RESOLVED** |
| `UI-GAP-027` .. `UI-GAP-029` | Iteration 2: Action Menu, File Groups & Convert-to-File | 3 | **RESOLVED** |
| `UI-GAP-030` .. `UI-GAP-032` | Iteration 3: Provider Keys, Confidence & Minimap | 3 | **RESOLVED** |

---

## 🔎 Canonical Entry Details

### [Phase 1-4: Baseline UI Parity]

#### `UI-GAP-001`: Stateful Chat Session Routing
- **Llumen Ref**: `src/routes/chat/[id]/+page.svelte`
- **Lumis Target**: `ui/src/lib/router.svelte.ts`
- **Status**: **RESOLVED**
- **Traces**: Svelte 5 reactive hash/history router parsing `#/chat/:id`, parameter extraction, 0ms tab sync.

#### `UI-GAP-002`: Dedicated Authentication & Login Screen
- **Llumen Ref**: `src/routes/login/+page.svelte`
- **Lumis Target**: `ui/src/lib/components/auth/LoginPage.svelte`
- **Status**: **RESOLVED**
- **Traces**: Enterprise SSO & password login form, error alerts, zero emoticons, lucide vector icons.

#### `UI-GAP-003`: Graceful Error Boundary
- **Llumen Ref**: `src/routes/+error.svelte`
- **Lumis Target**: `ui/src/lib/components/common/ErrorBoundary.svelte`
- **Status**: **RESOLVED**
- **Traces**: Global and component-level catch boundary with retry CTA and clean diagnostic logging.

#### `UI-GAP-004`: Streaming Markdown AST Parser & Renderer
- **Llumen Ref**: `src/lib/components/markdown/Parser.svelte`
- **Lumis Target**: `ui/src/lib/components/markdown/MarkdownRenderer.svelte`
- **Status**: **RESOLVED**
- **Traces**: `marked` token-based AST compilation with zero layout reflows during active token streaming.

#### `UI-GAP-005`: Syntax-Highlighted Code Blocks with 1-Click Copy & Wrap Toggle
- **Llumen Ref**: `src/lib/ui/code/Code.svelte`
- **Lumis Target**: `ui/src/lib/components/markdown/CodeBlock.svelte`
- **Status**: **RESOLVED**
- **Traces**: Language chip, wrap toggle, clipboard feedback animation, `[overflow-wrap:anywhere]`.

#### `UI-GAP-006`: Collapsible Chain-of-Thought Reasoning Box
- **Llumen Ref**: `src/lib/components/message/Reasoning.svelte`
- **Lumis Target**: `ui/src/lib/components/message/ReasoningBox.svelte`
- **Status**: **RESOLVED**
- **Traces**: Auto-collapsed `<think>` accordion, pulse thinking indicator, elapsed token/second timer.

#### `UI-GAP-007`: Tool & MCP Function Invocation Cards
- **Llumen Ref**: `src/lib/components/message/Tool.svelte` & `ToolBox.svelte`
- **Lumis Target**: `ui/src/lib/components/message/ToolCard.svelte`
- **Status**: **RESOLVED**
- **Traces**: Server name, tool name, expandable JSON payload/result, success/error status badges.

#### `UI-GAP-008`: Scientific LaTeX / KaTeX Math Rendering
- **Llumen Ref**: `src/lib/components/latex/Latex.svelte`
- **Lumis Target**: `ui/src/lib/components/markdown/LatexRenderer.svelte`
- **Status**: **RESOLVED**
- **Traces**: Inline math `$E=mc^2$` and block math `$$\dots$$` rendered with zero hydration error.

#### `UI-GAP-009`: Interactive Mermaid Architecture Diagrams
- **Llumen Ref**: `src/lib/ui/mermaid/Mermaid.svelte`
- **Lumis Target**: `ui/src/lib/components/markdown/MermaidRenderer.svelte`
- **Status**: **RESOLVED**
- **Traces**: Client-side SVG graph compilation with pan/zoom and error fallback.

#### `UI-GAP-010`: Message Action Bar (Copy, Edit, Regenerate, Fork)
- **Llumen Ref**: `src/lib/components/message/Result.svelte` & `ResponseEdit.svelte`
- **Lumis Target**: `ui/src/lib/components/message/MessageActions.svelte`
- **Status**: **RESOLVED**
- **Traces**: Action icon bar shown on hover/focus with copy, edit, retry, and fork triggers.

#### `UI-GAP-011`: User Message Bubble
- **Llumen Ref**: `src/lib/components/message/User.svelte`
- **Lumis Target**: `ui/src/lib/components/message/UserBubble.svelte`
- **Status**: **RESOLVED**
- **Traces**: User avatar, inline edit mode, timestamp, multi-line bubble styling.

#### `UI-GAP-012`: Assistant Message Bubble
- **Llumen Ref**: `src/lib/components/message/Assistant.svelte`
- **Lumis Target**: `ui/src/lib/components/message/AssistantBubble.svelte`
- **Status**: **RESOLVED**
- **Traces**: Integrated markdown renderer, reasoning box, tool cards, citations, and actions.

#### `UI-GAP-013`: Grounded RAG Citations & Source Viewer
- **Llumen Ref**: `src/lib/components/message/Citations.svelte`
- **Lumis Target**: `ui/src/lib/components/message/CitationsView.svelte`
- **Status**: **RESOLVED**
- **Traces**: Numbered superscript badges and source chip drawer with URL links.

#### `UI-GAP-014`: Streaming Abort & Generation Stop Controller
- **Llumen Ref**: `src/lib/components/input/Stop.svelte`
- **Lumis Target**: `ui/src/lib/components/input/MessageInput.svelte`
- **Status**: **RESOLVED**
- **Traces**: `AbortController.abort()` attached to stop button; swaps send icon to square stop icon.

#### `UI-GAP-015`: Drag-and-Drop & Clipboard Image/File Attachments
- **Llumen Ref**: `src/lib/components/input/Upload.svelte`
- **Lumis Target**: `ui/src/lib/components/input/FileDropzone.svelte`
- **Status**: **RESOLVED**
- **Traces**: Dragover overlay, dragleave cleanup, and clipboard `paste` event ingestion.

#### `UI-GAP-016`: Dynamic Model Selector in Composer
- **Llumen Ref**: `src/lib/components/input/ModelSelector.svelte`
- **Lumis Target**: `ui/src/lib/components/input/ModelSelector.svelte`
- **Status**: **RESOLVED**
- **Traces**: Dropdown selecting active model from configured catalog.

#### `UI-GAP-017`: Search & Deep Research Mode Toggles
- **Llumen Ref**: `src/lib/components/input/ModeSelector.svelte`
- **Lumis Target**: `ui/src/lib/components/input/ModeSelector.svelte`
- **Status**: **RESOLVED**
- **Traces**: Quick toggles for Standard, Web Search, and Deep Research modes.

#### `UI-GAP-018`: Audio Recording Modal
- **Llumen Ref**: `src/lib/components/input/RecordAudioModal.svelte`
- **Lumis Target**: `ui/src/lib/components/input/AudioRecordModal.svelte`
- **Status**: **RESOLVED**
- **Traces**: Web Audio API recording with live duration timer and waveform animation.

#### `UI-GAP-019`: Instant Keyset Session Search & Filter
- **Llumen Ref**: `src/lib/components/sidebar/Header.svelte`
- **Lumis Target**: `ui/src/lib/components/sidebar/SessionSearch.svelte`
- **Status**: **RESOLVED**
- **Traces**: Real-time keystroke filtering across session titles with clear button.

#### `UI-GAP-020`: Session Rename & Custom Title Editing
- **Llumen Ref**: `src/lib/components/sidebar/Entry.svelte`
- **Lumis Target**: `ui/src/lib/components/sidebar/SessionRenameDialog.svelte`
- **Status**: **RESOLVED**
- **Traces**: Modal dialog with autofocus input, Save/Cancel, and keyboard Enter shortcut.

#### `UI-GAP-021`: Session Deletion with Confirmation Dialog
- **Llumen Ref**: `src/lib/components/sidebar/Entry.svelte`
- **Lumis Target**: `ui/src/lib/components/sidebar/SessionDeleteDialog.svelte`
- **Status**: **RESOLVED**
- **Traces**: Danger modal preventing accidental loss with Delete/Cancel buttons.

#### `UI-GAP-022`: Mobile Touch Swipe Gestures for Sidebar Drawer
- **Llumen Ref**: `src/lib/components/sidebar/Sidebar.svelte`
- **Lumis Target**: `ui/src/lib/components/sidebar/Sidebar.svelte`
- **Status**: **RESOLVED**
- **Traces**: Touchstart/touchend delta threshold $>50\text{px}$ closing mobile sidebar smoothly.

#### `UI-GAP-023`: Multi-Tab Settings Modal, Themes & Admin Grid
- **Llumen Ref**: `src/lib/components/setting/Setting.svelte` & `tabs/*`
- **Lumis Target**: `ui/src/lib/components/settings/SettingsModal.svelte`, `ThemesTab.svelte`, `UserGrid.svelte`
- **Status**: **RESOLVED**
- **Traces**: Tabs for Account, Themes (Dracula, Llumen, Vitesse, Dark/Light), and Admin management.

---

### [Iteration 1: Deep Research Progress, Message Pagination, Lightbox Zoom]

#### `UI-GAP-024`: Message Branching & Turn Pagination (`< 1/N >`)
- **Llumen Ref**: `src/lib/components/message/Pagination.svelte`
- **Lumis Target**: `ui/src/lib/components/message/MessagePagination.svelte`
- **Status**: **RESOLVED**
- **Traces**: Branch switcher for edited messages or regenerated turns with prev/next arrows and count.

#### `UI-GAP-025`: Deep Research Multi-Step Progress Timeline
- **Llumen Ref**: `src/lib/components/message/DeepResearch.svelte` & `ResearchStep.svelte`
- **Lumis Target**: `ui/src/lib/components/message/DeepResearchProgress.svelte`
- **Status**: **RESOLVED**
- **Traces**: Step-by-step progress list showing search queries, scraping steps, timing, and status badges.

#### `UI-GAP-026`: Modal Lightbox Image / Diagram Zoom (`Zoomable`)
- **Llumen Ref**: `src/lib/ui/Zoomable.svelte`
- **Lumis Target**: `ui/src/lib/components/common/ZoomableModal.svelte`
- **Status**: **RESOLVED**
- **Traces**: Click-to-enlarge modal lightbox with ESC key dismissal, download button, and pan support.

---

### [Iteration 2: Composer Action Menu, File Groups, Convert to File]

#### `UI-GAP-027`: Fast Action Menu Popover (`+` Button)
- **Llumen Ref**: `src/lib/components/input/ActionMenu.svelte`
- **Lumis Target**: `ui/src/lib/components/input/ComposerActionMenu.svelte`
- **Status**: **RESOLVED**
- **Traces**: Popover menu providing quick actions: Upload Document, Convert to File, Insert Preset, Clear Chat.

#### `UI-GAP-028`: Multi-File Attachment Chip Group Preview
- **Llumen Ref**: `src/lib/components/common/FileGroup.svelte`
- **Lumis Target**: `ui/src/lib/components/input/AttachmentFileGroup.svelte`
- **Status**: **RESOLVED**
- **Traces**: Visual card preview for attached files showing file icon, filename, formatted byte size, and remove chip.

#### `UI-GAP-029`: Convert Long Text / Code to Attachment File Modal
- **Llumen Ref**: `src/lib/components/input/ConvertToFileModal.svelte`
- **Lumis Target**: `ui/src/lib/components/input/ConvertToFileModal.svelte`
- **Status**: **RESOLVED**
- **Traces**: Dialog prompting user when text exceeds threshold to convert into `.txt` or `.md` attachment.

---

### [Iteration 3: Provider Key Manager, Confidence Score, Message Minimap]

#### `UI-GAP-030`: Custom LLM Provider Key & Endpoint Manager
- **Llumen Ref**: `src/lib/components/setting/tabs/Openrouter.svelte` & `OpenrouterEdit.svelte`
- **Lumis Target**: `ui/src/lib/components/settings/ProviderKeysTab.svelte`
- **Status**: **RESOLVED**
- **Traces**: Settings tab to configure custom OpenAI/OpenRouter/Local LLM endpoints, custom model IDs, and test connection button.

#### `UI-GAP-031`: Grounded Factuality & Confidence Score Badge
- **Llumen Ref**: `src/lib/components/common/Hallucination.svelte`
- **Lumis Target**: `ui/src/lib/components/message/ConfidenceBadge.svelte`
- **Status**: **RESOLVED**
- **Traces**: Visual confidence badge (High / Medium / Low) with citation grounding ratio and tooltip.

#### `UI-GAP-032`: Message Outline & Heading Minimap Navigator
- **Llumen Ref**: `src/lib/components/message/Minimap.svelte`
- **Lumis Target**: `ui/src/lib/components/message/MessageMinimap.svelte`
- **Status**: **RESOLVED**
- **Traces**: Floating document outline parser extracting markdown headers for 1-click smooth jump navigation.
