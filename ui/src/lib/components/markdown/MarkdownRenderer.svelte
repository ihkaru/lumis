<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import CodeBlock from "./CodeBlock.svelte";
  import LatexRenderer from "./LatexRenderer.svelte";
  import MermaidRenderer from "./MermaidRenderer.svelte";
  import ReasoningBox from "../message/ReasoningBox.svelte";
  import TableBlock from "./TableBlock.svelte";

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

    const fallback: Block[] = [{ type: "markdown", content: text }];
    return blocks.length > 0 ? blocks : fallback;
  });

  function processMarkdownAndCode(raw: string, acc: Block[]) {
    // Extract code fences ```lang ... ```
    const fenceRegex = /```([a-zA-Z0-9_-]*)[ \t]*\n([\s\S]*?)```/g;
    let lIdx = 0;
    let fMatch: RegExpExecArray | null;

    while ((fMatch = fenceRegex.exec(raw)) !== null) {
      if (fMatch.index > lIdx) {
        const mdText = raw.slice(lIdx, fMatch.index);
        if (mdText.trim()) processMathBlocks(mdText, acc);
      }
      acc.push({
        type: "code",
        lang: (fMatch[1] || "text").trim().toLowerCase(),
        content: fMatch[2] || "",
      });
      lIdx = fMatch.index + fMatch[0].length;
    }

    if (lIdx < raw.length) {
      const remaining = raw.slice(lIdx);
      const unclosedMatch = remaining.match(/^```([a-zA-Z0-9_-]*)[ \t]*\n([\s\S]*)$/);
      if (unclosedMatch) {
        acc.push({
          type: "code",
          lang: (unclosedMatch[1] || "text").trim().toLowerCase(),
          content: unclosedMatch[2] || "",
        });
      } else if (remaining.trim()) {
        processMathBlocks(remaining, acc);
      }
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

  interface HtmlSegment {
    type: "html" | "table";
    content: string;
  }

  function splitHtmlTables(html: string): HtmlSegment[] {
    const tableRegex = /<table[\s\S]*?<\/table>/gi;
    const segments: HtmlSegment[] = [];
    let lastIdx = 0;
    let match: RegExpExecArray | null;

    while ((match = tableRegex.exec(html)) !== null) {
      if (match.index > lastIdx) {
        const text = html.slice(lastIdx, match.index);
        if (text.trim()) {
          segments.push({ type: "html", content: text });
        }
      }
      segments.push({ type: "table", content: match[0] });
      lastIdx = match.index + match[0].length;
    }

    if (lastIdx < html.length) {
      const rest = html.slice(lastIdx);
      if (rest.trim()) {
        segments.push({ type: "html", content: rest });
      }
    }

    return segments.length > 0 ? segments : [{ type: "html", content: html }];
  }

  function renderMarkdownHtml(mdText: string): string {
    try {
      const raw = marked.parse(mdText, {
        async: false,
        gfm: true,
        breaks: true,
      }) as string;
      if (typeof window !== "undefined") {
        return DOMPurify.sanitize(raw, {
          USE_PROFILES: { html: true },
          FORBID_TAGS: ["script", "style"],
          FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover", "onfocus", "onblur"],
        });
      }
      return raw;
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
      {#if block.lang === "mermaid"}
        <MermaidRenderer code={block.content} />
      {:else}
        <CodeBlock code={block.content} lang={block.lang} />
      {/if}
    {:else if block.type === "math_block"}
      <LatexRenderer math={block.content} displayMode={true} />
    {:else}
      {#each splitHtmlTables(renderMarkdownHtml(block.content)) as segment}
        {#if segment.type === "table"}
          <TableBlock html={segment.content} />
        {:else}
          <div class="markdown-body">
            {@html segment.content}
          </div>
        {/if}
      {/each}
    {/if}
  {/each}
</div>

<style>
  :global(.markdown-body p) {
    margin-top: 0.375rem;
    margin-bottom: 0.375rem;
    line-height: 1.55;
    color: var(--foreground, #e2e8f0);
  }
  :global(.markdown-body p:first-child) {
    margin-top: 0;
  }
  :global(.markdown-body p:last-child) {
    margin-bottom: 0;
  }
  :global(.markdown-body h1) {
    font-size: 1.15rem;
    font-weight: 600;
    letter-spacing: -0.015em;
    margin-top: 0.875rem;
    margin-bottom: 0.375rem;
    color: var(--foreground, #ffffff);
  }
  :global(.markdown-body h2) {
    font-size: 1.05rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    margin-top: 0.75rem;
    margin-bottom: 0.25rem;
    color: var(--foreground, #ffffff);
  }
  :global(.markdown-body h3) {
    font-size: 0.95rem;
    font-weight: 600;
    letter-spacing: -0.005em;
    margin-top: 0.625rem;
    margin-bottom: 0.2rem;
    color: var(--foreground, #ffffff);
  }
  :global(.markdown-body ul) {
    list-style-type: disc !important;
    padding-left: 1.15rem !important;
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }
  :global(.markdown-body ol) {
    list-style-type: decimal !important;
    padding-left: 1.15rem !important;
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }
  :global(.markdown-body li) {
    margin-top: 0.125rem;
    margin-bottom: 0.125rem;
    line-height: 1.5;
  }
  :global(.markdown-body li > p) {
    margin-top: 0;
    margin-bottom: 0;
    display: inline;
  }
  :global(.markdown-body ul ul) {
    list-style-type: circle !important;
    padding-left: 1.15rem !important;
    margin-top: 0.15rem;
    margin-bottom: 0.15rem;
  }
  :global(.markdown-body ol ol) {
    list-style-type: lower-alpha !important;
    padding-left: 1.15rem !important;
    margin-top: 0.15rem;
    margin-bottom: 0.15rem;
  }
  :global(.markdown-body ul ol),
  :global(.markdown-body ol ul) {
    padding-left: 1.15rem !important;
    margin-top: 0.15rem;
    margin-bottom: 0.15rem;
  }
  :global(.markdown-body strong) {
    font-weight: 600;
    color: var(--foreground, #ffffff);
  }
  :global(.markdown-body table) {
    width: 100%;
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }
  :global(.markdown-body th),
  :global(.markdown-body td) {
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 0.3rem 0.55rem;
    text-align: left;
  }
  :global(.markdown-body th) {
    background-color: rgba(255, 255, 255, 0.04);
    font-weight: 600;
  }
  :global(.markdown-body blockquote) {
    border-left: 1.5px solid hsl(var(--primary) / 0.6);
    padding-left: 0.5rem;
    margin: 0.25rem 0;
    color: var(--muted-foreground);
    font-style: italic;
  }
  :global(.markdown-body a) {
    color: #38bdf8;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :global(.markdown-body code:not(pre code)) {
    background-color: rgba(255, 255, 255, 0.07);
    padding: 0.1rem 0.3rem;
    border-radius: 0.25rem;
    font-family: monospace;
    font-size: 0.8em;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  :global(.markdown-body hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    margin: 0.625rem 0;
  }
</style>
