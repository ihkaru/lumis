<script lang="ts">
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import WrapText from "lucide-svelte/icons/wrap-text";
  import Code2 from "lucide-svelte/icons/code-2";
  import Maximize2 from "lucide-svelte/icons/maximize-2";
  import Minimize2 from "lucide-svelte/icons/minimize-2";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import hljs from "highlight.js/lib/common";

  interface Props {
    code: string;
    lang?: string;
  }

  let { code = "", lang = "text" }: Props = $props();

  let isCopied = $state(false);
  let isWrapped = $state(false);
  let isFullscreen = $state(false);

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

  $effect(() => {
    if (typeof window === "undefined") return;
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape" && isFullscreen) isFullscreen = false;
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
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
        class="h-5 w-5 text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 rounded cursor-pointer"
        onclick={() => (isFullscreen = !isFullscreen)}
        title="Fullscreen Code Block"
        aria-label="Toggle Fullscreen"
      >
        <Maximize2 class="h-3 w-3" />
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

<!-- Fullscreen Code Lightbox Modal -->
{#if isFullscreen}
  <div class="fixed inset-0 z-50 flex flex-col bg-slate-950/95 backdrop-blur-md p-3 sm:p-5 font-mono">
    <div class="flex items-center justify-between border-b border-white/10 pb-2 mb-2 select-none">
      <div class="flex items-center gap-2">
        <Code2 class="h-4 w-4 text-primary" />
        <span class="font-semibold text-sm text-slate-100 uppercase tracking-wide">
          {lang || "Code"} - Fullscreen Viewer
        </span>
      </div>
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2 text-xs text-slate-300 hover:text-white"
          onclick={() => (isWrapped = !isWrapped)}
        >
          <WrapText class="h-3.5 w-3.5 mr-1" />
          {isWrapped ? "No Wrap" : "Wrap"}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2 text-xs text-slate-300 hover:text-primary"
          onclick={handleCopy}
        >
          {#if isCopied}
            <Check class="h-3.5 w-3.5 mr-1 text-emerald-400" />
            <span class="text-emerald-400">Tersalin</span>
          {:else}
            <Copy class="h-3.5 w-3.5 mr-1" />
            <span>Salin</span>
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7 text-slate-300 hover:text-white"
          onclick={() => (isFullscreen = false)}
          title="Tutup (Esc)"
        >
          <X class="h-4 w-4" />
        </Button>
      </div>
    </div>
    <div class="flex-1 w-full rounded-lg border border-white/10 bg-slate-900/60 p-4 overflow-auto shadow-2xl text-xs leading-relaxed {isWrapped ? 'whitespace-pre-wrap [overflow-wrap:anywhere]' : 'whitespace-pre'}">
      <code class="hljs">{@html highlightedHtml}</code>
    </div>
  </div>
{/if}
