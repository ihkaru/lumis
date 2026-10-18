<script lang="ts">
  import { OpenUiParser } from "./parser";
  import { OpenUiEvaluator } from "./evaluator";
  import type { OpenUiContext, UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";
  import Code2 from "lucide-svelte/icons/code-2";
  import Play from "lucide-svelte/icons/play";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import Maximize2 from "lucide-svelte/icons/maximize-2";
  import Minimize2 from "lucide-svelte/icons/minimize-2";
  import Sparkles from "lucide-svelte/icons/sparkles";

  interface Props {
    code: string;
    isStreaming?: boolean;
    sessionId?: string;
  }

  let { code = "", isStreaming = false, sessionId }: Props = $props();

  let activeTab = $state<"preview" | "code">("preview");
  let isCopied = $state(false);
  let isFullscreen = $state(false);

  // Reactive state store for OpenUI bindings
  let runtimeState = $state<Record<string, any>>({});
  let stateVersion = $state(0);

  const parser = new OpenUiParser();
  const evaluator = new OpenUiEvaluator();

  const ctx: OpenUiContext = {
    get state() {
      return runtimeState;
    },
    initialState: {},
    definitions: new Map(),
    onToAssistant: (prompt: string, contextData?: any) => {
      // Dispatch custom DOM event to chat input if available
      if (typeof window !== "undefined") {
        window.dispatchEvent(
          new CustomEvent("lumis:to_assistant", {
            detail: { prompt, context: contextData, sessionId },
          })
        );
      }
    },
    onCopy: async (text: string) => {
      if (typeof navigator !== "undefined" && navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      }
    },
  };

  let renderedNode = $derived.by<UiNode | null>(() => {
    // Depend on code and stateVersion
    void stateVersion;
    if (!code || !code.trim()) return null;
    try {
      const program = parser.parseProgram(code);
      return evaluator.evaluateProgram(program, ctx);
    } catch {
      return null;
    }
  });

  function handleStateChange(key: string, value: any) {
    runtimeState[key] = value;
    stateVersion += 1;
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => (isCopied = false), 2000);
    } catch {}
  }
</script>

<div
  class="my-3 rounded-xl border border-border/70 bg-card/60 backdrop-blur-xs overflow-hidden shadow-xs transition-all {isFullscreen
    ? 'fixed inset-4 z-50 flex flex-col bg-card/95 shadow-2xl border-primary/30'
    : ''}"
>
  <!-- Header Bar -->
  <div class="flex items-center justify-between border-b border-border/60 bg-muted/30 px-3.5 py-2 text-xs">
    <div class="flex items-center gap-2">
      <div class="flex items-center gap-1.5 text-primary font-medium">
        <Sparkles class="w-3.5 h-3.5 animate-pulse" />
        <span class="tracking-tight">Interactive UI</span>
      </div>
      {#if isStreaming}
        <span class="inline-flex items-center gap-1 rounded-full bg-primary/15 px-2 py-0.5 text-[10px] text-primary">
          <span class="h-1.5 w-1.5 rounded-full bg-primary animate-ping"></span>
          Streaming...
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-1">
      <div class="flex rounded-lg border border-border/60 bg-muted/40 p-0.5 mr-1">
        <button
          type="button"
          onclick={() => (activeTab = "preview")}
          class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-colors {activeTab === 'preview'
            ? 'bg-background text-foreground shadow-xs'
            : 'text-muted-foreground hover:text-foreground'}"
        >
          <Play class="w-3 h-3" />
          <span>Interactive</span>
        </button>
        <button
          type="button"
          onclick={() => (activeTab = "code")}
          class="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium transition-colors {activeTab === 'code'
            ? 'bg-background text-foreground shadow-xs'
            : 'text-muted-foreground hover:text-foreground'}"
        >
          <Code2 class="w-3 h-3" />
          <span>DSL Code</span>
        </button>
      </div>

      <button
        type="button"
        onclick={handleCopy}
        class="inline-flex h-7 w-7 items-center justify-center rounded-md border border-border/60 bg-background/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        title="Copy DSL Code"
      >
        {#if isCopied}
          <Check class="w-3.5 h-3.5 text-emerald-400" />
        {:else}
          <Copy class="w-3.5 h-3.5" />
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (isFullscreen = !isFullscreen)}
        class="inline-flex h-7 w-7 items-center justify-center rounded-md border border-border/60 bg-background/60 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        title={isFullscreen ? "Exit Fullscreen" : "Expand to Canvas"}
      >
        {#if isFullscreen}
          <Minimize2 class="w-3.5 h-3.5" />
        {:else}
          <Maximize2 class="w-3.5 h-3.5" />
        {/if}
      </button>
    </div>
  </div>

  <!-- Content Body -->
  <div class="p-4 {isFullscreen ? 'flex-1 overflow-y-auto' : ''}">
    {#if activeTab === "preview"}
      {#if renderedNode}
        <OpenUiNode node={renderedNode} state={runtimeState} onStateChange={handleStateChange} />
      {:else}
        <div class="py-6 text-center text-xs text-muted-foreground">
          {#if isStreaming}
            <span>Generating interactive UI tree...</span>
          {:else}
            <span>No interactive elements generated.</span>
          {/if}
        </div>
      {/if}
    {:else}
      <pre class="overflow-x-auto rounded-lg bg-black/60 p-3 text-xs font-mono text-emerald-400/90 leading-relaxed max-h-96"><code>{code}</code></pre>
    {/if}
  </div>
</div>
