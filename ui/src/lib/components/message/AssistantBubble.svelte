<script lang="ts">
  import Bot from "lucide-svelte/icons/bot";
  import Zap from "lucide-svelte/icons/zap";
  import Eye from "lucide-svelte/icons/eye";
  import MarkdownRenderer from "../markdown/MarkdownRenderer.svelte";
  import ToolCard from "./ToolCard.svelte";
  import CitationsView, { type Citation } from "./CitationsView.svelte";
  import MessageActions from "./MessageActions.svelte";
  import MessagePagination from "./MessagePagination.svelte";
  import DeepResearchProgress, { type ResearchStep } from "./DeepResearchProgress.svelte";
  import ConfidenceBadge from "./ConfidenceBadge.svelte";
  import MessageMinimap from "./MessageMinimap.svelte";
  import ZoomableModal from "../common/ZoomableModal.svelte";
  import TaskProgressTree from "../task/TaskProgressTree.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { parseArtifactsFromContent } from "$lib/artifact-parser";
  import type { ArtifactSpecification, Message, SessionTask } from "$lib/types";

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

  // Extract preliminary text before tool execution if present
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

  let confidenceScore = $derived(
    typeof message.metadata?.confidence_score === "number" ? message.metadata.confidence_score : 0.96
  );

  let groundingRatio = $derived(
    typeof message.metadata?.grounding_ratio === "number" ? message.metadata.grounding_ratio : 0.92
  );

  let zoomModal = $state<{ open: boolean; src: string; alt: string }>({
    open: false,
    src: "",
    alt: ""
  });
</script>

<div class="flex justify-start font-sans group">
  <div class="flex flex-col items-start max-w-3xl w-full">
    <!-- Bubble Header with Model, Confidence, Minimap & Branch Pagination -->
    <div class="flex items-center justify-between w-full mb-1.5 text-[11px] text-muted-foreground select-none">
      <div class="flex items-center gap-2 flex-wrap">
        <div class="p-1 rounded-full bg-cyan-950/80 border border-cyan-800 text-cyan-400">
          <Bot class="h-3.5 w-3.5" />
        </div>
        <span class="font-semibold text-foreground">Lumis Assistant</span>
        {#if message.model_id}
          <Badge variant="outline" class="font-mono text-[10px] py-0 px-1.5 text-muted-foreground border-border/60">
            {message.model_id}
          </Badge>
        {/if}
        <!-- Factuality & Confidence Badge -->
        <ConfidenceBadge score={confidenceScore} {groundingRatio} />
      </div>

      <div class="flex items-center gap-2">
        <!-- Message Outline Minimap -->
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
    </div>

    <!-- Assistant Response Stream (Borderless, Flat Modern Layout) -->
    <div class="w-full text-foreground [overflow-wrap:anywhere] space-y-3 pt-0.5">
      <!-- Initial generation indicator while waiting for tokens -->
      {#if isStreaming && !parsed.cleanText && !preContent && toolCalls.length === 0 && (!tasks || tasks.length === 0)}
        <div class="inline-flex items-center gap-2 text-xs text-muted-foreground animate-pulse py-1">
          <span class="w-1.5 h-1.5 rounded-full bg-cyan-400"></span>
          <span>Sedang memproses respons...</span>
        </div>
      {/if}

      <!-- Preliminary Response / Explanation (if model emitted text before calling tool) -->
      {#if preContent}
        <MarkdownRenderer content={preContent} isStreaming={false} />
      {/if}

      <!-- Tool Execution & Result Cards (Rendered immediately after LLM response content) -->
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

      <!-- Autonomous Plan / Session Tasks (Rendered inline with executing assistant turn) -->
      {#if tasks && tasks.length > 0}
        <TaskProgressTree {tasks} isGenerating={isStreaming} />
      {/if}

      <!-- Main / Post-Tool LLM Response Stream -->
      {#if parsed.cleanText}
        <MarkdownRenderer content={parsed.cleanText} {isStreaming} />
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
            <Zap class="h-3.5 w-3.5 text-cyan-400" />
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
      content={message.content}
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
