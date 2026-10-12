<script lang="ts">
  import Bot from "lucide-svelte/icons/bot";
  import Zap from "lucide-svelte/icons/zap";
  import Eye from "lucide-svelte/icons/eye";
  import MarkdownRenderer from "../markdown/MarkdownRenderer.svelte";
  import ToolCard from "./ToolCard.svelte";
  import CitationsView, { type Citation } from "./CitationsView.svelte";
  import MessageActions from "./MessageActions.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { parseArtifactsFromContent } from "$lib/artifact-parser";
  import type { ArtifactSpecification, Message } from "$lib/types";

  interface Props {
    message: Message;
    isStreaming?: boolean;
    onOpenArtifact?: (artifact: ArtifactSpecification) => void;
    onRegenerate?: () => void;
    onFork?: () => void;
  }

  let { message, isStreaming = false, onOpenArtifact, onRegenerate, onFork }: Props = $props();

  let parsed = $derived(parseArtifactsFromContent(message.content));

  // Extract tools & citations from metadata if present
  let toolCalls = $derived(
    (message.metadata?.tool_calls as Array<{ tool: string; args: any; result: any }>) || []
  );

  let citations = $derived(
    (message.metadata?.citations as Citation[]) || []
  );
</script>

<div class="flex justify-start font-sans group">
  <div class="flex flex-col items-start max-w-3xl w-full">
    <!-- Bubble Header -->
    <div class="flex items-center gap-2 mb-1.5 text-[11px] text-muted-foreground select-none">
      <div class="p-1 rounded-full bg-cyan-950/80 border border-cyan-800 text-cyan-400">
        <Bot class="h-3.5 w-3.5" />
      </div>
      <span class="font-semibold text-foreground">Lumis Assistant</span>
      {#if message.model_id}
        <Badge variant="outline" class="font-mono text-[10px] py-0 px-1.5 text-muted-foreground border-border/60">
          {message.model_id}
        </Badge>
      {/if}
    </div>

    <!-- Bubble Body -->
    <div class="w-full px-5 py-4 rounded-2xl rounded-tl-none bg-card border border-border/80 text-foreground shadow-xs [overflow-wrap:anywhere]">
      <!-- Tool Cards -->
      {#each toolCalls as tc}
        <ToolCard toolName={tc.tool} args={tc.args} result={tc.result} />
      {/each}

      <!-- Markdown & Reasoning Body -->
      <MarkdownRenderer content={parsed.cleanText} {isStreaming} />

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
            <div class="flex items-center justify-between p-2.5 rounded-xl bg-background/80 border border-border gap-2">
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
