<script lang="ts">
  import DOMPurify from "dompurify";
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";

  interface Props {
    html: string;
  }

  let { html }: Props = $props();

  let zoomLevel = $state(100);
  let isCopied = $state(false);

  function zoomIn() {
    if (zoomLevel < 150) {
      zoomLevel = Math.min(150, zoomLevel + 15);
    }
  }

  function zoomOut() {
    if (zoomLevel > 30) {
      zoomLevel = Math.max(30, zoomLevel - 15);
    }
  }

  function resetZoom() {
    zoomLevel = 100;
  }

  async function copyTableToClipboard() {
    if (typeof window === "undefined") return;
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, "text/html");
      const table = doc.querySelector("table");
      if (!table) return;

      const rows: string[] = [];
      table.querySelectorAll("tr").forEach((tr) => {
        const cells: string[] = [];
        tr.querySelectorAll("th, td").forEach((cell) => {
          const text = (cell.textContent || "").trim().replace(/\r?\n/g, " ").replace(/\t/g, " ");
          cells.push(text);
        });
        if (cells.length > 0) {
          rows.push(cells.join("\t"));
        }
      });
      const tsvData = rows.join("\n");
      const htmlData = table.outerHTML;

      if (navigator.clipboard && window.ClipboardItem) {
        const textBlob = new Blob([tsvData], { type: "text/plain" });
        const htmlBlob = new Blob([htmlData], { type: "text/html" });
        await navigator.clipboard.write([
          new ClipboardItem({
            "text/plain": textBlob,
            "text/html": htmlBlob,
          }),
        ]).catch(() => navigator.clipboard.writeText(tsvData));
      } else {
        await navigator.clipboard.writeText(tsvData);
      }

      isCopied = true;
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } catch {
      // Graceful fallback
    }
  }
</script>

<div class="relative w-full my-1.5">
  <!-- Horizontal Scroll Viewport -->
  <div class="w-full overflow-x-auto smooth-momentum-scroll">
    <div
      class="inline-block min-w-full align-middle transition-all duration-200"
      style="font-size: {(zoomLevel / 100) * 0.8125}rem"
    >
      <div class="table-content min-w-full">
        {@html typeof window !== "undefined"
          ? DOMPurify.sanitize(html, {
              USE_PROFILES: { html: true, mathMl: true, svg: true },
              ADD_TAGS: ["annotation", "semantics", "span", "div"],
              ADD_ATTR: ["align", "style", "aria-hidden", "viewBox", "class"],
            })
          : html}
      </div>
    </div>
  </div>

  <!-- Bottom-Right Controls Bar (Outside Table, Below Scrollbar X) -->
  <div class="flex justify-end mt-1">
    <div
      class="inline-flex items-center gap-0.5 rounded-full bg-background/85 dark:bg-card/90 backdrop-blur-md border border-border/30 dark:border-white/10 shadow-xs px-1.5 py-0.5 text-xs select-none opacity-70 hover:opacity-100 transition-opacity"
    >
      <!-- 1-Click Copy for Excel / Google Sheets -->
      <button
        type="button"
        onclick={copyTableToClipboard}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground transition-colors touch-manipulation text-[11px]"
        title="Salin tabel untuk Excel / Google Sheets"
        aria-label="Salin tabel ke Excel"
      >
        {#if isCopied}
          <Check class="size-3 text-emerald-400" />
          <span class="text-emerald-400 font-medium">Tersalin</span>
        {:else}
          <Copy class="size-3" />
          <span>Salin</span>
        {/if}
      </button>

      <div class="h-3 w-px bg-border/60 mx-0.5" aria-hidden="true"></div>

      <button
        type="button"
        onclick={zoomOut}
        disabled={zoomLevel <= 30}
        class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground disabled:opacity-30 transition-colors touch-manipulation"
        title="Zoom Out Tabel"
        aria-label="Zoom Out Tabel"
      >
        <ZoomOut class="size-3" />
      </button>

      <button
        type="button"
        onclick={zoomIn}
        disabled={zoomLevel >= 150}
        class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground disabled:opacity-30 transition-colors touch-manipulation"
        title="Zoom In Tabel"
        aria-label="Zoom In Tabel"
      >
        <ZoomIn class="size-3" />
      </button>

      {#if zoomLevel !== 100}
        <button
          type="button"
          onclick={resetZoom}
          class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground transition-colors touch-manipulation"
          title="Reset Zoom"
          aria-label="Reset Zoom"
        >
          <RotateCcw class="size-2.5" />
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  :global(.table-content table) {
    width: max-content;
    min-width: 100%;
    border-collapse: collapse;
    font-size: inherit;
    line-height: 1.5;
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }
  :global(.table-content th[align="center"]),
  :global(.table-content td[align="center"]) {
    text-align: center !important;
  }
  :global(.table-content th[align="right"]),
  :global(.table-content td[align="right"]) {
    text-align: right !important;
  }
  :global(.table-content th[align="left"]),
  :global(.table-content td[align="left"]) {
    text-align: left !important;
  }
  :global(.table-content th),
  :global(.table-content td) {
    border: 1px solid hsl(var(--border) / 0.8);
    padding: 0.35rem 0.65rem;
    text-align: left;
    white-space: normal;
    min-width: 80px;
  }
  :global(.table-content th) {
    background-color: hsl(var(--muted) / 0.7);
    font-weight: 600;
    color: var(--foreground);
  }
  :global(.table-content tr:nth-child(even)) {
    background-color: hsl(var(--muted) / 0.25);
  }
  :global(.table-content tr:hover) {
    background-color: hsl(var(--muted) / 0.45);
  }
</style>
