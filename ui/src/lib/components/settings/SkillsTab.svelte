<script lang="ts">
  import Sparkles from "lucide-svelte/icons/sparkles";
  import Search from "lucide-svelte/icons/search";
  import Check from "lucide-svelte/icons/check";
  import X from "lucide-svelte/icons/x";
  import Tag from "lucide-svelte/icons/tag";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import { skillsApi } from "$lib/skills-api";
  import type { SkillSummary } from "$lib/types";

  let searchQuery = $state("");
  let skills = $state<SkillSummary[]>([]);
  let isLoading = $state(true);
  let errorMsg = $state<string | null>(null);

  async function loadSkills() {
    isLoading = true;
    errorMsg = null;
    try {
      if (searchQuery.trim()) {
        const res = await skillsApi.searchSkills(searchQuery.trim(), 50);
        skills = res.skills;
      } else {
        const res = await skillsApi.listSkills({ limit: 100 });
        skills = res.skills;
      }
    } catch (err: any) {
      errorMsg = err.message || "Gagal memuat daftar skills";
    } finally {
      isLoading = false;
    }
  }

  async function handleToggle(skill: SkillSummary) {
    const nextState = !skill.is_enabled;
    try {
      // Optimistic update
      skills = skills.map((s) => (s.id === skill.id ? { ...s, is_enabled: nextState } : s));
      await skillsApi.toggleSkill(skill.id, nextState);
    } catch (err: any) {
      // Revert on error
      skills = skills.map((s) => (s.id === skill.id ? { ...s, is_enabled: !nextState } : s));
      errorMsg = err.message || "Gagal memperbarui status skill";
    }
  }

  $effect(() => {
    const query = searchQuery;
    const timer = setTimeout(() => {
      loadSkills();
    }, 250);
    return () => clearTimeout(timer);
  });
</script>

<div class="space-y-4 font-sans text-foreground">
  <!-- Header & Description -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-border">
    <div>
      <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
        <Sparkles class="size-4 text-primary" />
        Katalog Skill Enterprise
      </h3>
      <p class="text-xs text-muted-foreground mt-0.5 [overflow-wrap:anywhere]">
        Kelola keahlian agen on-demand (diindeks sub-milidetik FTS5). Panggil langsung di chat dengan tanda <span class="font-mono text-primary font-medium">$nama-skill</span>.
      </p>
    </div>
  </div>

  <!-- Search Bar -->
  <div class="relative">
    <Search class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Cari keahlian (contoh: data-analysis, code-reviewer, sql)..."
      class="w-full pl-9 pr-4 py-2 bg-muted/40 border border-border/80 rounded-xl text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary transition min-h-[44px] touch-manipulation [overflow-wrap:anywhere]"
    />
  </div>

  {#if errorMsg}
    <div class="p-3 bg-destructive/10 border border-destructive/30 rounded-xl text-xs text-destructive flex items-center justify-between">
      <span>{errorMsg}</span>
      <button
        type="button"
        onclick={() => loadSkills()}
        class="underline font-medium hover:text-destructive/80 cursor-pointer min-h-[32px] px-2 touch-manipulation"
      >
        Coba lagi
      </button>
    </div>
  {/if}

  <!-- Skills Catalog List -->
  {#if isLoading && skills.length === 0}
    <div class="py-12 text-center text-xs text-muted-foreground flex flex-col items-center justify-center gap-2">
      <div class="size-5 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
      <span>Memuat katalog keahlian...</span>
    </div>
  {:else if skills.length === 0}
    <div class="py-12 text-center text-xs text-muted-foreground border border-dashed border-border/60 rounded-xl p-6">
      Tidak ada keahlian yang cocok dengan pencarian "{searchQuery}".
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-2.5">
      {#each skills as skill (skill.id)}
        <div class="p-3 sm:p-3.5 bg-card/60 hover:bg-card border border-border/60 hover:border-border rounded-xl transition-all flex flex-col sm:flex-row sm:items-start justify-between gap-3 shadow-xs">
          <div class="min-w-0 flex-1 space-y-1">
            <div class="flex items-center flex-wrap gap-1.5">
              <span class="font-mono text-xs font-semibold text-foreground bg-primary/10 text-primary px-2 py-0.5 rounded-md border border-primary/20">
                ${skill.name}
              </span>
              <span class="text-xs font-medium text-foreground truncate max-w-[200px]">
                {skill.display_title}
              </span>
              <span class="text-[10px] text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
                v{skill.version}
              </span>
              {#if skill.always_apply}
                <span class="text-[10px] bg-amber-500/10 text-amber-500 font-medium px-1.5 py-0.5 rounded border border-amber-500/20 flex items-center gap-1">
                  <ShieldCheck class="size-3" /> Selalu Aktif
                </span>
              {/if}
            </div>

            <p class="text-xs text-muted-foreground line-clamp-2 leading-relaxed [overflow-wrap:anywhere]">
              {skill.description}
            </p>

            {#if skill.category}
              <div class="flex items-center gap-1 pt-1">
                <Tag class="size-3 text-muted-foreground shrink-0" />
                <span class="text-[10px] text-muted-foreground bg-muted/60 px-1.5 py-0.5 rounded border border-border/40 font-mono">
                  {skill.category}
                </span>
              </div>
            {/if}
          </div>

          <!-- Toggle Button -->
          <div class="shrink-0 flex sm:flex-col items-center sm:items-end justify-between sm:justify-center gap-2 pt-2 sm:pt-0 border-t sm:border-t-0 border-border/40">
            <button
              type="button"
              onclick={() => handleToggle(skill)}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium transition cursor-pointer min-h-[44px] sm:min-h-[36px] touch-manipulation {skill.is_enabled ? 'bg-primary/15 text-primary border border-primary/30 hover:bg-primary/20' : 'bg-muted text-muted-foreground border border-border/60 hover:bg-muted/80'}"
              aria-label={skill.is_enabled ? `Nonaktifkan skill ${skill.display_title}` : `Aktifkan skill ${skill.display_title}`}
            >
              {#if skill.is_enabled}
                <Check class="size-3.5" />
                <span>Aktif</span>
              {:else}
                <X class="size-3.5" />
                <span>Nonaktif</span>
              {/if}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
