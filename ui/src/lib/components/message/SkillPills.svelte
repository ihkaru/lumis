<script lang="ts">
  import ScrollText from "lucide-svelte/icons/scroll-text";
  import Pin from "lucide-svelte/icons/pin";

  interface Props {
    manualSkills?: string[];
    alwaysAppliedSkills?: string[];
    class?: string;
  }

  let {
    manualSkills = [],
    alwaysAppliedSkills = [],
    class: className = "",
  }: Props = $props();

  let hasSkills = $derived(manualSkills.length > 0 || alwaysAppliedSkills.length > 0);
</script>

{#if hasSkills}
  <div class="flex flex-wrap items-center gap-1.5 py-1 select-none {className}" role="list" aria-label="Skill aktif">
    {#each alwaysAppliedSkills as skillName}
      <span
        role="listitem"
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-secondary/80 text-foreground border border-border/60 shadow-2xs"
        title="Skill otomatis aktif (Always-Apply)"
      >
        <Pin class="size-3 text-primary shrink-0" aria-hidden="true" />
        <span class="max-w-[12rem] truncate">{skillName}</span>
      </span>
    {/each}

    {#each manualSkills as skillName}
      <span
        role="listitem"
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium bg-primary/10 text-primary border border-primary/20 shadow-2xs"
        title="Skill dipanggil manual ($)"
      >
        <ScrollText class="size-3 text-primary shrink-0" aria-hidden="true" />
        <span class="max-w-[12rem] truncate">{skillName}</span>
      </span>
    {/each}
  </div>
{/if}
