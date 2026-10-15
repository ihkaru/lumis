<script lang="ts">
  import GitGraph from "lucide-svelte/icons/git-graph";

  interface Props {
    code: string;
  }

  let { code = "" }: Props = $props();
</script>

<div class="my-3 overflow-hidden rounded-xl border border-primary/30 bg-muted/30 dark:bg-slate-950 p-4 font-sans text-xs shadow-md">
  <div class="flex items-center gap-2 mb-3 text-primary border-b border-border/40 pb-2">
    <GitGraph class="h-4 w-4" />
    <span class="font-semibold text-xs tracking-wider uppercase font-mono">Mermaid Architecture Diagram</span>
  </div>
  <div class="p-3 bg-card/60 rounded-lg overflow-x-auto flex items-center justify-center text-foreground font-mono text-[11px] whitespace-pre">
    <code>{code}</code>
  </div>
</div>
