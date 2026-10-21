<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";

  interface Props {
    node: UiNode;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state, onStateChange }: Props = $props();

  function handleInputChange(e: Event, key?: string, type?: string) {
    if (!key) return;
    const target = e.target as HTMLInputElement;
    const val = type === "number" ? parseFloat(target.value) || 0 : target.value;
    onStateChange(key, val);
  }

  function handleSliderChange(e: Event, key?: string) {
    if (!key) return;
    const target = e.target as HTMLInputElement;
    onStateChange(key, parseFloat(target.value) || 0);
  }

  function handleCheckboxToggle(key?: string) {
    if (!key) return;
    onStateChange(key, !state[key]);
  }

  function handleSegmentSelect(key: string | undefined, val: string) {
    if (!key) return;
    onStateChange(key, val);
  }
</script>

{#if node.type === "Button"}
  <button
    type="button"
    disabled={node.props.disabled}
    onclick={() => typeof node.props.action === "function" && node.props.action()}
    class="inline-flex items-center justify-center gap-1.5 rounded-lg px-3.5 py-1.5 text-xs font-medium transition-colors focus-visible:outline-hidden disabled:opacity-50 {node.props.variant === 'outline' ? 'border border-border bg-background hover:bg-muted text-foreground' : node.props.variant === 'ghost' ? 'hover:bg-muted text-foreground' : 'bg-primary text-primary-foreground hover:bg-primary/90'}"
  >
    {node.props.label}
  </button>
{:else if node.type === "IconButton"}
  <button
    type="button"
    onclick={() => typeof node.props.action === "function" && node.props.action()}
    class="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-background/80 hover:bg-muted text-foreground transition-colors"
    title={node.props.label || node.props.icon}
  >
    <span class="text-xs font-semibold">{node.props.icon === 'plus' ? '+' : node.props.icon === 'minus' ? '−' : '•'}</span>
  </button>
{:else if node.type === "Pressable"}
  <div
    role="button"
    tabindex="0"
    onclick={() => typeof node.props.action === "function" && node.props.action()}
    onkeydown={(e) => e.key === "Enter" && typeof node.props.action === "function" && node.props.action()}
    class="rounded-lg p-2 transition-colors cursor-pointer {node.props.selected ? 'bg-primary/10 border border-primary/30' : 'hover:bg-muted/50 border border-transparent'}"
  >
    {#if node.children}
      {#each node.children as child}
        <OpenUiNode node={child} {state} {onStateChange} />
      {/each}
    {/if}
  </div>
{:else if node.type === "Slider"}
  <div class="flex flex-col gap-1.5 w-full">
    <div class="flex justify-between text-xs">
      <span class="text-muted-foreground">{node.props.label}</span>
      <span class="font-medium text-foreground">{state[node.props.bindingVar] ?? node.props.value}{node.props.unit ? ' ' + node.props.unit : ''}</span>
    </div>
    <input
      type="range"
      id={"openui_slider_" + (node.props.bindingVar || "val")}
      min={node.props.min}
      max={node.props.max}
      step={node.props.step}
      value={state[node.props.bindingVar] ?? node.props.value}
      oninput={(e) => handleSliderChange(e, node.props.bindingVar)}
      class="w-full h-1.5 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary"
    />
  </div>
{:else if node.type === "Input"}
  <div class="flex flex-col gap-1 w-full">
    {#if node.props.label}
      <span class="text-xs text-muted-foreground">{node.props.label}</span>
    {/if}
    <input
      type={node.props.type || "text"}
      id={"openui_input_" + (node.props.bindingVar || "text")}
      placeholder={node.props.placeholder}
      value={state[node.props.bindingVar] ?? node.props.value}
      oninput={(e) => handleInputChange(e, node.props.bindingVar, node.props.type)}
      class="h-8 rounded-lg border border-border bg-background/80 px-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:border-primary"
    />
  </div>
{:else if node.type === "TextArea"}
  <div class="flex flex-col gap-1 w-full">
    {#if node.props.label}
      <span class="text-xs text-muted-foreground">{node.props.label}</span>
    {/if}
    <textarea
      id={"openui_textarea_" + (node.props.bindingVar || "text")}
      placeholder={node.props.placeholder}
      rows={node.props.rows || 3}
      value={state[node.props.bindingVar] ?? node.props.value}
      oninput={(e) => handleInputChange(e, node.props.bindingVar)}
      class="rounded-lg border border-border bg-background/80 p-2 text-xs text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:border-primary resize-y"
    ></textarea>
  </div>
{:else if node.type === "Select"}
  <div class="flex flex-col gap-1 w-full">
    {#if node.props.label}
      <span class="text-xs text-muted-foreground">{node.props.label}</span>
    {/if}
    <select
      id={"openui_select_" + (node.props.bindingVar || "sel")}
      value={state[node.props.bindingVar] ?? node.props.value}
      onchange={(e) => handleInputChange(e, node.props.bindingVar)}
      class="h-8 rounded-lg border border-border bg-background/80 px-2.5 text-xs text-foreground focus:outline-hidden focus:border-primary"
    >
      {#each node.props.options as opt}
        {@const val = typeof opt === "object" ? opt.value : opt}
        {@const label = typeof opt === "object" ? opt.label : opt}
        <option value={val}>{label}</option>
      {/each}
    </select>
  </div>
{:else if node.type === "SegmentedControl"}
  <div class="inline-flex rounded-lg border border-border bg-muted/40 p-0.5">
    {#each node.props.options as opt}
      {@const val = typeof opt === "object" ? opt.value : opt}
      {@const label = typeof opt === "object" ? opt.label : opt}
      {@const isSelected = (state[node.props.bindingVar] ?? node.props.value) == val}
      <button
        type="button"
        onclick={() => handleSegmentSelect(node.props.bindingVar, val)}
        class="rounded-md px-3 py-1 text-xs font-medium transition-all {isSelected ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'}"
      >
        {label}
      </button>
    {/each}
  </div>
{:else if node.type === "RadioGroup"}
  <div class="flex flex-col gap-1.5 w-full">
    {#if node.props.label}
      <span class="text-xs text-muted-foreground font-medium">{node.props.label}</span>
    {/if}
    <div class="flex flex-col gap-1 sm:flex-row sm:gap-3">
      {#each node.props.options as opt}
        {@const val = typeof opt === "object" ? opt.value : opt}
        {@const label = typeof opt === "object" ? opt.label : opt}
        <label class="inline-flex items-center gap-2 cursor-pointer text-xs text-foreground">
          <input
            type="radio"
            name={"openui_radio_" + (node.props.bindingVar || "group")}
            value={val}
            checked={(state[node.props.bindingVar] ?? node.props.value) == val}
            onchange={() => handleSegmentSelect(node.props.bindingVar, val)}
            class="h-3.5 w-3.5 text-primary border-border bg-background focus:ring-0"
          />
          {label}
        </label>
      {/each}
    </div>
  </div>
{:else if node.type === "Checkbox"}
  <label class="inline-flex items-center gap-2 cursor-pointer select-none">
    <input
      type="checkbox"
      id={"openui_check_" + (node.props.bindingVar || "check")}
      checked={!!(state[node.props.bindingVar] ?? node.props.checked)}
      onchange={() => handleCheckboxToggle(node.props.bindingVar)}
      class="h-4 w-4 rounded-sm border-border bg-background text-primary focus:ring-0"
    />
    <span class="text-xs {state[node.props.bindingVar] && node.props.lineThrough ? 'line-through text-muted-foreground' : 'text-foreground'}">
      {node.props.label}
    </span>
  </label>
{:else if node.type === "DatePicker"}
  <div class="flex flex-col gap-1 w-full">
    {#if node.props.label}
      <span class="text-xs text-muted-foreground">{node.props.label}</span>
    {/if}
    <input
      type="date"
      id={"openui_date_" + (node.props.bindingVar || "date")}
      min={node.props.min}
      max={node.props.max}
      value={state[node.props.bindingVar] ?? node.props.value}
      oninput={(e) => handleInputChange(e, node.props.bindingVar)}
      class="h-8 rounded-lg border border-border bg-background/80 px-2.5 text-xs text-foreground focus:outline-hidden focus:border-primary"
    />
  </div>
{:else if node.type === "Timer"}
  <div class="inline-flex items-center gap-2 rounded-lg border border-border bg-muted/30 px-3 py-1.5 font-mono text-xs font-semibold text-foreground">
    <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
    <span>{String(Math.floor((state[node.props.bindingVar] ?? node.props.seconds ?? 0) / 60)).padStart(2, '0')}:{String(Math.floor((state[node.props.bindingVar] ?? node.props.seconds ?? 0) % 60)).padStart(2, '0')}</span>
  </div>
{/if}
