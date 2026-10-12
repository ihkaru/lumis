<script lang="ts">
  import Brain from "lucide-svelte/icons/brain";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import Sparkles from "lucide-svelte/icons/sparkles";

  interface Props {
    content: string;
    isStreaming?: boolean;
    defaultOpen?: boolean;
  }

  let { content = "", isStreaming = false, defaultOpen = false }: Props = $props();

  // svelte-ignore state_referenced_locally
  let isOpen = $state(defaultOpen);

  // Auto-open when streaming, close when finished streaming if user hasn't explicitly toggled
  $effect(() => {
    if (isStreaming) {
      isOpen = true;
    }
  });

  let wordCount = $derived(content.trim() ? content.trim().split(/\s+/).length : 0);
</script>

<div class="my-3 overflow-hidden rounded-xl border border-cyan-900/40 bg-cyan-950/20 font-sans text-xs transition-all">
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex w-full items-center justify-between px-3.5 py-2.5 text-left text-cyan-300 hover:bg-cyan-900/30 transition-colors select-none cursor-pointer"
    aria-expanded={isOpen}
  >
    <div class="flex items-center gap-2">
      <Brain class="h-4 w-4 text-cyan-400 shrink-0" />
      <span class="font-semibold tracking-wide">Proses Berpikir (Reasoning)</span>
      {#if isStreaming}
        <span class="inline-flex items-center gap-1 text-[10px] text-cyan-400 bg-cyan-900/50 px-2 py-0.5 rounded-full animate-pulse">
          <Sparkles class="h-3 w-3" />
          <span>Berpikir...</span>
        </span>
      {:else}
        <span class="text-[11px] text-cyan-500/80 font-mono">
          ({wordCount} kata)
        </span>
      {/if}
    </div>
    <ChevronDown class="h-4 w-4 text-cyan-400 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div class="border-t border-cyan-900/30 px-4 py-3 text-muted-foreground whitespace-pre-wrap leading-relaxed font-mono text-[11px] [overflow-wrap:anywhere] bg-background/40">
      {content}
    </div>
  {/if}
</div>
