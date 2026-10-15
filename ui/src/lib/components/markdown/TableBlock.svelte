<script lang="ts">
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";

  interface Props {
    html: string;
  }

  let { html }: Props = $props();

  let zoomLevel = $state(100);

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
</script>

<div class="relative w-full my-3 group">
  <!-- Minimalist Floating Zoom Pill (No outer card encapsulation) -->
  <div
    class="absolute top-1 right-2 z-10 flex items-center gap-0.5 rounded-full bg-background/85 dark:bg-card/90 backdrop-blur-md border border-border/80 shadow-xs px-1.5 py-0.5 text-xs select-none opacity-60 hover:opacity-100 group-hover:opacity-100 transition-opacity pointer-events-auto"
  >
    <button
      type="button"
      onclick={zoomOut}
      disabled={zoomLevel <= 30}
      class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground disabled:opacity-30 transition-colors touch-manipulation"
      title="Zoom Out Table (-15%)"
      aria-label="Zoom Out Table"
    >
      <ZoomOut class="size-3" />
    </button>

    <button
      type="button"
      onclick={resetZoom}
      class="px-1.5 py-0.5 rounded-full hover:bg-muted text-[11px] font-mono text-muted-foreground hover:text-foreground transition-colors touch-manipulation"
      title="Reset Zoom (100%)"
      aria-label="Reset Table Zoom to 100%"
    >
      {zoomLevel}%
    </button>

    <button
      type="button"
      onclick={zoomIn}
      disabled={zoomLevel >= 150}
      class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground disabled:opacity-30 transition-colors touch-manipulation"
      title="Zoom In Table (+15%)"
      aria-label="Zoom In Table"
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

  <!-- Horizontal Scroll Viewport -->
  <div class="w-full overflow-x-auto smooth-momentum-scroll">
    <div
      class="inline-block min-w-full align-middle transition-all duration-200"
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
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }
  :global(.table-content th),
  :global(.table-content td) {
    border: 1px solid var(--border);
    padding: 0.5rem 0.85rem;
    text-align: left;
    white-space: normal;
    min-width: 100px;
  }
  :global(.table-content th) {
    background-color: rgba(255, 255, 255, 0.05);
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
