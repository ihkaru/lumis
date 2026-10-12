<script lang="ts">
  import { marked } from "marked";
  import CodeBlock from "./CodeBlock.svelte";
  import LatexRenderer from "./LatexRenderer.svelte";
  import ReasoningBox from "../message/ReasoningBox.svelte";

  interface Props {
    content: string;
    isStreaming?: boolean;
  }

  let { content = "", isStreaming = false }: Props = $props();

  interface Block {
    type: "think" | "code" | "math_block" | "markdown";
    content: string;
    lang?: string;
  }

  // Parse structured blocks from content
  let parsedBlocks = $derived.by(() => {
    const blocks: Block[] = [];
    let text = content;

    // 1. Extract <think> blocks
    const thinkRegex = /<think>([\s\S]*?)(?:<\/think>|$)/gi;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = thinkRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        const pre = text.slice(lastIndex, match.index);
        if (pre.trim()) processMarkdownAndCode(pre, blocks);
      }
      blocks.push({
        type: "think",
        content: match[1] || "",
      });
      lastIndex = match.index + match[0].length;
    }

    if (lastIndex < text.length) {
      const rest = text.slice(lastIndex);
      if (rest.trim()) processMarkdownAndCode(rest, blocks);
    }

    return blocks.length > 0 ? blocks : [{ type: "markdown", content: text }];
  });

  function processMarkdownAndCode(raw: string, acc: Block[]) {
    // Extract code fences ```lang ... ```
    const fenceRegex = /```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g;
    let lIdx = 0;
    let fMatch: RegExpExecArray | null;

    while ((fMatch = fenceRegex.exec(raw)) !== null) {
      if (fMatch.index > lIdx) {
        const mdText = raw.slice(lIdx, fMatch.index);
        if (mdText.trim()) processMathBlocks(mdText, acc);
      }
      acc.push({
        type: "code",
        lang: fMatch[1] || "text",
        content: fMatch[2] || "",
      });
      lIdx = fMatch.index + fMatch[0].length;
    }

    if (lIdx < raw.length) {
      const remaining = raw.slice(lIdx);
      if (remaining.trim()) processMathBlocks(remaining, acc);
    }
  }

  function processMathBlocks(raw: string, acc: Block[]) {
    // Extract block math $$ ... $$
    const mathRegex = /\$\$([\s\S]*?)\$\$/g;
    let idx = 0;
    let mMatch: RegExpExecArray | null;

    while ((mMatch = mathRegex.exec(raw)) !== null) {
      if (mMatch.index > idx) {
        const t = raw.slice(idx, mMatch.index);
        if (t.trim()) acc.push({ type: "markdown", content: t });
      }
      acc.push({
        type: "math_block",
        content: mMatch[1] || "",
      });
      idx = mMatch.index + mMatch[0].length;
    }

    if (idx < raw.length) {
      const rem = raw.slice(idx);
      if (rem.trim()) acc.push({ type: "markdown", content: rem });
    }
  }

  function renderMarkdownHtml(mdText: string): string {
    try {
      return marked.parse(mdText, {
        async: false,
        gfm: true,
        breaks: true,
      }) as string;
    } catch {
      return mdText;
    }
  }
</script>

<div class="prose prose-invert max-w-none text-sm leading-relaxed [overflow-wrap:anywhere] break-words font-sans space-y-2">
  {#each parsedBlocks as block}
    {#if block.type === "think"}
      <ReasoningBox content={block.content} {isStreaming} />
    {:else if block.type === "code"}
      <CodeBlock code={block.content} lang={block.lang} />
    {:else if block.type === "math_block"}
      <LatexRenderer math={block.content} displayMode={true} />
    {:else}
      <div class="markdown-body">
        {@html renderMarkdownHtml(block.content)}
      </div>
    {/if}
  {/each}
</div>

<style>
  :global(.markdown-body table) {
    width: 100%;
    margin-top: 0.75rem;
    margin-bottom: 0.75rem;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }
  :global(.markdown-body th),
  :global(.markdown-body td) {
    border: 1px solid var(--border);
    padding: 0.5rem 0.75rem;
    text-align: left;
  }
  :global(.markdown-body th) {
    background-color: rgba(255, 255, 255, 0.05);
    font-weight: 600;
  }
  :global(.markdown-body blockquote) {
    border-left: 3px solid var(--primary);
    padding-left: 1rem;
    margin: 0.75rem 0;
    color: var(--muted-foreground);
    font-style: italic;
  }
  :global(.markdown-body a) {
    color: #38bdf8;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  :global(.markdown-body code:not(pre code)) {
    background-color: rgba(255, 255, 255, 0.1);
    padding: 0.15rem 0.35rem;
    border-radius: 0.25rem;
    font-family: monospace;
    font-size: 0.8em;
  }
</style>
