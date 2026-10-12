<script lang="ts">
  import Wrench from "lucide-svelte/icons/wrench";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import { Badge } from "$lib/components/ui/badge";

  interface Props {
    toolName: string;
    args?: Record<string, unknown> | string;
    result?: unknown;
    isRunning?: boolean;
  }

  let { toolName = "default_tool", args, result, isRunning = false }: Props = $props();

  let isOpen = $state(false);

  let formattedArgs = $derived(
    typeof args === "string" ? args : JSON.stringify(args || {}, null, 2)
  );

  let formattedResult = $derived(
    typeof result === "string" ? result : JSON.stringify(result || {}, null, 2)
  );
</script>

<div class="my-2.5 overflow-hidden rounded-xl border border-border/80 bg-muted/30 font-sans text-xs">
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex w-full items-center justify-between px-3.5 py-2 text-left hover:bg-muted/50 transition-colors select-none cursor-pointer"
    aria-expanded={isOpen}
  >
    <div class="flex items-center gap-2">
      <Wrench class="h-3.5 w-3.5 text-cyan-400 shrink-0" />
      <span class="text-muted-foreground font-medium">Tool Call:</span>
      <Badge variant="outline" class="font-mono text-[11px] py-0 px-2 text-cyan-400 border-cyan-800/40 bg-cyan-950/40">
        {toolName}
      </Badge>
      {#if isRunning}
        <span class="inline-flex items-center gap-1 text-[10px] text-amber-400 font-medium">
          <Loader2 class="h-3 w-3 animate-spin" />
          <span>Menjalankan...</span>
        </span>
      {:else if result !== undefined}
        <span class="inline-flex items-center gap-1 text-[10px] text-emerald-400 font-medium">
          <CheckCircle2 class="h-3 w-3" />
          <span>Selesai</span>
        </span>
      {/if}
    </div>
    <ChevronDown class="h-4 w-4 text-muted-foreground transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div class="border-t border-border/60 p-3 space-y-2 bg-background/50 font-mono text-[11px]">
      {#if args}
        <div>
          <span class="text-muted-foreground font-sans font-semibold text-[10px] uppercase">Parameter Input:</span>
          <pre class="mt-1 p-2 rounded bg-slate-950 text-slate-200 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere]">{formattedArgs}</pre>
        </div>
      {/if}
      {#if result !== undefined}
        <div>
          <span class="text-muted-foreground font-sans font-semibold text-[10px] uppercase">Hasil Kembalian (Payload):</span>
          <pre class="mt-1 p-2 rounded bg-slate-950 text-emerald-300 overflow-x-auto whitespace-pre-wrap [overflow-wrap:anywhere]">{formattedResult}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
