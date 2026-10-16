<script lang="ts">
  import Brain from "lucide-svelte/icons/brain";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";

  interface Props {
    content: string;
    isStreaming?: boolean;
    defaultOpen?: boolean;
  }

  let { content = "", isStreaming = false, defaultOpen = false }: Props = $props();

  // svelte-ignore state_referenced_locally
  let isOpen = $state(defaultOpen);
  let isCopied = $state(false);

  // Auto-open when streaming, close when finished streaming if user hasn't explicitly toggled
  $effect(() => {
    if (isStreaming) {
      isOpen = true;
    }
  });

  async function handleCopy(e: MouseEvent) {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(content);
      isCopied = true;
      setTimeout(() => (isCopied = false), 1800);
    } catch {}
  }

  let wordCount = $derived(content.trim() ? content.trim().split(/\s+/).length : 0);
</script>

<div class="my-1.5 overflow-hidden rounded-lg border border-primary/20 bg-primary/5 font-sans text-xs transition-all">
  <div class="flex w-full items-center justify-between px-3 py-1.5 text-primary hover:bg-primary/10 transition-colors select-none">
    <button
      type="button"
      onclick={() => (isOpen = !isOpen)}
      class="flex items-center gap-1.5 flex-1 text-left cursor-pointer"
      aria-expanded={isOpen}
    >
      <Brain class="h-3.5 w-3.5 text-primary shrink-0" />
      <span class="font-medium text-[11px] tracking-wide">Proses Berpikir</span>
      {#if isStreaming}
        <span class="inline-flex items-center gap-1 text-[10px] text-primary bg-primary/15 px-1.5 py-0.2 rounded-full animate-pulse">
          <Sparkles class="h-2.5 w-2.5" />
          <span>Berpikir...</span>
        </span>
      {:else}
        <span class="text-[10px] text-primary/70 font-mono">
          ({wordCount} kata)
        </span>
      {/if}
    </button>
    <div class="flex items-center gap-1.5">
      {#if content.trim()}
        <button
          type="button"
          onclick={handleCopy}
          class="p-0.5 rounded hover:bg-primary/20 text-primary/80 hover:text-primary transition-colors cursor-pointer"
          title="Salin proses berpikir"
          aria-label="Salin proses berpikir"
        >
          {#if isCopied}
            <Check class="h-3 w-3 text-emerald-400" />
          {:else}
            <Copy class="h-3 w-3" />
          {/if}
        </button>
      {/if}
      <button
        type="button"
        onclick={() => (isOpen = !isOpen)}
        class="p-0.5 rounded hover:bg-primary/20 text-primary/80 hover:text-primary transition-colors cursor-pointer"
        aria-label="Buka tutup proses berpikir"
      >
        <ChevronDown class="h-3.5 w-3.5 text-primary transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
      </button>
    </div>
  </div>

  {#if isOpen}
    <div class="border-t border-primary/15 px-3 py-2 text-muted-foreground whitespace-pre-wrap leading-relaxed font-mono text-[11px] [overflow-wrap:anywhere] bg-background/40">
      {content}
    </div>
  {/if}
</div>
