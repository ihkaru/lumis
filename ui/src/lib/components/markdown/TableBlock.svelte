<script lang="ts">
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Table from "lucide-svelte/icons/table";

  interface Props {
    html: string;
  }

  let { html }: Props = $props();

  let zoomLevel = $state(100);

  function zoomIn() {
    if (zoomLevel < 160) zoomLevel += 15;
  }

  function zoomOut() {
    if (zoomLevel > 70) zoomLevel -= 15;
  }

  function resetZoom() {
    zoomLevel = 100;
  }
</script>

<div class="my-3.5 rounded-xl border border-border/80 bg-card/60 backdrop-blur-xs overflow-hidden shadow-xs">
  <div class="flex items-center justify-between px-3 py-1.5 border-b border-border/60 bg-muted/40 text-xs text-muted-foreground select-none">
    <div class="flex items-center space-x-1.5">
      <Table class="size-3.5 text-primary/80" />
      <span class="font-medium text-[11px] tracking-wider uppercase opacity-80">Table</span>
    </div>
    <div class="flex items-center space-x-1">
      <button
        type="button"
        onclick={zoomOut}
        disabled={zoomLevel <= 70}
        class="p-1 rounded-md hover:bg-background/80 hover:text-foreground text-muted-foreground disabled:opacity-30 transition-colors touch-manipulation"
        title="Zoom Out Table (-15%)"
        aria-label="Zoom Out Table"
      >
        <ZoomOut class="size-3.5" />
      </button>

      <button
        type="button"
        onclick={resetZoom}
        class="px-1.5 py-0.5 rounded-md hover:bg-background/80 hover:text-foreground text-[11px] font-mono transition-colors touch-manipulation"
        title="Reset Zoom (100%)"
        aria-label="Reset Table Zoom to 100%"
      >
        {zoomLevel}%
      </button>

      <button
        type="button"
        onclick={zoomIn}
        disabled={zoomLevel >= 160}
        class="p-1 rounded-md hover:bg-background/80 hover:text-foreground text-muted-foreground disabled:opacity-30 transition-colors touch-manipulation"
        title="Zoom In Table (+15%)"
        aria-label="Zoom In Table"
      >
        <ZoomIn class="size-3.5" />
      </button>

      {#if zoomLevel !== 100}
        <button
          type="button"
          onclick={resetZoom}
          class="p-1 ml-0.5 rounded-md hover:bg-background/80 hover:text-foreground text-muted-foreground transition-colors touch-manipulation"
          title="Reset"
          aria-label="Reset Zoom"
        >
          <RotateCcw class="size-3" />
        </button>
      {/if}
    </div>
  </div>

  <div class="w-full overflow-x-auto smooth-momentum-scroll">
    <div
      class="inline-block min-w-full align-middle p-2.5 transition-all duration-200"
      style="font-size: {(zoomLevel / 100) * 0.8125}rem"
    >
      <div class="table-content min-w-full">
        {@html html}
      </div>
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
    margin: 0;
  }
  :global(.table-content th),
  :global(.table-content td) {
    border: 1px solid var(--border);
    padding: 0.5rem 0.85rem;
    text-align: left;
    white-space: normal;
    min-width: 110px;
  }
  :global(.table-content th) {
    background-color: rgba(255, 255, 255, 0.06);
    font-weight: 600;
    color: var(--foreground, #ffffff);
  }
  :global(.table-content tr:nth-child(even)) {
    background-color: rgba(255, 255, 255, 0.02);
  }
  :global(.table-content tr:hover) {
    background-color: rgba(255, 255, 255, 0.04);
  }
</style>
