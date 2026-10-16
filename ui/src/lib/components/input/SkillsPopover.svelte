<script lang="ts">
  import ScrollText from "lucide-svelte/icons/scroll-text";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import { skillsApi } from "$lib/skills-api";
  import type { SkillSummary } from "$lib/types";

  interface Props {
    open: boolean;
    query: string;
    onSelect: (skill: SkillSummary) => void;
    onClose: () => void;
  }

  let { open = false, query = "", onSelect, onClose }: Props = $props();

  let skills = $state<SkillSummary[]>([]);
  let selectedIndex = $state(0);
  let isLoading = $state(false);

  $effect(() => {
    if (!open) {
      skills = [];
      selectedIndex = 0;
      return;
    }

    let active = true;
    isLoading = true;

    skillsApi
      .searchSkills(query, 8)
      .then((res) => {
        if (active) {
          skills = res.skills.filter((s) => s.is_enabled && s.user_invocable);
          selectedIndex = 0;
          isLoading = false;
        }
      })
      .catch(() => {
        if (active) {
          skills = [];
          isLoading = false;
        }
      });

    return () => {
      active = false;
    };
  });

  export function handleKeyDown(e: KeyboardEvent): boolean {
    if (!open || skills.length === 0) return false;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % skills.length;
      return true;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + skills.length) % skills.length;
      return true;
    }

    if (e.key === "Enter" || e.key === "Tab") {
      if (skills[selectedIndex]) {
        e.preventDefault();
        onSelect(skills[selectedIndex]);
        return true;
      }
    }

    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
      return true;
    }

    return false;
  }
</script>

{#if open}
  <div
    class="absolute bottom-full left-0 mb-2 w-full max-w-md bg-card/95 backdrop-blur-md border border-border/80 rounded-xl shadow-lg overflow-hidden z-50 font-sans animate-in fade-in zoom-in-95 duration-100"
    role="listbox"
    aria-label="Pilih Skill"
  >
    <div class="flex items-center justify-between px-3 py-2 border-b border-border/60 bg-muted/40 text-xs text-muted-foreground">
      <div class="flex items-center gap-1.5 font-medium text-foreground">
        <Sparkles class="size-3.5 text-primary" />
        <span>Pintasan Skill Enterprise</span>
      </div>
      <span>Ketik nama atau pilih</span>
    </div>

    {#if isLoading && skills.length === 0}
      <div class="px-4 py-3 text-xs text-muted-foreground text-center">
        Mencari skill terindeks...
      </div>
    {:else if skills.length === 0}
      <div class="px-4 py-3 text-xs text-muted-foreground text-center">
        Tidak ada skill yang cocok dengan "{query}"
      </div>
    {:else}
      <div class="max-h-56 overflow-y-auto py-1">
        {#each skills as skill, idx}
          <button
            type="button"
            class="w-full flex items-start gap-2.5 px-3 py-2 text-left transition-colors {idx === selectedIndex ? 'bg-primary/10 text-foreground' : 'hover:bg-muted/50 text-foreground'}"
            role="option"
            aria-selected={idx === selectedIndex}
            onclick={() => onSelect(skill)}
          >
            <div class="mt-0.5 p-1 rounded-md bg-primary/15 text-primary shrink-0">
              <ScrollText class="size-3.5" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="font-medium text-xs text-foreground truncate">${skill.name}</span>
                <span class="text-[10px] text-muted-foreground font-mono bg-muted/60 px-1 rounded truncate">
                  {skill.category}
                </span>
              </div>
              <p class="text-[11px] text-muted-foreground line-clamp-1 mt-0.5">
                {skill.description}
              </p>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}
