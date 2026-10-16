<script lang="ts">
  import { Marked } from "marked";
  import markedFootnote from "marked-footnote";
  import markedKatex from "marked-katex-extension";
  import DOMPurify from "dompurify";
  import CodeBlock from "./CodeBlock.svelte";
  import MermaidRenderer from "./MermaidRenderer.svelte";
  import ReasoningBox from "../message/ReasoningBox.svelte";
  import TableBlock from "./TableBlock.svelte";

  interface Props {
    content: string;
    isStreaming?: boolean;
  }

  let { content = "", isStreaming = false }: Props = $props();

  interface Block {
    type: "think" | "code" | "markdown";
    content: string;
    lang?: string;
  }

  const markedParser = new Marked();
  markedParser.use(markedFootnote());
  markedParser.use(markedKatex({
    throwOnError: false,
    nonStandard: true,
  }));
  const alertIcons: Record<string, string> = {
    NOTE: `<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M12 16v-4"></path><path d="M12 8h.01"></path></svg>`,
    TIP: `<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18h6"></path><path d="M10 22h4"></path><path d="M12 2a7 7 0 0 0-7 7c0 2.5 1.5 4.5 3 6h8c1.5-1.5 3-3.5 3-6a7 7 0 0 0-7-7z"></path></svg>`,
    IMPORTANT: `<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>`,
    WARNING: `<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>`,
    CAUTION: `<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"></polygon><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>`
  };

  markedParser.use({
    renderer: {
      blockquote(token) {
        const text = token.text || "";
        const match = text.match(/^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?:\s*\n|\s*)([\s\S]*)$/i);
        if (match) {
          const type = match[1].toUpperCase();
          const title = type.charAt(0) + type.slice(1).toLowerCase();
          const icon = alertIcons[type] || "";
          const clonedTokens = JSON.parse(JSON.stringify(token.tokens || []));
          if (clonedTokens[0]?.type === "paragraph" && clonedTokens[0]?.tokens?.[0]?.type === "text") {
            clonedTokens[0].tokens[0].text = clonedTokens[0].tokens[0].text.replace(/^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?:\s*\n|\s*)/i, "");
            clonedTokens[0].text = clonedTokens[0].text.replace(/^\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?:\s*\n|\s*)/i, "");
          }
          const body = (this as any).parser.parse(clonedTokens);
          return `<div class="markdown-alert markdown-alert-${type.toLowerCase()}"><div class="markdown-alert-header"><span class="markdown-alert-title">${icon}<span>${title}</span></span></div><div class="markdown-alert-body">${body}</div></div>\n`;
        }
        return `<blockquote>${(this as any).parser.parse(token.tokens || [])}</blockquote>\n`;
      }
    }
  });

  // Parse structured blocks from content (think, code fences, markdown)
  let parsedBlocks = $derived.by(() => {
    const blocks: Block[] = [];
    let text = content;

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
    const fenceRegex = /```([a-zA-Z0-9_-]*)[ \t]*\n([\s\S]*?)```/g;
    let lIdx = 0;
    let fMatch: RegExpExecArray | null;

    while ((fMatch = fenceRegex.exec(raw)) !== null) {
      if (fMatch.index > lIdx) {
        const mdText = raw.slice(lIdx, fMatch.index);
        if (mdText.trim()) acc.push({ type: "markdown", content: mdText });
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
        acc.push({ type: "markdown", content: remaining });
      }
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
        if (text.trim()) segments.push({ type: "html", content: text });
      }
      segments.push({ type: "table", content: match[0] });
      lastIdx = match.index + match[0].length;
    }

    if (lastIdx < html.length) {
      const rest = html.slice(lastIdx);
      if (rest.trim()) segments.push({ type: "html", content: rest });
    }

    return segments.length > 0 ? segments : [{ type: "html", content: html }];
  }

  function renderMarkdownHtml(mdText: string): string {
    try {
      const raw = markedParser.parse(mdText, {
        async: false,
        gfm: true,
        breaks: true,
      }) as string;
      if (typeof window !== "undefined") {
        return DOMPurify.sanitize(raw, {
          USE_PROFILES: { html: true, mathMl: true, svg: true },
          ADD_TAGS: ["annotation", "semantics", "span", "div", "section"],
          ADD_ATTR: [
            "data-footnote-ref",
            "data-footnote-backref",
            "data-footnotes",
            "aria-label",
            "aria-hidden",
            "align",
            "target",
            "style"
          ],
          FORBID_TAGS: ["script", "iframe"],
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
