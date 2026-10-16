<script lang="ts">
  import Play from "lucide-svelte/icons/play";
  import Code2 from "lucide-svelte/icons/code-2";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import Maximize2 from "lucide-svelte/icons/maximize-2";
  import Minimize2 from "lucide-svelte/icons/minimize-2";
  import X from "lucide-svelte/icons/x";
  import hljs from "highlight.js/lib/common";

  interface Props {
    code: string;
    lang?: string;
  }

  let { code = "", lang = "html:preview" }: Props = $props();

  let activeTab = $state<"preview" | "code">("preview");
  let isCopied = $state(false);
  let isFullscreen = $state(false);
  let iframeKey = $state(0);
  let containerHeight = $state(380);

  const bridgeScript = `\n<` + `script>\n  (function() {\n    window.onerror = function(msg, url, line) {\n      console.warn("AI HTML App Error:", msg, "at line", line);\n      return false;\n    };\n    try {\n      var ro = new ResizeObserver(function(entries) {\n        for (var i = 0; i < entries.length; i++) {\n          var h = Math.ceil(entries[i].target.scrollHeight || entries[i].contentRect.height);\n          if (h > 60) {\n            window.parent.postMessage({ type: 'lumis:preview_height', height: h }, '*');\n          }\n        }\n      });\n      if (document.body) ro.observe(document.body);\n    } catch (e) {}\n  })();\n</` + `script>\n`;

  let srcDocHtml = $derived.by(() => {
    let clean = code.trim();
    if (!clean.includes("<html") && !clean.includes("<!DOCTYPE") && !clean.includes("<body")) {
      clean = `<!DOCTYPE html><html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><script src="https://cdn.tailwindcss.com"><\/script><style>body { margin: 0; padding: 1rem; font-family: system-ui, -apple-system, sans-serif; color: #f8fafc; background: #090d16; }</style></head><body>${clean}</body></html>`;
    }
    // Inject bridge script before </head> or at start
    if (clean.includes("</head>")) {
      return clean.replace("</head>", `${bridgeScript}</head>`);
    }
    return `${bridgeScript}${clean}`;
  });

  let highlightedCode = $derived.by(() => {
    try {
      return hljs.highlight(code, { language: "html", ignoreIllegals: true }).value;
    } catch {
      return code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }
  });

  $effect(() => {
    if (typeof window === "undefined") return;
    function handleMessage(event: MessageEvent) {
      if (event.data && event.data.type === "lumis:preview_height" && typeof event.data.height === "number") {
        const h = Math.min(Math.max(event.data.height + 24, 220), 720);
        containerHeight = h;
      }
    }
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  });

  $effect(() => {
    if (typeof window === "undefined") return;
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape" && isFullscreen) isFullscreen = false;
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  });

  function reloadIframe() {
    iframeKey += 1;
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      isCopied = true;
      setTimeout(() => (isCopied = false), 1800);
    } catch {}
  }
</script>

<!-- Embedded Card Container -->
<div class="my-2 overflow-hidden rounded-md border border-border/30 dark:border-white/10 bg-slate-950 font-sans text-xs shadow-xs">
  <!-- Toolbar Header -->
  <div class="flex items-center justify-between border-b border-border/20 bg-slate-900/70 px-2.5 py-1 select-none">
    <!-- Left: App Type Badge & Tab Switcher -->
    <div class="flex items-center gap-1.5">
      <span class="inline-flex items-center gap-1 text-primary font-mono text-[10px] font-semibold tracking-wider uppercase">
        <Play class="h-3 w-3 fill-primary/30" />
        Interactive App
      </span>
      <div class="flex items-center rounded bg-slate-800/80 p-0.5 text-[10px] ml-1">
        <button
          type="button"
          onclick={() => (activeTab = "preview")}
          class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors {activeTab === 'preview' ? 'bg-primary text-primary-foreground font-medium' : 'text-slate-400 hover:text-slate-200'}"
        >
          Preview
        </button>
        <button
          type="button"
          onclick={() => (activeTab = "code")}
          class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors {activeTab === 'code' ? 'bg-primary text-primary-foreground font-medium' : 'text-slate-400 hover:text-slate-200'}"
        >
          Code
        </button>
      </div>
    </div>

    <!-- Right Controls -->
    <div class="flex items-center gap-0.5 text-slate-400">
      {#if activeTab === "preview"}
        <button
          type="button"
          onclick={reloadIframe}
          class="p-1 rounded hover:bg-slate-800 hover:text-slate-100 transition-colors"
          title="Reload Preview"
          aria-label="Reload Preview"
        >
          <RotateCcw class="h-3 w-3" />
        </button>
      {/if}
      <button
        type="button"
        onclick={() => (isFullscreen = !isFullscreen)}
        class="p-1 rounded hover:bg-slate-800 hover:text-slate-100 transition-colors"
        title={isFullscreen ? "Exit Fullscreen" : "Fullscreen Preview"}
        aria-label="Toggle Fullscreen"
      >
        {#if isFullscreen}
          <Minimize2 class="h-3 w-3" />
        {:else}
          <Maximize2 class="h-3 w-3" />
        {/if}
      </button>
      <button
        type="button"
        onclick={handleCopy}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] hover:bg-slate-800 hover:text-primary transition-colors"
        title="Salin kode"
        aria-label="Salin kode"
      >
        {#if isCopied}
          <Check class="h-3 w-3 text-emerald-400" />
          <span class="text-emerald-400">Tersalin</span>
        {:else}
          <Copy class="h-3 w-3" />
          <span>Salin</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Body Content -->
  {#if activeTab === "preview"}
    <div class="w-full bg-slate-950/60 overflow-hidden" style="height: {containerHeight}px;">
      {#key iframeKey}
        <iframe
          title="AI Interactive Preview"
          srcdoc={srcDocHtml}
          sandbox="allow-scripts allow-forms"
          class="w-full h-full border-none bg-transparent"
        ></iframe>
      {/key}
    </div>
  {:else}
    <div class="max-h-[460px] overflow-auto px-3 py-2 bg-slate-950 font-mono text-[11px] leading-relaxed text-slate-200">
      <code class="hljs">{@html highlightedCode}</code>
    </div>
  {/if}
</div>

<!-- Fullscreen Modal Lightbox -->
{#if isFullscreen}
  <div class="fixed inset-0 z-50 flex flex-col bg-slate-950/95 backdrop-blur-md p-3 sm:p-5">
    <div class="flex items-center justify-between border-b border-white/10 pb-2 mb-2 select-none">
      <div class="flex items-center gap-2">
        <Play class="h-4 w-4 text-primary fill-primary/30" />
        <span class="font-semibold text-sm text-slate-100">AI Interactive App - Fullscreen Mode</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          type="button"
          onclick={reloadIframe}
          class="p-1.5 rounded hover:bg-white/10 text-slate-300 hover:text-white transition-colors"
          title="Reload"
        >
          <RotateCcw class="h-4 w-4" />
        </button>
        <button
          type="button"
          onclick={() => (isFullscreen = false)}
          class="p-1.5 rounded hover:bg-white/10 text-slate-300 hover:text-white transition-colors"
          title="Close Fullscreen (Esc)"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>
    <div class="flex-1 w-full rounded-lg border border-white/10 bg-slate-900/60 overflow-hidden shadow-2xl">
      {#key iframeKey}
        <iframe
          title="AI Interactive Preview (Fullscreen)"
          srcdoc={srcDocHtml}
          sandbox="allow-scripts allow-forms"
          class="w-full h-full border-none bg-transparent"
        ></iframe>
      {/key}
    </div>
  </div>
{/if}
