<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiCanvas from "./OpenUiCanvas.svelte";

  interface Props {
    node: any;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state: uiState, onStateChange }: Props = $props();
</script>

{#if node.type === "Canvas"}
  <div class="rounded-xl border border-border/60 bg-card/75 p-3 backdrop-blur-xs shadow-xs space-y-2 w-full overflow-hidden">
    <svg
      viewBox="0 0 {node.props.width || 400} {node.props.height || 250}"
      class="w-full h-auto select-none rounded-lg border border-border/40 bg-background/50"
    >
      {#if node.children}
        {#each node.children as child}
          <OpenUiCanvas node={child} state={uiState} {onStateChange} />
        {/each}
      {/if}
    </svg>
  </div>
{:else if node.type === "Rect"}
  <rect
    x={node.props.x}
    y={node.props.y}
    width={node.props.width}
    height={node.props.height}
    rx={node.props.rx ?? 0}
    fill={node.props.fill ?? "#3b82f6"}
    stroke={node.props.stroke ?? "none"}
  />
{:else if node.type === "Circle"}
  <circle
    cx={node.props.cx}
    cy={node.props.cy}
    r={node.props.r}
    fill={node.props.fill ?? "#10b981"}
    stroke={node.props.stroke ?? "none"}
  />
{:else if node.type === "Line"}
  <line
    x1={node.props.x1}
    y1={node.props.y1}
    x2={node.props.x2}
    y2={node.props.y2}
    stroke={node.props.stroke ?? "#8b5cf6"}
    stroke-width={node.props.strokeWidth ?? 2}
  />
{:else if node.type === "Path"}
  <path
    d={node.props.d}
    fill={node.props.fill ?? "none"}
    stroke={node.props.stroke ?? "currentColor"}
    stroke-width={node.props.strokeWidth ?? 2}
  />
{:else if node.type === "SvgText"}
  <text
    x={node.props.x}
    y={node.props.y}
    font-size={node.props.fontSize ?? 12}
    fill={node.props.fill ?? "currentColor"}
    class="font-sans font-medium"
  >
    {node.props.text}
  </text>
{:else if node.type === "Group"}
  <g transform={node.props.transform || undefined}>
    {#if node.children}
      {#each node.children as child}
        <OpenUiCanvas node={child} state={uiState} {onStateChange} />
      {/each}
    {/if}
  </g>
{/if}
