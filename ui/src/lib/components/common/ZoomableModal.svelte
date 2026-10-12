<script lang="ts">
  import X from "lucide-svelte/icons/x";
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Download from "lucide-svelte/icons/download";

  interface Props {
    src: string;
    alt?: string;
    open: boolean;
    onClose?: () => void;
  }

  let { src, alt = "Preview", open = false, onClose }: Props = $props();

  let zoomLevel = $state(1);

  function handleZoomIn() {
    zoomLevel = Math.min(zoomLevel + 0.25, 3);
  }

  function handleZoomOut() {
    zoomLevel = Math.max(zoomLevel - 0.25, 0.5);
  }

  function handleReset() {
    zoomLevel = 1;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      onClose?.();
    }
  }

  function handleDownload() {
    const link = document.createElement("a");
    link.href = src;
    link.download = alt || "image-preview";
    link.click();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm transition-opacity duration-200"
    role="dialog"
    aria-modal="true"
    aria-label="Image Zoom Lightbox"
  >
    <!-- Backdrop dismiss -->
    <button
      type="button"
      class="absolute inset-0 w-full h-full cursor-zoom-out border-0 bg-transparent p-0"
      onclick={() => onClose?.()}
      aria-label="Close lightbox"
    ></button>

    <!-- Controls toolbar -->
    <div class="absolute top-4 right-4 z-10 flex items-center gap-1.5 p-1 rounded-xl bg-background/90 border border-border shadow-lg">
      <button
        type="button"
        class="p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
        onclick={handleZoomIn}
        aria-label="Zoom in"
        title="Zoom In"
      >
        <ZoomIn class="w-4 h-4" />
      </button>

      <button
        type="button"
        class="p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
        onclick={handleZoomOut}
        aria-label="Zoom out"
        title="Zoom Out"
      >
        <ZoomOut class="w-4 h-4" />
      </button>

      <button
        type="button"
        class="p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
        onclick={handleReset}
        aria-label="Reset zoom"
        title="Reset Zoom"
      >
        <RotateCcw class="w-4 h-4" />
      </button>

      <button
        type="button"
        class="p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer"
        onclick={handleDownload}
        aria-label="Download image"
        title="Unduh Berkas"
      >
        <Download class="w-4 h-4" />
      </button>

      <div class="h-4 w-px bg-border mx-1"></div>

      <button
        type="button"
        class="p-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-destructive/10 hover:text-destructive transition-colors cursor-pointer"
        onclick={() => onClose?.()}
        aria-label="Close"
        title="Tutup (ESC)"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Scaled Image Container -->
    <div class="relative z-1 max-w-[90vw] max-h-[90vh] overflow-auto flex items-center justify-center p-4">
      <img
        {src}
        {alt}
        class="max-w-full max-h-[85vh] object-contain rounded-lg shadow-2xl transition-transform duration-150 ease-out will-change-transform select-none"
        style="transform: scale({zoomLevel});"
      />
    </div>
  </div>
{/if}
