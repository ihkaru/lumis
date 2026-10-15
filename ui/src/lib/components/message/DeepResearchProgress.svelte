<script lang="ts">
  import Compass from "lucide-svelte/icons/compass";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import ChevronUp from "lucide-svelte/icons/chevron-up";
  import Search from "lucide-svelte/icons/search";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import { Badge } from "$lib/components/ui/badge";

  export interface ResearchStep {
    id: string;
    title: string;
    query?: string;
    status: "pending" | "running" | "completed" | "failed";
    duration_ms?: number;
  }

  interface Props {
    steps: ResearchStep[];
    isActive?: boolean;
    class?: string;
  }

  let { steps = [], isActive = false, class: className = "" }: Props = $props();

  let isExpanded = $state(true);

  let completedCount = $derived(steps.filter((s) => s.status === "completed").length);
  let hasRunning = $derived(steps.some((s) => s.status === "running") || isActive);

  function formatDuration(ms?: number): string {
    if (!ms) return "";
    return `${(ms / 1000).toFixed(1)}s`;
  }
</script>

{#if steps.length > 0}
  <div class="my-3 rounded-xl border border-primary/20 bg-primary/5 backdrop-blur-xs overflow-hidden font-sans text-xs {className}">
    <!-- Header banner -->
    <button
      type="button"
      class="w-full flex items-center justify-between px-3.5 py-2.5 bg-primary/10 hover:bg-primary/15 transition-colors cursor-pointer select-none"
      onclick={() => (isExpanded = !isExpanded)}
    >
      <div class="flex items-center gap-2">
        <div class="p-1 rounded-full bg-primary/20 text-primary">
          {#if hasRunning}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
          {:else}
            <Compass class="w-3.5 h-3.5" />
          {/if}
        </div>
        <span class="font-medium text-foreground">
          Deep Research Engine
        </span>
        <Badge variant="outline" class="text-[10px] py-0 px-1.5 border-primary/30 text-primary font-mono">
          {completedCount} / {steps.length} langkah
        </Badge>
      </div>

      <div class="flex items-center gap-1.5 text-primary">
        {#if isExpanded}
          <ChevronUp class="w-4 h-4" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </div>
    </button>

    <!-- Expanded steps list -->
    {#if isExpanded}
      <div class="p-3 space-y-2 border-t border-primary/15 bg-background/50">
        {#each steps as step, idx}
          <div class="flex items-start gap-2.5 p-2 rounded-lg bg-card/60 border border-border/40 text-xs">
            <div class="mt-0.5 shrink-0">
              {#if step.status === "completed"}
                <CheckCircle2 class="w-4 h-4 text-emerald-400" />
              {:else if step.status === "running"}
                <Loader2 class="w-4 h-4 text-primary animate-spin" />
              {:else if step.status === "failed"}
                <AlertCircle class="w-4 h-4 text-destructive" />
              {:else}
                <div class="w-4 h-4 rounded-full border border-border flex items-center justify-center text-[10px] text-muted-foreground font-mono">
                  {idx + 1}
                </div>
              {/if}
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex items-center justify-between gap-2">
                <span class="font-medium text-foreground truncate">{step.title}</span>
                {#if step.duration_ms}
                  <span class="text-[10px] font-mono text-muted-foreground shrink-0">
                    {formatDuration(step.duration_ms)}
                  </span>
                {/if}
              </div>

              {#if step.query}
                <div class="mt-1 flex items-center gap-1.5 text-[11px] text-muted-foreground font-mono bg-muted/40 px-2 py-0.5 rounded border border-border/40 truncate">
                  <Search class="w-3 h-3 text-primary shrink-0" />
                  <span class="truncate">{step.query}</span>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}
