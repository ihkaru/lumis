<script lang="ts">
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import WrapText from "lucide-svelte/icons/wrap-text";
  import Code2 from "lucide-svelte/icons/code-2";
  import { Button } from "$lib/components/ui/button";
  import hljs from "highlight.js/lib/common";

  interface Props {
    code: string;
    lang?: string;
  }

  let { code = "", lang = "text" }: Props = $props();

  let isCopied = $state(false);
  let isWrapped = $state(false);

  let highlightedHtml = $derived.by(() => {
    if (!code) return "";
    const cleanLang = (lang || "").trim().toLowerCase();
    try {
      if (cleanLang && hljs.getLanguage(cleanLang)) {
        return hljs.highlight(code, { language: cleanLang, ignoreIllegals: true }).value;
      }
      return hljs.highlightAuto(code).value;
    } catch {
      return code
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
    }
  });

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

<div class="my-1 overflow-hidden rounded-md border border-border/30 dark:border-white/10 bg-slate-950 text-slate-100 font-mono text-xs shadow-xs">
  <!-- Code Block Header -->
  <div class="flex items-center justify-between border-b border-border/20 bg-slate-900/60 px-2.5 py-0.5 select-none">
    <div class="flex items-center gap-1.5 text-slate-400">
      <Code2 class="h-3 w-3 text-primary/90" />
      <span class="text-[10px] font-medium uppercase tracking-wider text-slate-400">
        {lang || "plain"}
      </span>
    </div>
    <div class="flex items-center gap-0.5">
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 rounded cursor-pointer"
        onclick={() => (isWrapped = !isWrapped)}
        title="Toggle Word Wrap"
        aria-label="Toggle Word Wrap"
      >
        <WrapText class="h-3 w-3" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 text-slate-400 hover:text-primary hover:bg-slate-800/80 rounded cursor-pointer"
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
  <div class="overflow-x-auto px-2.5 py-1.5 text-xs leading-relaxed font-mono {isWrapped ? 'whitespace-pre-wrap [overflow-wrap:anywhere]' : 'whitespace-pre'}">
    <code class="hljs">{@html highlightedHtml}</code>
  </div>
</div>
