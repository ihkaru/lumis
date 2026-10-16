<script lang="ts">
  import Zap from "lucide-svelte/icons/zap";
  import Eye from "lucide-svelte/icons/eye";
  import MarkdownRenderer from "../markdown/MarkdownRenderer.svelte";
  import ToolCard from "./ToolCard.svelte";
  import CitationsView, { type Citation } from "./CitationsView.svelte";
  import MessageActions from "./MessageActions.svelte";
  import MessagePagination from "./MessagePagination.svelte";
  import DeepResearchProgress, { type ResearchStep } from "./DeepResearchProgress.svelte";
  import MessageMinimap from "./MessageMinimap.svelte";
  import ZoomableModal from "../common/ZoomableModal.svelte";
  import TaskProgressTree from "../task/TaskProgressTree.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { parseArtifactsFromContent } from "$lib/artifact-parser";
  import type { ArtifactSpecification, Message, MessagePart, SessionTask } from "$lib/types";

  interface Props {
    message: Message;
    isStreaming?: boolean;
    tasks?: SessionTask[];
    branchIndex?: number;
    branchTotal?: number;
    onPrevBranch?: () => void;
    onNextBranch?: () => void;
    onOpenArtifact?: (artifact: ArtifactSpecification) => void;
    onRegenerate?: () => void;
    onFork?: () => void;
  }

  let {
    message,
    isStreaming = false,
    tasks = [],
    branchIndex = 1,
    branchTotal = 1,
    onPrevBranch,
    onNextBranch,
    onOpenArtifact,
    onRegenerate,
    onFork
  }: Props = $props();

  let parsed = $derived(parseArtifactsFromContent(message.content));

  // Extract sequential parts if present (modern interleaved model)
  let parts = $derived((message.metadata?.parts as MessagePart[]) || []);

  // Extract preliminary text before tool execution if present (legacy fallback)
  let preContent = $derived((message.metadata?.pre_content as string) || "");

  // Extract tools, research steps & citations from metadata if present
  let toolCalls = $derived(
    (message.metadata?.tool_calls as Array<{ tool: string; server?: string; args: any; result: any; error?: string; isRunning?: boolean }>) || []
  );

  let citations = $derived(
    (message.metadata?.citations as Citation[]) || []
  );

  let researchSteps = $derived(
    (message.metadata?.research_steps as ResearchStep[]) || []
  );

  let fullCopyText = $derived.by(() => {
    if (parts.length > 0) {
      const formattedParts = parts.map((part) => {
        if (part.type === "text") return parseArtifactsFromContent(part.text).cleanText;
        if (part.type === "tool_call") {
          const name = part.tool || "tool";
          const input = typeof part.args === "string" ? part.args : JSON.stringify(part.args, null, 2);
          let b = `### Tool Execution: \`${name}\`\n**Input:**\n\`\`\`json\n${input}\n\`\`\``;
          if (part.error) b += `\n**Error:**\n\`\`\`\n${part.error}\n\`\`\``;
          else if (part.result !== undefined && part.result !== null) {
            const out = typeof part.result === "string" ? part.result : JSON.stringify(part.result, null, 2);
            b += `\n**Output:**\n\`\`\`json\n${out}\n\`\`\``;
          }
          return b;
        }
        return "";
      }).filter(Boolean);
      return formattedParts.join("\n\n");
    }

    const blocks: string[] = [];
    if (preContent) blocks.push(preContent);
    for (const tc of toolCalls) {
      const name = tc.tool || "tool";
      const input = typeof tc.args === "string" ? tc.args : JSON.stringify(tc.args, null, 2);
      let b = `### Tool Execution: \`${name}\`\n**Input:**\n\`\`\`json\n${input}\n\`\`\``;
      if (tc.error) b += `\n**Error:**\n\`\`\`\n${tc.error}\n\`\`\``;
      else if (tc.result !== undefined && tc.result !== null) {
        const out = typeof tc.result === "string" ? tc.result : JSON.stringify(tc.result, null, 2);
        b += `\n**Output:**\n\`\`\`json\n${out}\n\`\`\``;
      }
      blocks.push(b);
    }
    if (parsed.cleanText) blocks.push(parsed.cleanText);
    return blocks.length > 0 ? blocks.join("\n\n") : message.content;
  });

  let zoomModal = $state<{ open: boolean; src: string; alt: string }>({
    open: false,
    src: "",
    alt: ""
  });
</script>

<div class="flex justify-start font-sans group" data-role="assistant" data-message-id={message.id}>
  <div class="flex flex-col items-start max-w-3xl w-full">
    <!-- Top Utility Controls (Only shown when Branch Pagination or Outline Minimap is active) -->
    {#if branchTotal > 1 || parsed.cleanText.includes("##")}
      <div class="flex items-center justify-end w-full mb-1 text-[11px] text-muted-foreground select-none gap-2">
        <MessageMinimap content={message.content} />
        {#if branchTotal > 1}
          <MessagePagination
            current={branchIndex}
            total={branchTotal}
            onPrev={onPrevBranch}
            onNext={onNextBranch}
          />
        {/if}
      </div>
    {/if}

    <!-- Assistant Response Stream (Borderless, Flat Modern Layout) -->
    <div class="w-full text-foreground [overflow-wrap:anywhere] space-y-3 pt-0.5">
      <!-- Initial generation indicator while waiting for tokens -->
      {#if isStreaming && parts.length === 0 && !parsed.cleanText && !preContent && toolCalls.length === 0 && (!tasks || tasks.length === 0)}
        <div class="inline-flex items-center gap-2 text-xs text-muted-foreground animate-pulse py-1">
          <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
          <span>Sedang memproses respons...</span>
        </div>
      {/if}

      <!-- Sequential Content Blocks (Ordered Interleaved Flow) -->
      {#if parts.length > 0}
        {#each parts as part, idx}
          {#if part.type === "text"}
            {@const partParsed = parseArtifactsFromContent(part.text)}
            {#if partParsed.cleanText}
              <MarkdownRenderer
                content={partParsed.cleanText}
                isStreaming={isStreaming && idx === parts.length - 1}
              />
            {/if}
          {:else if part.type === "tool_call"}
            <ToolCard
              toolName={part.tool}
              serverName={part.server}
              args={part.args}
              result={part.result}
              error={part.error}
              isRunning={part.isRunning}
            />
          {/if}
        {/each}
        {#if isStreaming && parts[parts.length - 1]?.type === "tool_call"}
          <div class="inline-flex items-center gap-2 text-xs text-muted-foreground animate-pulse py-1">
            <span class="w-1.5 h-1.5 rounded-full bg-primary animate-ping"></span>
            <span>Menganalisis hasil dan menyusun jawaban...</span>
          </div>
        {/if}
      {:else}
        <!-- Legacy Fallback (for historical records in DB without parts) -->
        {#if preContent}
          <MarkdownRenderer content={preContent} isStreaming={false} />
        {/if}

        {#each toolCalls as tc}
          <ToolCard
            toolName={tc.tool}
            serverName={tc.server}
            args={tc.args}
            result={tc.result}
            error={tc.error}
            isRunning={tc.isRunning}
          />
        {/each}

        {#if parsed.cleanText}
          <MarkdownRenderer content={parsed.cleanText} {isStreaming} />
        {/if}
      {/if}

      <!-- Autonomous Plan / Session Tasks (Rendered inline with executing assistant turn) -->
      {#if tasks && tasks.length > 0}
        <TaskProgressTree {tasks} isGenerating={isStreaming} />
      {/if}

      <!-- Deep Research Timeline if active -->
      {#if researchSteps.length > 0}
        <DeepResearchProgress steps={researchSteps} isActive={isStreaming} />
      {/if}

      <!-- Citations View -->
      <CitationsView {citations} />

      <!-- Artifacts Preview List -->
      {#if parsed.artifacts.length > 0}
        <div class="mt-4 pt-3 border-t border-border/60 space-y-2">
          <div class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
            <Zap class="h-3.5 w-3.5 text-primary" />
            <span>Interactive Artifacts ({parsed.artifacts.length})</span>
          </div>
          {#each parsed.artifacts as art}
            <div class="flex items-center justify-between p-2.5 rounded-xl bg-muted/30 border border-border/60 gap-2">
              <div class="flex items-center gap-2 min-w-0">
                <span class="font-medium text-xs text-foreground truncate">{art.title}</span>
                <Badge variant="outline" class="text-[10px] uppercase font-mono shrink-0">
                  {art.artifact_type.split("/")[1] || art.artifact_type}
                </Badge>
              </div>
              <Button
                variant="secondary"
                size="sm"
                class="h-7 px-2.5 text-xs gap-1.5 font-sans shrink-0 cursor-pointer"
                onclick={() => onOpenArtifact?.(art)}
              >
                <Eye class="w-3.5 h-3.5" />
                <span>Buka Canvas</span>
              </Button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Action Bar -->
    <MessageActions
      content={fullCopyText}
      role="assistant"
      {onRegenerate}
      {onFork}
    />
  </div>
</div>

<!-- Lightbox Zoom Modal for Images / Diagrams -->
<ZoomableModal
  open={zoomModal.open}
  src={zoomModal.src}
  alt={zoomModal.alt}
  onClose={() => (zoomModal.open = false)}
/>
