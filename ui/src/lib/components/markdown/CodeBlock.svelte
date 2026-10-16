<script lang="ts">
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import WrapText from "lucide-svelte/icons/wrap-text";
  import Code2 from "lucide-svelte/icons/code-2";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    code: string;
    lang?: string;
  }

  let { code = "", lang = "text" }: Props = $props();

  let isCopied = $state(false);
  let isWrapped = $state(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch (e) {
      console.error("Clipboard copy failed:", e);
    }
  }
</script>

<div class="my-1.5 overflow-hidden rounded-lg border border-border/80 bg-slate-950/95 text-slate-100 font-mono text-xs shadow-xs">
  <!-- Code Block Header -->
  <div class="flex items-center justify-between border-b border-border/50 bg-slate-900/90 px-3 py-1 select-none">
    <div class="flex items-center gap-1.5 text-slate-400">
      <Code2 class="h-3 w-3 text-primary" />
      <span class="text-[10px] font-semibold uppercase tracking-wider text-slate-300">
        {lang || "plain"}
      </span>
    </div>
    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 text-slate-400 hover:text-slate-100 hover:bg-slate-800 rounded cursor-pointer"
        onclick={() => (isWrapped = !isWrapped)}
        title="Toggle Word Wrap"
        aria-label="Toggle Word Wrap"
      >
        <WrapText class="h-3 w-3" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 text-slate-400 hover:text-primary hover:bg-slate-800 rounded cursor-pointer"
        onclick={handleCopy}
        title="Salin kode"
        aria-label="Salin kode"
      >
        {#if isCopied}
          <Check class="h-3 w-3 text-emerald-400" />
        {:else}
          <Copy class="h-3 w-3" />
        {/if}
      </Button>
    </div>
  </div>

  <!-- Code Content -->
  <div class="overflow-x-auto px-3 py-2 text-xs leading-relaxed font-mono {isWrapped ? 'whitespace-pre-wrap [overflow-wrap:anywhere]' : 'whitespace-pre'}">
    <code>{code}</code>
  </div>
</div>
