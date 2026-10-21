<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";

  interface Props {
    node: UiNode | string | number | boolean | null;
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

{#if node === null || node === undefined}
  <!-- Empty -->
{:else if typeof node !== "object"}
  <span>{String(node)}</span>
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
{:else if node.type === "Title"}
  <h3 class="font-sans font-semibold tracking-tight text-foreground {node.props.size === 'lg' ? 'text-xl' : node.props.size === 'xl' ? 'text-2xl' : 'text-base'}">
    {node.props.text}
  </h3>
{:else if node.type === "Subtitle"}
  <p class="text-xs sm:text-sm text-muted-foreground font-sans">
    {node.props.text}
  </p>
{:else if node.type === "Text"}
  <p class="text-sm leading-relaxed text-foreground/90 font-sans {node.props.weight === 'semibold' ? 'font-semibold' : ''} {node.props.color === 'muted' ? 'text-muted-foreground' : ''}">
    {#if Array.isArray(node.props.text)}
      {#each node.props.text as part}
        <OpenUiNode node={part} {state} {onStateChange} />
      {/each}
    {:else}
      {node.props.text}
    {/if}
  </p>
{:else if node.type === "Caption"}
  <span class="text-xs text-muted-foreground font-sans">
    {node.props.text}
  </span>
{:else if node.type === "Badge"}
  <span class="inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium border {node.props.color === 'success' ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : node.props.color === 'info' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20' : node.props.color === 'warning' ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' : node.props.color === 'danger' ? 'bg-rose-500/10 text-rose-400 border-rose-500/20' : 'bg-secondary text-secondary-foreground border-border/50'}">
    {node.props.text}
  </span>
{:else if node.type === "Button"}
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
{:else if node.type === "Link"}
  <a
    href={node.props.url}
    target="_blank"
    rel="noopener noreferrer"
    class="text-xs text-primary underline underline-offset-2 hover:text-primary/80 transition-colors"
  >
    {node.props.text}
  </a>
{:else if node.type === "Code"}
  <pre class="rounded-lg border border-border bg-muted/40 p-2.5 text-xs font-mono overflow-x-auto text-foreground/90"><code>{node.props.text}</code></pre>
{:else if node.type === "Quote"}
  <blockquote class="border-l-2 border-primary/60 pl-3 py-1 my-1 text-xs italic text-muted-foreground">
    "{node.props.text}"
    {#if node.props.author}
      <span class="block not-italic font-medium text-foreground mt-0.5">— {node.props.author}</span>
    {/if}
  </blockquote>
{:else if node.type === "Image"}
  <img
    src={node.props.src}
    alt={node.props.alt || ""}
    width={node.props.width}
    height={node.props.height}
    class="rounded-lg max-w-full h-auto object-cover border border-border/50"
    loading="lazy"
  />
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
{:else if node.type === "Table"}
  <div class="overflow-x-auto rounded-lg border border-border">
    <table class="w-full text-xs text-left">
      <thead class="bg-muted/50 text-muted-foreground uppercase text-[10px]">
        <tr>
          {#each node.props.columns as col}
            <th class="px-3 py-2 font-medium">{col}</th>
          {/each}
        </tr>
      </thead>
      <tbody class="divide-y divide-border/50">
        {#each node.props.rows as row}
          <tr class="hover:bg-muted/30">
            {#each row as cell}
              <td class="px-3 py-2 text-foreground/90">
                <OpenUiNode node={cell} {state} {onStateChange} />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
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
