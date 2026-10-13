<script lang="ts">
  import Wrench from "lucide-svelte/icons/wrench";
  import Server from "lucide-svelte/icons/server";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import XCircle from "lucide-svelte/icons/x-circle";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import { Badge } from "$lib/components/ui/badge";

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

  // Extract server name from tool prefix if formatted as server::tool or server__tool
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
    } catch {
      // Clipboard fallback
    }
  }
</script>

<div class="my-2 overflow-hidden rounded-xl border border-border/70 bg-muted/20 font-sans text-xs">
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex w-full items-center justify-between px-3.5 py-2.5 text-left hover:bg-muted/40 transition-colors select-none cursor-pointer"
    aria-expanded={isOpen}
  >
    <div class="flex items-center gap-2 flex-wrap min-w-0">
      <div class="p-1 rounded-md bg-cyan-950/50 border border-cyan-800/40 text-cyan-400 shrink-0">
        <Wrench class="h-3.5 w-3.5" />
      </div>

      <span class="text-muted-foreground font-medium text-[11px]">Tool:</span>

      {#if parsedInfo.server}
        <Badge variant="outline" class="font-mono text-[10px] py-0 px-1.5 text-muted-foreground border-border/60 bg-muted/40 flex items-center gap-1">
          <Server class="h-2.5 w-2.5" />
          <span>{parsedInfo.server}</span>
        </Badge>
      {/if}

      <Badge variant="outline" class="font-mono text-[11px] py-0 px-2 text-cyan-400 border-cyan-800/40 bg-cyan-950/40 font-semibold truncate">
        {parsedInfo.tool}
      </Badge>

      {#if isRunning}
        <span class="inline-flex items-center gap-1 text-[10px] text-amber-400 font-medium">
          <Loader2 class="h-3 w-3 animate-spin" />
          <span>Menjalankan...</span>
        </span>
      {:else if error}
        <span class="inline-flex items-center gap-1 text-[10px] text-red-400 font-medium">
          <XCircle class="h-3 w-3" />
          <span>Gagal</span>
        </span>
      {:else if result !== undefined}
        <span class="inline-flex items-center gap-1 text-[10px] text-emerald-400 font-medium">
          <CheckCircle2 class="h-3 w-3" />
          <span>Selesai</span>
        </span>
      {/if}
    </div>

    <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform duration-200 shrink-0 ml-2 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div class="border-t border-border/50 p-3 space-y-3 bg-background/40 font-mono text-[11px]">
      {#if args}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground font-sans font-semibold text-[10px] uppercase tracking-wider">Parameter Input:</span>
            <button
              type="button"
              onclick={() => copyToClipboard(formattedArgs, "args")}
              class="inline-flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground font-sans transition-colors cursor-pointer py-0.5 px-1 rounded hover:bg-muted/50"
              title="Salin parameter"
            >
              {#if copiedField === "args"}
                <Check class="h-3 w-3 text-emerald-400" />
                <span class="text-emerald-400">Tersalin</span>
              {:else}
                <Copy class="h-3 w-3" />
                <span>Salin</span>
              {/if}
            </button>
          </div>
          <pre class="p-2.5 rounded-lg bg-slate-950 text-slate-200 border border-border/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] max-h-60 overflow-y-auto leading-relaxed">{formattedArgs}</pre>
        </div>
      {/if}

      {#if error}
        <div class="space-y-1">
          <span class="text-red-400 font-sans font-semibold text-[10px] uppercase tracking-wider">Pesan Kesalahan (Error):</span>
          <pre class="p-2.5 rounded-lg bg-red-950/40 text-red-200 border border-red-800/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] leading-relaxed">{error}</pre>
        </div>
      {/if}

      {#if result !== undefined}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground font-sans font-semibold text-[10px] uppercase tracking-wider">Hasil Kembalian (Payload):</span>
            <button
              type="button"
              onclick={() => copyToClipboard(formattedResult, "result")}
              class="inline-flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground font-sans transition-colors cursor-pointer py-0.5 px-1 rounded hover:bg-muted/50"
              title="Salin hasil kembalian"
            >
              {#if copiedField === "result"}
                <Check class="h-3 w-3 text-emerald-400" />
                <span class="text-emerald-400">Tersalin</span>
              {:else}
                <Copy class="h-3 w-3" />
                <span>Salin</span>
              {/if}
            </button>
          </div>
          <pre class="p-2.5 rounded-lg bg-slate-950 text-emerald-300 border border-border/40 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere] max-h-60 overflow-y-auto leading-relaxed">{formattedResult}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
