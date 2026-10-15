<script lang="ts">
  import Wrench from "lucide-svelte/icons/wrench";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import XCircle from "lucide-svelte/icons/x-circle";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";

  interface Props {
    toolName: string;
    serverName?: string;
    args?: Record<string, unknown> | string;
    result?: unknown;
    error?: string;
    isRunning?: boolean;
  }

  let {
    toolName = "default_tool",
    serverName,
    args,
    result,
    error,
    isRunning = false
  }: Props = $props();

  let isOpen = $state(false);
  let copiedField = $state<"args" | "result" | null>(null);

  let parsedInfo = $derived.by(() => {
    if (serverName) {
      return { server: serverName, tool: toolName };
    }
    if (toolName.includes("::")) {
      const parts = toolName.split("::");
      return { server: parts[0], tool: parts.slice(1).join("::") };
    }
    if (toolName.includes("__")) {
      const parts = toolName.split("__");
      return { server: parts[0], tool: parts.slice(1).join("__") };
    }
    return { server: null, tool: toolName };
  });

  let formattedArgs = $derived(
    typeof args === "string" ? args : JSON.stringify(args || {}, null, 2)
  );

  let formattedResult = $derived(
    typeof result === "string" ? result : JSON.stringify(result || {}, null, 2)
  );

  async function copyToClipboard(text: string, field: "args" | "result") {
    try {
      await navigator.clipboard.writeText(text);
      copiedField = field;
      setTimeout(() => {
        if (copiedField === field) copiedField = null;
      }, 1800);
    } catch {}
  }
</script>

<div class="my-1.5 overflow-hidden rounded-lg border border-border/50 bg-muted/15 font-sans text-xs">
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex w-full items-center justify-between px-3 py-1.5 text-left hover:bg-muted/30 transition-colors select-none cursor-pointer"
    aria-expanded={isOpen}
  >
    <div class="flex items-center gap-2 min-w-0">
      <Wrench class="h-3.5 w-3.5 text-cyan-400 shrink-0" />

      {#if parsedInfo.server}
        <span class="font-mono text-[11px] text-muted-foreground truncate">{parsedInfo.server}/</span>
      {/if}

      <span class="font-mono text-[11px] font-semibold text-foreground truncate">
        {parsedInfo.tool}
      </span>

      {#if isRunning}
        <span class="inline-flex items-center gap-1 text-[11px] text-amber-400 font-medium shrink-0 ml-1">
          <Loader2 class="h-3 w-3 animate-spin" />
          <span>Running...</span>
        </span>
      {:else if error}
        <span class="inline-flex items-center gap-1 text-[11px] text-red-400 font-medium shrink-0 ml-1">
          <XCircle class="h-3 w-3" />
          <span>Failed</span>
        </span>
      {:else if result !== undefined}
        <span class="inline-flex items-center gap-1 text-[11px] text-emerald-400 font-medium shrink-0 ml-1">
          <CheckCircle2 class="h-3 w-3" />
          <span>Completed</span>
        </span>
      {/if}
    </div>

    <ChevronDown class="h-3.5 w-3.5 text-muted-foreground transition-transform duration-150 shrink-0 ml-2 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div class="border-t border-border/40 p-2.5 space-y-2.5 bg-background/50 font-mono text-[11px]">
      {#if args}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground font-sans font-medium text-[10px] uppercase tracking-wider">Input Parameters:</span>
            <button
              type="button"
              onclick={() => copyToClipboard(formattedArgs, "args")}
              class="inline-flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground font-sans transition-colors cursor-pointer py-0.5 px-1 rounded hover:bg-muted/50"
              title="Copy input arguments"
            >
              {#if copiedField === "args"}
                <Check class="h-3 w-3 text-emerald-400" />
                <span class="text-emerald-400">Copied</span>
              {:else}
                <Copy class="h-3 w-3" />
                <span>Copy</span>
              {/if}
            </button>
          </div>
          <pre class="p-2.5 rounded-md bg-slate-950/90 text-slate-200 border border-border/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] max-h-56 overflow-y-auto leading-relaxed">{formattedArgs}</pre>
        </div>
      {/if}

      {#if error}
        <div class="space-y-1">
          <span class="text-red-400 font-sans font-medium text-[10px] uppercase tracking-wider">Error Message:</span>
          <pre class="p-2.5 rounded-md bg-red-950/30 text-red-200 border border-red-800/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] leading-relaxed">{error}</pre>
        </div>
      {/if}

      {#if result !== undefined}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground font-sans font-medium text-[10px] uppercase tracking-wider">Output Result:</span>
            <button
              type="button"
              onclick={() => copyToClipboard(formattedResult, "result")}
              class="inline-flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground font-sans transition-colors cursor-pointer py-0.5 px-1 rounded hover:bg-muted/50"
              title="Copy output result"
            >
              {#if copiedField === "result"}
                <Check class="h-3 w-3 text-emerald-400" />
                <span class="text-emerald-400">Copied</span>
              {:else}
                <Copy class="h-3 w-3" />
                <span>Copy</span>
              {/if}
            </button>
          </div>
          <pre class="p-2.5 rounded-md bg-slate-950/90 text-emerald-300 border border-border/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] max-h-56 overflow-y-auto leading-relaxed">{formattedResult}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
