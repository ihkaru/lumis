<script lang="ts">
  import { onMount } from "svelte";
  import Cpu from "lucide-svelte/icons/cpu";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import Check from "lucide-svelte/icons/check";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import Eye from "lucide-svelte/icons/eye";
  import Wrench from "lucide-svelte/icons/wrench";
  import Search from "lucide-svelte/icons/search";
  import Lock from "lucide-svelte/icons/lock";
  import Plus from "lucide-svelte/icons/plus";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import SlidersHorizontal from "lucide-svelte/icons/sliders-horizontal";
  import { api } from "$lib/api";
  import type { ModelDescriptor } from "$lib/types";

  interface Props {
    selectedModelId: string;
    onSelectModel: (modelId: string) => void;
  }

  let { selectedModelId = "my9model-free", onSelectModel }: Props = $props();

  let isOpen = $state(false);
  let models = $state<ModelDescriptor[]>([]);
  let isLoading = $state(false);
  let searchQuery = $state("");
  let selectedCategory = $state<"all" | "reasoning" | "vision" | "combo">("all");
  let customModelInput = $state("");
  let showCustomInput = $state(false);

  async function loadModels() {
    isLoading = true;
    try {
      const data = await api.getModels();
      if (Array.isArray(data) && data.length > 0) {
        models = data;
      }
    } catch (e) {
      console.warn("Gagal memuat katalog model dinamis, menggunakan fallback", e);
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    loadModels();
  });

  let activeModel = $derived(
    models.find((m) => m.id === selectedModelId) || {
      id: selectedModelId,
      name: selectedModelId,
      provider: "AI Gateway",
      tier: "Standard",
      is_default: false,
      can_access: true,
      capabilities: undefined,
    }
  );

  let filteredModels = $derived(
    models.filter((m) => {
      const q = searchQuery.toLowerCase().trim();
      const matchSearch =
        !q ||
        m.name.toLowerCase().includes(q) ||
        m.id.toLowerCase().includes(q) ||
        m.provider.toLowerCase().includes(q);

      if (!matchSearch) return false;

      if (selectedCategory === "reasoning") {
        return (
          m.capabilities?.reasoning ||
          m.tier === "Reasoning" ||
          m.id.includes("5.2") ||
          m.id.includes("r1")
        );
      }
      if (selectedCategory === "vision") {
        return (
          m.capabilities?.vision ||
          m.tier === "Advanced" ||
          m.id.includes("5v") ||
          m.id.includes("m3") ||
          m.id.includes("k2")
        );
      }
      if (selectedCategory === "combo") {
        return (
          m.owned_by === "combo" ||
          m.owned_by === "openrouter" ||
          m.id.includes("free") ||
          m.id.includes("openrouter")
        );
      }
      return true;
    })
  );

  function applyCustomModel() {
    const trimmed = customModelInput.trim();
    if (trimmed) {
      onSelectModel(trimmed);
      customModelInput = "";
      showCustomInput = false;
      isOpen = false;
    }
  }

  let triggerBtnEl = $state<HTMLButtonElement | null>(null);
  let popupStyle = $state("");

  function updatePopupPosition() {
    if (!triggerBtnEl || typeof window === "undefined") return;
    const rect = triggerBtnEl.getBoundingClientRect();
    const isMobile = window.innerWidth < 640;
    if (isMobile) {
      popupStyle = `bottom: calc(100vh - ${rect.top}px + 8px); left: 1rem; width: calc(100vw - 2rem);`;
    } else {
      const left = Math.max(16, Math.min(rect.left, window.innerWidth - 400));
      popupStyle = `bottom: calc(100vh - ${rect.top}px + 8px); left: ${left}px; width: 24rem;`;
    }
  }

  $effect(() => {
    if (isOpen) {
      updatePopupPosition();
      window.addEventListener("resize", updatePopupPosition);
      return () => window.removeEventListener("resize", updatePopupPosition);
    }
  });

  function formatTokens(tokens?: number): string {
    if (!tokens) return "";
    if (tokens >= 1000000) return `${Math.round(tokens / 1000000)}M ctx`;
    if (tokens >= 1000) return `${Math.round(tokens / 1000)}k ctx`;
    return `${tokens} ctx`;
  }
</script>

<div class="relative inline-block font-sans select-none">
  <!-- Trigger Button -->
  <button
    bind:this={triggerBtnEl}
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex items-center gap-1 h-7 sm:h-8 px-2 rounded-lg border border-border/70 bg-card/80 hover:bg-muted text-[11px] sm:text-xs text-foreground transition-colors cursor-pointer touch-manipulation"
    aria-expanded={isOpen}
    title="Pilih Model AI Gateway (9Router)"
  >
    <Cpu class="h-3 w-3 sm:h-3.5 sm:w-3.5 text-primary" />
    <span class="font-medium max-w-[80px] sm:max-w-[140px] md:max-w-[180px] truncate">
      {activeModel.name}
    </span>
    {#if activeModel.capabilities?.reasoning || activeModel.tier === "Reasoning"}
      <Sparkles class="h-3 w-3 text-amber-400 shrink-0" />
    {/if}
    {#if activeModel.capabilities?.vision}
      <Eye class="h-3 w-3 text-sky-400 shrink-0" />
    {/if}
    <ChevronDown class="h-3.5 w-3.5 text-muted-foreground ml-0.5 shrink-0" />
  </button>

  {#if isOpen}
    <!-- Backdrop overlay -->
    <button
      type="button"
      class="fixed inset-0 z-40 cursor-default bg-black/40 backdrop-blur-xs w-full h-full border-0 p-0"
      onclick={() => (isOpen = false)}
      aria-label="Tutup Menu Pilihan Model"
    ></button>

    <!-- Dropdown Content (Fixed positioned, unclipped by parent overflow-x) -->
    <div
      style={popupStyle}
      class="fixed z-50 max-w-[420px] rounded-xl border border-border bg-popover text-popover-foreground p-2.5 shadow-2xl flex flex-col gap-2 max-h-[460px] animate-in fade-in zoom-in-95 duration-100"
    >
      <!-- Header with Refresh Button -->
      <div class="flex items-center justify-between px-1">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-foreground">
          <SlidersHorizontal class="h-3.5 w-3.5 text-primary" />
          <span>Model Gateway Tersedia</span>
        </div>
        <button
          type="button"
          onclick={loadModels}
          disabled={isLoading}
          class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors touch-manipulation"
          title="Segarkan Katalog Model"
        >
          <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
        </button>
      </div>

      <!-- Search Input -->
      <div class="relative">
        <Search class="h-3.5 w-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Cari model atau provider..."
          class="w-full pl-8 pr-3 py-1.5 text-xs bg-muted/50 border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
        />
      </div>

      <!-- Category Filter Tabs -->
      <div class="flex items-center gap-1 overflow-x-auto no-scrollbar pb-0.5">
        <button
          type="button"
          onclick={() => (selectedCategory = "all")}
          class="px-2 py-1 text-[11px] rounded-md transition-colors whitespace-nowrap {selectedCategory === 'all' ? 'bg-primary text-primary-foreground font-medium' : 'bg-muted/60 text-muted-foreground hover:text-foreground'}"
        >
          Semua ({models.length})
        </button>
        <button
          type="button"
          onclick={() => (selectedCategory = "combo")}
          class="px-2 py-1 text-[11px] rounded-md transition-colors whitespace-nowrap {selectedCategory === 'combo' ? 'bg-primary text-primary-foreground font-medium' : 'bg-muted/60 text-muted-foreground hover:text-foreground'}"
        >
          Free / Combo
        </button>
        <button
          type="button"
          onclick={() => (selectedCategory = "reasoning")}
          class="px-2 py-1 text-[11px] rounded-md transition-colors whitespace-nowrap {selectedCategory === 'reasoning' ? 'bg-primary text-primary-foreground font-medium' : 'bg-muted/60 text-muted-foreground hover:text-foreground'}"
        >
          Reasoning
        </button>
        <button
          type="button"
          onclick={() => (selectedCategory = "vision")}
          class="px-2 py-1 text-[11px] rounded-md transition-colors whitespace-nowrap {selectedCategory === 'vision' ? 'bg-primary text-primary-foreground font-medium' : 'bg-muted/60 text-muted-foreground hover:text-foreground'}"
        >
          Vision
        </button>
      </div>

      <!-- Models List -->
      <div class="overflow-y-auto max-h-56 space-y-1 pr-0.5">
        {#if filteredModels.length === 0}
          <div class="text-center py-6 text-xs text-muted-foreground">
            Tidak ada model yang cocok dengan pencarian
          </div>
        {:else}
          {#each filteredModels as m (m.id)}
            <button
              type="button"
              disabled={!m.can_access}
              onclick={() => {
                if (m.can_access) {
                  onSelectModel(m.id);
                  isOpen = false;
                }
              }}
              class="w-full flex items-center justify-between p-2 rounded-lg text-left text-xs transition-colors touch-manipulation {m.id === selectedModelId ? 'bg-primary/10 border border-primary/30 text-foreground' : 'hover:bg-muted/70 text-foreground'} {!m.can_access ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}"
            >
              <div class="min-w-0 pr-2 flex-1">
                <div class="flex items-center gap-1.5 flex-wrap">
                  <span class="font-medium [overflow-wrap:anywhere] break-words">{m.name}</span>
                  {#if m.is_default}
                    <span class="text-[9px] px-1.5 py-0.2 rounded bg-primary/20 text-primary font-semibold">DEFAULT</span>
                  {/if}
                </div>

                <div class="flex items-center gap-2 mt-1 text-[10px] text-muted-foreground flex-wrap">
                  <span>{m.provider}</span>
                  {#if m.capabilities?.context_length}
                    <span class="px-1 py-0.2 rounded bg-muted text-muted-foreground font-mono">
                      {formatTokens(m.capabilities.context_length)}
                    </span>
                  {/if}
                  {#if m.capabilities?.reasoning}
                    <span class="flex items-center gap-0.5 text-amber-500 font-medium">
                      <Sparkles class="h-2.5 w-2.5" /> Reasoning
                    </span>
                  {/if}
                  {#if m.capabilities?.vision}
                    <span class="flex items-center gap-0.5 text-sky-500 font-medium">
                      <Eye class="h-2.5 w-2.5" /> Vision
                    </span>
                  {/if}
                  {#if m.capabilities?.tools}
                    <span class="flex items-center gap-0.5 text-emerald-500">
                      <Wrench class="h-2.5 w-2.5" /> Tools
                    </span>
                  {/if}
                </div>
              </div>

              {#if m.id === selectedModelId}
                <Check class="h-4 w-4 text-primary shrink-0" />
              {:else if !m.can_access}
                <span title="Akses dibatasi hak peran" class="shrink-0 flex items-center">
                  <Lock class="h-3.5 w-3.5 text-muted-foreground" />
                </span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <!-- Custom Model ID Accordion / Bottom Option -->
      <div class="pt-1.5 border-t border-border/60">
        {#if !showCustomInput}
          <button
            type="button"
            onclick={() => (showCustomInput = true)}
            class="w-full flex items-center justify-center gap-1.5 py-1.5 px-2 text-[11px] rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors touch-manipulation"
          >
            <Plus class="h-3 w-3" />
            <span>Ketik Model Kustom Manual...</span>
          </button>
        {:else}
          <div class="flex items-center gap-1.5">
            <input
              type="text"
              bind:value={customModelInput}
              placeholder="misal: cbai/deepseek-v3-2-volc"
              class="flex-1 px-2.5 py-1 text-xs bg-muted/60 border border-border rounded-lg text-foreground focus:outline-hidden focus:ring-1 focus:ring-primary"
              onkeydown={(e) => e.key === "Enter" && applyCustomModel()}
            />
            <button
              type="button"
              onclick={applyCustomModel}
              class="px-2.5 py-1 bg-primary text-primary-foreground text-xs rounded-lg font-medium hover:bg-primary/90 transition-colors touch-manipulation"
            >
              Terapkan
            </button>
            <button
              type="button"
              onclick={() => (showCustomInput = false)}
              class="px-1.5 py-1 text-xs text-muted-foreground hover:text-foreground"
            >
              Batal
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
