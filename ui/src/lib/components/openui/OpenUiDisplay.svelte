<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";

  interface Props {
    node: UiNode;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state, onStateChange }: Props = $props();
</script>

{#if node.type === "Title"}
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
{:else if node.type === "Label"}
  <label for={node.props.forId} class="text-xs font-medium text-foreground inline-flex items-center gap-1">
    {node.props.text}
    {#if node.props.required}
      <span class="text-rose-500">*</span>
    {/if}
  </label>
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
{:else if node.type === "Math"}
  <div class="my-1.5 overflow-x-auto py-1 font-mono text-sm tracking-wide text-foreground">
    <span class="rounded bg-muted/40 px-2 py-0.5 border border-border/40 inline-block font-sans">{node.props.formula}</span>
  </div>
{:else if node.type === "Markdown"}
  <div class="prose prose-xs dark:prose-invert max-w-none text-xs text-foreground/90 font-sans">
    {node.props.content}
  </div>
{:else if node.type === "List"}
  {#if node.props.ordered}
    <ol class="list-decimal list-inside space-y-1 text-xs text-foreground/90">
      {#each node.props.items as item}
        <li><OpenUiNode node={item} {state} {onStateChange} /></li>
      {/each}
    </ol>
  {:else}
    <ul class="list-disc list-inside space-y-1 text-xs text-foreground/90">
      {#each node.props.items as item}
        <li><OpenUiNode node={item} {state} {onStateChange} /></li>
      {/each}
    </ul>
  {/if}
{:else if node.type === "ListItem"}
  <li class="text-xs text-foreground/90">
    <OpenUiNode node={node.props.content} {state} {onStateChange} />
  </li>
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
{/if}
