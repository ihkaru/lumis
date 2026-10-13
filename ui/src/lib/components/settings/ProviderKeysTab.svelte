<script lang="ts">
  import Key from "lucide-svelte/icons/key";
  import Server from "lucide-svelte/icons/server";
  import Plus from "lucide-svelte/icons/plus";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import Eye from "lucide-svelte/icons/eye";
  import EyeOff from "lucide-svelte/icons/eye-off";
  import CheckCircle2 from "lucide-svelte/icons/check-circle-2";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";

  interface ProviderConfig {
    id: string;
    name: string;
    baseUrl: string;
    apiKey: string;
    defaultModel: string;
    isActive: boolean;
  }

  let providers = $state<ProviderConfig[]>([
    {
      id: "prov-default",
      name: "Decolua 9router (Coolify Gateway)",
      baseUrl: "https://router.dvlpid.my.id/v1",
      apiKey: "sk-9router-master-key",
      defaultModel: "my9model-free",
      isActive: true,
    },
  ]);

  let showKeyMap = $state<Record<string, boolean>>({});
  let pingStatusMap = $state<Record<string, "idle" | "testing" | "success" | "error">>({});

  let newProvider = $state({
    name: "",
    baseUrl: "",
    apiKey: "",
    defaultModel: "",
  });

  function toggleShowKey(id: string) {
    showKeyMap[id] = !showKeyMap[id];
  }

  function handleAddProvider() {
    if (!newProvider.name.trim() || !newProvider.baseUrl.trim()) return;
    const id = `prov_${Date.now()}`;
    providers = [
      ...providers,
      {
        id,
        name: newProvider.name.trim(),
        baseUrl: newProvider.baseUrl.trim(),
        apiKey: newProvider.apiKey.trim(),
        defaultModel: newProvider.defaultModel.trim() || "default",
        isActive: false,
      },
    ];
    newProvider = { name: "", baseUrl: "", apiKey: "", defaultModel: "" };
  }

  function handleRemoveProvider(id: string) {
    providers = providers.filter((p) => p.id !== id);
  }

  async function handleTestConnection(p: ProviderConfig) {
    pingStatusMap[p.id] = "testing";
    try {
      const res = await fetch(`${p.baseUrl}/models`, {
        headers: p.apiKey ? { Authorization: `Bearer ${p.apiKey}` } : {},
      });
      pingStatusMap[p.id] = res.ok ? "success" : "error";
    } catch {
      pingStatusMap[p.id] = "error";
    }
    setTimeout(() => {
      pingStatusMap[p.id] = "idle";
    }, 4000);
  }
</script>

<div class="space-y-6 font-sans text-xs">
  <div>
    <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
      <Server class="w-4 h-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Manajemen Provider LLM & Kunci API</span>
    </h3>
    <p class="text-muted-foreground text-[11px] mt-0.5 [overflow-wrap:anywhere] break-words">
      Konfigurasi endpoint gateway AI kompatibel OpenAI (9router, OpenRouter, Ollama, vLLM).
    </p>
  </div>

  <!-- Provider List -->
  <div class="space-y-3">
    {#each providers as prov (prov.id)}
      <div class="p-3.5 rounded-xl bg-background/80 border border-border space-y-2.5">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0">
            <span class="font-semibold text-foreground truncate [overflow-wrap:anywhere]">{prov.name}</span>
            {#if prov.isActive}
              <Badge variant="outline" class="text-[10px] bg-emerald-500/10 border-emerald-500/30 text-emerald-400 shrink-0">
                Aktif Default
              </Badge>
            {/if}
          </div>
          <div class="flex items-center gap-1.5 shrink-0 self-start sm:self-auto">
            <Button
              variant="outline"
              size="sm"
              class="h-8 px-2.5 text-[11px] gap-1 cursor-pointer min-h-[44px] sm:min-h-8 touch-manipulation whitespace-nowrap"
              onclick={() => handleTestConnection(prov)}
              disabled={pingStatusMap[prov.id] === "testing"}
            >
              {#if pingStatusMap[prov.id] === "testing"}
                <Loader2 class="w-3 h-3 animate-spin text-primary" />
                <span>Uji...</span>
              {:else if pingStatusMap[prov.id] === "success"}
                <CheckCircle2 class="w-3 h-3 text-emerald-400" />
                <span>Terhubung</span>
              {:else if pingStatusMap[prov.id] === "error"}
                <AlertCircle class="w-3 h-3 text-destructive" />
                <span>Gagal</span>
              {:else}
                <span>Tes Ping</span>
              {/if}
            </Button>
            {#if !prov.isActive}
              <button
                type="button"
                class="p-2 rounded-lg text-muted-foreground hover:text-destructive transition-colors cursor-pointer min-h-[44px] min-w-[44px] flex items-center justify-center touch-manipulation"
                onclick={() => handleRemoveProvider(prov.id)}
                aria-label="Hapus provider {prov.name}"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px]">
          <div class="min-w-0">
            <span class="text-muted-foreground block text-[10px]">Base URL</span>
            <span class="font-mono text-foreground truncate block [overflow-wrap:anywhere] break-all">{prov.baseUrl}</span>
          </div>
          <div class="min-w-0">
            <span class="text-muted-foreground block text-[10px]">Model Default</span>
            <span class="font-mono text-foreground truncate block [overflow-wrap:anywhere]">{prov.defaultModel}</span>
          </div>
        </div>

        <!-- API Key masked row -->
        <div class="flex items-center gap-2 pt-1 border-t border-border/40 min-w-0">
          <Key class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <span class="font-mono text-[11px] flex-1 text-foreground truncate min-w-0 [overflow-wrap:anywhere] break-all">
            {showKeyMap[prov.id] ? prov.apiKey : "••••••••••••••••••••••••"}
          </span>
          <button
            type="button"
            class="p-2 text-muted-foreground hover:text-foreground cursor-pointer min-h-[44px] min-w-[44px] flex items-center justify-center touch-manipulation"
            onclick={() => toggleShowKey(prov.id)}
            aria-label="Toggle tampilkan kunci API"
          >
            {#if showKeyMap[prov.id]}
              <EyeOff class="w-3.5 h-3.5" />
            {:else}
              <Eye class="w-3.5 h-3.5" />
            {/if}
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Add New Provider Form -->
  <div class="p-4 rounded-xl border border-dashed border-border/80 space-y-3 bg-card/60">
    <div class="font-semibold text-foreground text-xs flex items-center gap-1.5">
      <Plus class="w-3.5 h-3.5 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Tambah Provider Kustom</span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div>
        <label for="prov-name" class="block text-muted-foreground text-[10px] mb-1">Nama Provider</label>
        <Input
          id="prov-name"
          type="text"
          bind:value={newProvider.name}
          placeholder="cth: Local vLLM Server"
          class="h-9 text-xs"
        />
      </div>
      <div>
        <label for="prov-url" class="block text-muted-foreground text-[10px] mb-1">Base URL (/v1)</label>
        <Input
          id="prov-url"
          type="text"
          bind:value={newProvider.baseUrl}
          placeholder="http://localhost:8000/v1"
          class="h-9 text-xs font-mono"
        />
      </div>
      <div>
        <label for="prov-key" class="block text-muted-foreground text-[10px] mb-1">API Key (Opsional)</label>
        <Input
          id="prov-key"
          type="password"
          bind:value={newProvider.apiKey}
          placeholder="sk-..."
          class="h-9 text-xs font-mono"
        />
      </div>
      <div>
        <label for="prov-mod" class="block text-muted-foreground text-[10px] mb-1">Default Model ID</label>
        <Input
          id="prov-mod"
          type="text"
          bind:value={newProvider.defaultModel}
          placeholder="qwen2.5-coder:7b"
          class="h-9 text-xs font-mono"
        />
      </div>
    </div>

    <div class="flex justify-end pt-1">
      <Button
        variant="default"
        size="sm"
        class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words"
        onclick={handleAddProvider}
        disabled={!newProvider.name.trim() || !newProvider.baseUrl.trim()}
      >
        <Plus class="w-3.5 h-3.5 shrink-0" />
        <span>Simpan Provider</span>
      </Button>
    </div>
  </div>
</div>
