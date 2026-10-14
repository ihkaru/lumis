<script lang="ts">
  import type { SessionTask, TaskStatus } from "$lib/types";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import Circle from "lucide-svelte/icons/circle";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import XCircle from "lucide-svelte/icons/x-circle";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import ChevronRight from "lucide-svelte/icons/chevron-right";
  import ListTodo from "lucide-svelte/icons/list-todo";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import { Badge } from "$lib/components/ui/badge";

  interface Props {
    tasks: SessionTask[];
    isGenerating?: boolean;
    className?: string;
  }

  let { tasks = [], isGenerating = false, className = "" }: Props = $props();

  let isCollapsed = $state(false);
  let expandedTaskIds = $state<Record<string, boolean>>({});

  let completedCount = $derived(
    tasks.filter((t) => t.status === "completed").length
  );
  let totalCount = $derived(tasks.length);
  let percentage = $derived(
    totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0
  );
  let hasActiveTask = $derived(
    tasks.some((t) => t.status === "in_progress")
  );

  // Group into root tasks and nested subtasks
  let rootTasks = $derived(
    tasks.filter((t) => !t.parent_task_id)
  );

  function getSubtasks(parentId: string): SessionTask[] {
    return tasks.filter((t) => t.parent_task_id === parentId);
  }

  function toggleTaskDetail(taskId: string) {
    expandedTaskIds[taskId] = !expandedTaskIds[taskId];
  }
</script>

{#if tasks.length > 0}
  <div
    class="my-3 rounded-xl border border-border/70 bg-card/60 backdrop-blur-xs shadow-xs overflow-hidden transition-all duration-200 {className}"
  >
    <!-- Header / Banner -->
    <div
      class="flex items-center justify-between px-3.5 py-2.5 bg-muted/30 cursor-pointer select-none hover:bg-muted/50 transition-colors"
      onclick={() => (isCollapsed = !isCollapsed)}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") isCollapsed = !isCollapsed;
      }}
      role="button"
      tabindex="0"
      aria-expanded={!isCollapsed}
    >
      <div class="flex items-center gap-2 min-w-0">
        <div class="p-1 rounded-md bg-primary/10 text-primary">
          <ListTodo class="w-4 h-4" />
        </div>
        <div class="flex items-center gap-2 min-w-0">
          <span class="text-xs font-semibold tracking-tight text-foreground truncate">
            Rencana Tugas Otonom
          </span>
          {#if hasActiveTask && isGenerating}
            <Badge variant="outline" class="h-5 px-1.5 gap-1 text-[10px] font-medium border-blue-500/30 text-blue-500 bg-blue-500/5">
              <Loader2 class="w-2.5 h-2.5 animate-spin" />
              Menjalankan
            </Badge>
          {:else if percentage === 100}
            <Badge variant="outline" class="h-5 px-1.5 gap-1 text-[10px] font-medium border-emerald-500/30 text-emerald-500 bg-emerald-500/5">
              <Sparkles class="w-2.5 h-2.5" />
              Selesai
            </Badge>
          {/if}
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <!-- Progress bar pill -->
        <div class="flex items-center gap-1.5 text-xs text-muted-foreground font-mono">
          <span>{completedCount}/{totalCount}</span>
          <div class="w-16 h-1.5 rounded-full bg-muted overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-300"
              style="width: {percentage}%"
            ></div>
          </div>
          <span class="text-[11px] font-medium">{percentage}%</span>
        </div>

        <button
          type="button"
          class="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/60 transition-colors"
          aria-label={isCollapsed ? "Buka rencana tugas" : "Tutup rencana tugas"}
        >
          <ChevronDown
            class="w-4 h-4 transition-transform duration-200 {isCollapsed ? '-rotate-90' : ''}"
          />
        </button>
      </div>
    </div>

    <!-- Tree Content -->
    {#if !isCollapsed}
      <div class="p-3 space-y-2 text-xs">
        {#each rootTasks as task (task.id)}
          {@const subtasks = getSubtasks(task.id)}
          {@const isExpanded = !!expandedTaskIds[task.id]}
          {@const hasDetail = !!(task.description || task.result_summary)}

          <div class="rounded-lg border border-border/40 bg-background/50 p-2.5 space-y-1.5 transition-colors">
            <!-- Root Task Row -->
            <div class="flex items-start gap-2.5">
              <div class="mt-0.5 shrink-0">
                {#if task.status === "completed"}
                  <CheckCircle2 class="w-4 h-4 text-emerald-500" />
                {:else if task.status === "in_progress"}
                  <Loader2 class="w-4 h-4 text-blue-500 animate-spin" />
                {:else if task.status === "cancelled"}
                  <XCircle class="w-4 h-4 text-muted-foreground/60" />
                {:else}
                  <Circle class="w-4 h-4 text-muted-foreground/60" />
                {/if}
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2">
                  <span
                    class="font-medium text-foreground break-words [overflow-wrap:anywhere] {task.status === 'cancelled' ? 'line-through text-muted-foreground' : ''}"
                  >
                    {task.title}
                  </span>

                  {#if hasDetail}
                    <button
                      type="button"
                      class="text-muted-foreground hover:text-foreground p-0.5 rounded transition-colors"
                      onclick={() => toggleTaskDetail(task.id)}
                      aria-label="Lihat rincian tugas"
                    >
                      <ChevronRight class="w-3.5 h-3.5 transition-transform {isExpanded ? 'rotate-90' : ''}" />
                    </button>
                  {/if}
                </div>

                {#if isExpanded && hasDetail}
                  <div class="mt-2 pt-2 border-t border-border/30 space-y-1.5 text-[11px] text-muted-foreground">
                    {#if task.description}
                      <p class="break-words [overflow-wrap:anywhere]">
                        <span class="font-medium text-foreground/80">Instruksi:</span> {task.description}
                      </p>
                    {/if}
                    {#if task.result_summary}
                      <div class="p-2 rounded bg-muted/40 border border-border/40 text-foreground/90 font-mono text-[10px] break-words [overflow-wrap:anywhere]">
                        <span class="font-semibold text-emerald-500 font-sans">Bukti Empiris:</span> {task.result_summary}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>

            <!-- Subtasks (Nested DAG) -->
            {#if subtasks.length > 0}
              <div class="ml-5 pl-2.5 border-l-2 border-border/60 space-y-1.5 mt-2">
                {#each subtasks as sub (sub.id)}
                  {@const isSubExpanded = !!expandedTaskIds[sub.id]}
                  {@const hasSubDetail = !!(sub.description || sub.result_summary)}

                  <div class="flex items-start gap-2 pt-1">
                    <div class="mt-0.5 shrink-0">
                      {#if sub.status === "completed"}
                        <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
                      {:else if sub.status === "in_progress"}
                        <Loader2 class="w-3.5 h-3.5 text-blue-500 animate-spin" />
                      {:else if sub.status === "cancelled"}
                        <XCircle class="w-3.5 h-3.5 text-muted-foreground/60" />
                      {:else}
                        <Circle class="w-3.5 h-3.5 text-muted-foreground/60" />
                      {/if}
                    </div>

                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between gap-2">
                        <span
                          class="text-[11px] font-medium text-foreground/90 break-words [overflow-wrap:anywhere] {sub.status === 'cancelled' ? 'line-through text-muted-foreground' : ''}"
                        >
                          {sub.title}
                        </span>

                        {#if hasSubDetail}
                          <button
                            type="button"
                            class="text-muted-foreground hover:text-foreground p-0.5 rounded transition-colors"
                            onclick={() => toggleTaskDetail(sub.id)}
                            aria-label="Lihat rincian subtask"
                          >
                            <ChevronRight class="w-3 h-3 transition-transform {isSubExpanded ? 'rotate-90' : ''}" />
                          </button>
                        {/if}
                      </div>

                      {#if isSubExpanded && hasSubDetail}
                        <div class="mt-1 pt-1 border-t border-border/20 text-[10px] text-muted-foreground space-y-1">
                          {#if sub.description}
                            <p class="break-words [overflow-wrap:anywhere]">{sub.description}</p>
                          {/if}
                          {#if sub.result_summary}
                            <p class="text-emerald-500 font-mono text-[9px] break-words [overflow-wrap:anywhere]">
                              {sub.result_summary}
                            </p>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}
