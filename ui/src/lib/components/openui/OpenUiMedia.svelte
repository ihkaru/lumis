<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";

  interface Props {
    node: UiNode;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state: uiState, onStateChange }: Props = $props();

  let carouselIndex = $state(0);
  let isPopoverOpen = $state(false);

  function nextSlide(len: number) {
    if (len <= 0) return;
    carouselIndex = (carouselIndex + 1) % len;
  }

  function prevSlide(len: number) {
    if (len <= 0) return;
    carouselIndex = (carouselIndex - 1 + len) % len;
  }
</script>

{#if node.type === "Carousel"}
  {@const slides = node.children || []}
  <div class="relative w-full overflow-hidden rounded-xl border border-border/60 bg-card/60 p-2 backdrop-blur-xs">
    <div class="overflow-hidden rounded-lg">
      <div
        class="flex transition-transform duration-300 ease-out"
        style="transform: translate3d(-{carouselIndex * 100}%, 0, 0);"
      >
        {#each slides as slide}
          <div class="w-full shrink-0 p-2">
            <OpenUiNode node={slide} state={uiState} {onStateChange} />
          </div>
        {/each}
      </div>
    </div>

    {#if slides.length > 1}
      <button
        type="button"
        onclick={() => prevSlide(slides.length)}
        class="absolute left-3 top-1/2 -translate-y-1/2 flex h-7 w-7 items-center justify-center rounded-full bg-background/80 border border-border text-foreground shadow-xs hover:bg-muted transition-colors text-xs"
        aria-label="Previous slide"
      >
        ‹
      </button>
      <button
        type="button"
        onclick={() => nextSlide(slides.length)}
        class="absolute right-3 top-1/2 -translate-y-1/2 flex h-7 w-7 items-center justify-center rounded-full bg-background/80 border border-border text-foreground shadow-xs hover:bg-muted transition-colors text-xs"
        aria-label="Next slide"
      >
        ›
      </button>
      <div class="flex justify-center gap-1.5 pt-2 pb-1">
        {#each slides as _, i}
          <button
            type="button"
            onclick={() => (carouselIndex = i)}
            class="h-1.5 rounded-full transition-all {carouselIndex === i ? 'w-4 bg-primary' : 'w-1.5 bg-muted-foreground/30'}"
            aria-label="Go to slide {i + 1}"
          ></button>
        {/each}
      </div>
    {/if}
  </div>
{:else if node.type === "Popover"}
  <div class="relative inline-block">
    <div
      role="button"
      tabindex="0"
      onclick={() => (isPopoverOpen = !isPopoverOpen)}
      onkeydown={(e) => e.key === "Enter" && (isPopoverOpen = !isPopoverOpen)}
      class="cursor-pointer inline-block"
    >
      <OpenUiNode node={node.props.trigger} state={uiState} {onStateChange} />
    </div>

    {#if isPopoverOpen}
      <!-- Backdrop dismiss -->
      <button
        type="button"
        tabindex="-1"
        class="fixed inset-0 z-20 cursor-default bg-transparent border-0 p-0"
        onclick={() => (isPopoverOpen = false)}
        aria-label="Close popover"
      ></button>
      <div class="absolute left-0 top-full z-30 mt-1.5 w-64 rounded-xl border border-border/70 bg-card p-3 shadow-xl backdrop-blur-md">
        <OpenUiNode node={node.props.content} state={uiState} {onStateChange} />
      </div>
    {/if}
  </div>
{:else if node.type === "Video"}
  <div class="rounded-xl border border-border/60 overflow-hidden bg-black/40">
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      src={node.props.src}
      poster={node.props.poster}
      controls={node.props.controls ?? true}
      autoplay={node.props.autoplay ?? false}
      class="w-full h-auto max-h-72 object-cover"
    ></video>
  </div>
{:else if node.type === "Map"}
  {@const lat = Number(node.props.latitude) || -6.1754}
  {@const lon = Number(node.props.longitude) || 106.8272}
  {@const bbox = `${lon - 0.015},${lat - 0.015},${lon + 0.015},${lat + 0.015}`}
  <div class="rounded-xl border border-border/60 bg-card/75 p-3 backdrop-blur-xs space-y-2">
    <div class="flex items-center justify-between">
      <span class="text-xs font-semibold text-foreground tracking-tight">{node.props.title || "Peta Wilayah"}</span>
      <span class="text-[10px] font-mono rounded bg-muted/60 px-2 py-0.5 text-muted-foreground">{lat.toFixed(4)}, {lon.toFixed(4)}</span>
    </div>
    <div class="overflow-hidden rounded-lg border border-border/50 h-44 w-full">
      <iframe
        title={node.props.title || "Peta Interaktif"}
        src="https://www.openstreetmap.org/export/embed.html?bbox={encodeURIComponent(bbox)}&layer=mapnik&marker={lat}%2C{lon}"
        class="w-full h-full border-0 select-none"
        loading="lazy"
      ></iframe>
    </div>
    <div class="flex justify-end">
      <a
        href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=15/{lat}/{lon}"
        target="_blank"
        rel="noopener noreferrer"
        class="text-[10px] text-primary underline underline-offset-2 hover:text-primary/80"
      >
        Buka di OpenStreetMap ↗
      </a>
    </div>
  </div>
{/if}
