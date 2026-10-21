<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";
  import OpenUiControls from "./OpenUiControls.svelte";
  import OpenUiDisplay from "./OpenUiDisplay.svelte";
  import OpenUiCharts from "./OpenUiCharts.svelte";
  import OpenUiMedia from "./OpenUiMedia.svelte";
  import OpenUiCanvas from "./OpenUiCanvas.svelte";
  import OpenUiPlay from "./OpenUiPlay.svelte";

  interface Props {
    node: UiNode | string | number | boolean | null;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state, onStateChange }: Props = $props();

  const CONTROL_TYPES = new Set([
    "Button",
    "IconButton",
    "Pressable",
    "Slider",
    "Input",
    "TextArea",
    "Select",
    "SegmentedControl",
    "RadioGroup",
    "Checkbox",
    "DatePicker",
    "Timer",
  ]);

  const DISPLAY_TYPES = new Set([
    "Title",
    "Subtitle",
    "Text",
    "Caption",
    "Badge",
    "Label",
    "Link",
    "Code",
    "Quote",
    "Image",
    "Math",
    "Markdown",
    "List",
    "ListItem",
    "Table",
  ]);

  const CHART_TYPES = new Set([
    "LineChart",
    "BarChart",
    "PieChart",
    "DonutChart",
    "AreaChart",
  ]);

  const MEDIA_TYPES = new Set([
    "Carousel",
    "Popover",
    "Video",
    "Map",
  ]);

  const CANVAS_TYPES = new Set([
    "Canvas",
    "Rect",
    "Circle",
    "Line",
    "Path",
    "SvgText",
    "Group",
  ]);

  const PLAY_TYPES = new Set([
    "FlashCard",
    "Confetti",
    "Audio",
  ]);
</script>

{#if node === null || node === undefined}
  <!-- Empty -->
{:else if typeof node !== "object"}
  <span>{String(node)}</span>
{:else if CONTROL_TYPES.has(node.type)}
  <OpenUiControls {node} {state} {onStateChange} />
{:else if DISPLAY_TYPES.has(node.type)}
  <OpenUiDisplay {node} {state} {onStateChange} />
{:else if CHART_TYPES.has(node.type)}
  <OpenUiCharts {node} {state} {onStateChange} />
{:else if MEDIA_TYPES.has(node.type)}
  <OpenUiMedia {node} {state} {onStateChange} />
{:else if CANVAS_TYPES.has(node.type)}
  <OpenUiCanvas {node} {state} {onStateChange} />
{:else if PLAY_TYPES.has(node.type)}
  <OpenUiPlay {node} {state} {onStateChange} />
{:else if node.type === "Card"}
  <div class="rounded-xl border border-border/60 bg-card/75 p-4 backdrop-blur-xs shadow-xs space-y-3">
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Col"}
  <div class="flex flex-col space-y-2.5">
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Row"}
  <div class="flex flex-row items-center gap-2.5 flex-wrap">
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Grid"}
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Box"}
  <div
    class="flex {node.props.direction === 'row' ? 'flex-row items-center flex-wrap' : 'flex-col'} {node.props.border ? 'rounded-lg border border-border/60 bg-muted/20' : ''}"
    style="gap: {(node.props.gap ?? 2) * 4}px; padding: {(node.props.padding ?? 0) * 4}px;"
  >
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Form"}
  <form
    onsubmit={(e) => {
      e.preventDefault();
      if (typeof node.props.onSubmit === "function") node.props.onSubmit();
    }}
    class="space-y-3 w-full"
  >
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </form>
{:else if node.type === "Spacer"}
  <div class="flex-1"></div>
{:else if node.type === "Divider"}
  <hr class="border-t border-border/50 my-2" />
{:else}
  <!-- Generic Container Fallback -->
  <div class="space-y-1">
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{/if}
