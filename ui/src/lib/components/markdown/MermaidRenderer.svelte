<script lang="ts">
  import GitGraph from "lucide-svelte/icons/git-graph";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";

  interface Props {
    code: string;
  }

  let { code = "" }: Props = $props();
  let isCopied = $state(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => (isCopied = false), 1800);
    } catch {}
  }
</script>

<div class="my-1.5 overflow-hidden rounded-lg border border-primary/20 bg-muted/20 dark:bg-slate-950 font-sans text-xs shadow-xs">
  <div class="flex items-center justify-between border-b border-border/40 bg-muted/40 px-3 py-1 select-none">
    <div class="flex items-center gap-1.5 text-primary">
      <GitGraph class="h-3 w-3" />
      <span class="font-medium text-[10px] tracking-wider uppercase font-mono">Mermaid Diagram</span>
    </div>
    <button
      type="button"
      onclick={handleCopy}
      class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] text-muted-foreground hover:text-foreground hover:bg-muted/60 transition-colors cursor-pointer"
      title="Salin kode diagram"
      aria-label="Salin kode diagram"
    >
      {#if isCopied}
        <Check class="h-3 w-3 text-emerald-400" />
        <span>Tersalin</span>
      {:else}
        <Copy class="h-3 w-3" />
        <span>Salin</span>
      {/if}
    </button>
  </div>
  <div class="px-3 py-2 bg-card/40 overflow-x-auto text-foreground font-mono text-[11px] whitespace-pre leading-relaxed">
    <code>{code}</code>
  </div>
</div>
