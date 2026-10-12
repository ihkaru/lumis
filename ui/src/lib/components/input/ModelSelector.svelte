<script lang="ts">
  import Cpu from "lucide-svelte/icons/cpu";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import Check from "lucide-svelte/icons/check";
  import Sparkles from "lucide-svelte/icons/sparkles";

  interface ModelOption {
    id: string;
    name: string;
    provider: string;
    tier: string;
    isReasoning?: boolean;
  }

  const AVAILABLE_MODELS: ModelOption[] = [
    {
      id: "my9model-free",
      name: "Standard Free (9router)",
      provider: "Decolua Gateway",
      tier: "Basic",
    },
    {
      id: "nvidia/nemotron-3-ultra-550b",
      name: "Nemotron 3 Ultra 550B",
      provider: "NVIDIA NIM",
      tier: "Reasoning",
      isReasoning: true,
    },
    {
      id: "deepseek-ai/deepseek-r1",
      name: "DeepSeek R1 (Chain-of-Thought)",
      provider: "DeepSeek",
      tier: "Reasoning",
      isReasoning: true,
    },
    {
      id: "meta-llama/llama-3.3-70b-instruct",
      name: "Llama 3.3 70B Instruct",
      provider: "Meta",
      tier: "Standard",
    },
  ];

  interface Props {
    selectedModelId: string;
    onSelectModel: (modelId: string) => void;
  }

  let { selectedModelId = "my9model-free", onSelectModel }: Props = $props();

  let isOpen = $state(false);

  let activeModel = $derived(
    AVAILABLE_MODELS.find((m) => m.id === selectedModelId) || AVAILABLE_MODELS[0]
  );
</script>

<div class="relative inline-block font-sans select-none">
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="flex items-center gap-1.5 h-8 px-2.5 rounded-lg border border-border bg-card/80 hover:bg-muted text-xs text-foreground transition-colors cursor-pointer"
    aria-expanded={isOpen}
    title="Pilih Model AI"
  >
    <Cpu class="h-3.5 w-3.5 text-cyan-400" />
    <span class="font-medium max-w-[120px] sm:max-w-[160px] truncate">{activeModel.name}</span>
    {#if activeModel.isReasoning}
      <Sparkles class="h-3 w-3 text-cyan-400" />
    {/if}
    <ChevronDown class="h-3.5 w-3.5 text-muted-foreground ml-0.5" />
  </button>

  {#if isOpen}
    <button
      type="button"
      class="fixed inset-0 z-30 cursor-default bg-transparent w-full h-full border-0 p-0"
      onclick={() => (isOpen = false)}
      aria-label="Close Model Dropdown"
    ></button>

    <div class="absolute bottom-full left-0 mb-1.5 z-40 w-72 rounded-xl border border-border bg-popover p-1.5 shadow-xl space-y-1">
      <div class="px-2.5 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
        Model Inferensi Tersedia
      </div>

      {#each AVAILABLE_MODELS as m}
        <button
          type="button"
          onclick={() => {
            onSelectModel(m.id);
            isOpen = false;
          }}
          class="flex w-full items-center justify-between p-2 rounded-lg text-left text-xs hover:bg-muted/60 transition-colors {m.id === selectedModelId ? 'bg-cyan-950/40 text-cyan-400' : 'text-foreground'}"
        >
          <div class="min-w-0 pr-2">
            <div class="font-medium truncate flex items-center gap-1.5">
              <span>{m.name}</span>
              {#if m.isReasoning}
                <Sparkles class="h-3 w-3 text-cyan-400 shrink-0" />
              {/if}
            </div>
            <div class="text-[10px] text-muted-foreground">
              {m.provider} • {m.tier}
            </div>
          </div>
          {#if m.id === selectedModelId}
            <Check class="h-4 w-4 text-cyan-400 shrink-0" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
