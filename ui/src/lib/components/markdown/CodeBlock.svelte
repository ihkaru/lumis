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

<div class="my-3 overflow-hidden rounded-xl border border-border bg-slate-950/90 text-slate-100 font-mono text-xs shadow-md">
  <!-- Code Block Header -->
  <div class="flex items-center justify-between border-b border-border/60 bg-slate-900/80 px-3.5 py-1.5 select-none">
    <div class="flex items-center gap-2 text-slate-400">
      <Code2 class="h-3.5 w-3.5 text-cyan-400" />
      <span class="text-[11px] font-semibold uppercase tracking-wider text-slate-300">
        {lang || "plain"}
      </span>
    </div>
    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 text-slate-400 hover:text-slate-100 hover:bg-slate-800"
        onclick={() => (isWrapped = !isWrapped)}
        title="Toggle Word Wrap"
        aria-label="Toggle Word Wrap"
      >
        <WrapText class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 text-slate-400 hover:text-cyan-400 hover:bg-slate-800"
        onclick={handleCopy}
        title="Copy code to clipboard"
        aria-label="Copy code to clipboard"
      >
        {#if isCopied}
          <Check class="h-3.5 w-3.5 text-emerald-400" />
        {:else}
          <Copy class="h-3.5 w-3.5" />
        {/if}
      </Button>
    </div>
  </div>

  <!-- Code Content -->
  <div class="overflow-x-auto p-4 leading-relaxed font-mono {isWrapped ? 'whitespace-pre-wrap [overflow-wrap:anywhere]' : 'whitespace-pre'}">
    <code>{code}</code>
  </div>
</div>
