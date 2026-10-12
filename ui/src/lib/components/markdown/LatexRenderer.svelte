<script lang="ts">
  import katex from "katex";

  interface Props {
    math: string;
    displayMode?: boolean;
  }

  let { math = "", displayMode = false }: Props = $props();

  let html = $derived.by(() => {
    try {
      return katex.renderToString(math, {
        displayMode,
        throwOnError: false,
      });
    } catch (e) {
      return `<span class="text-destructive">${math}</span>`;
    }
  });
</script>

<!-- Render KaTeX HTML safely -->
{#if displayMode}
  <div class="my-3 overflow-x-auto py-2 text-center select-all">
    {@html html}
  </div>
{:else}
  <span class="inline-block select-all">
    {@html html}
  </span>
{/if}
