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

  export interface ProviderConfig {
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
      isActive: true
    }
  ]);

  let showKeyMap = $state<Record<string, boolean>>({});
  let pingStatusMap = $state<Record<string, "idle" | "testing" | "success" | "error">>({});

  let newProvider = $state({
    name: "",
    baseUrl: "",
    apiKey: "",
    defaultModel: ""
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
        isActive: false
      }
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
        headers: p.apiKey ? { Authorization: `Bearer ${p.apiKey}` } : {}
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
      <Server class="w-4 h-4 text-cyan-400" />
      <span>Manajemen Provider LLM & Kunci API</span>
    </h3>
    <p class="text-muted-foreground text-[11px] mt-0.5">
      Konfigurasi endpoint gateway AI kompatibel OpenAI (9router, OpenRouter, Ollama, vLLM).
    </p>
  </div>

  <!-- Provider List -->
  <div class="space-y-3">
    {#each providers as prov (prov.id)}
      <div class="p-3.5 rounded-xl bg-background/80 border border-border space-y-2.5">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="font-semibold text-foreground">{prov.name}</span>
            {#if prov.isActive}
              <Badge variant="outline" class="text-[10px] bg-emerald-500/10 border-emerald-500/30 text-emerald-400">
                Aktif Default
              </Badge>
            {/if}
          </div>
          <div class="flex items-center gap-1.5">
            <Button
              variant="outline"
              size="sm"
              class="h-7 px-2 text-[11px] gap-1 cursor-pointer"
              onclick={() => handleTestConnection(prov)}
              disabled={pingStatusMap[prov.id] === "testing"}
            >
              {#if pingStatusMap[prov.id] === "testing"}
                <Loader2 class="w-3 h-3 animate-spin text-cyan-400" />
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
                class="p-1 rounded text-muted-foreground hover:text-destructive transition-colors cursor-pointer"
                onclick={() => handleRemoveProvider(prov.id)}
                aria-label="Hapus provider {prov.name}"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            {/if}
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px]">
          <div>
            <span class="text-muted-foreground block text-[10px]">Base URL</span>
            <span class="font-mono text-foreground truncate block">{prov.baseUrl}</span>
          </div>
          <div>
            <span class="text-muted-foreground block text-[10px]">Model Default</span>
            <span class="font-mono text-foreground">{prov.defaultModel}</span>
          </div>
        </div>

        <!-- API Key masked row -->
        <div class="flex items-center gap-2 pt-1 border-t border-border/40">
          <Key class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <span class="font-mono text-[11px] flex-1 text-foreground">
            {showKeyMap[prov.id] ? prov.apiKey : "••••••••••••••••••••••••"}
          </span>
          <button
            type="button"
            class="p-1 text-muted-foreground hover:text-foreground cursor-pointer"
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
  <div class="p-3.5 rounded-xl border border-dashed border-border/80 space-y-3 bg-muted/20">
    <div class="font-semibold text-foreground text-xs flex items-center gap-1.5">
      <Plus class="w-3.5 h-3.5 text-cyan-400" />
      <span>Tambah Provider Kustom</span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
      <div>
        <label for="prov-name" class="block text-muted-foreground text-[10px] mb-1">Nama Provider</label>
        <input
          id="prov-name"
          type="text"
          bind:value={newProvider.name}
          placeholder="cth: Local vLLM Server"
          class="w-full px-2.5 py-1.5 rounded-lg bg-background border border-border text-foreground text-xs focus:outline-hidden"
        />
      </div>
      <div>
        <label for="prov-url" class="block text-muted-foreground text-[10px] mb-1">Base URL (/v1)</label>
        <input
          id="prov-url"
          type="text"
          bind:value={newProvider.baseUrl}
          placeholder="http://localhost:8000/v1"
          class="w-full px-2.5 py-1.5 rounded-lg bg-background border border-border text-foreground text-xs font-mono focus:outline-hidden"
        />
      </div>
      <div>
        <label for="prov-key" class="block text-muted-foreground text-[10px] mb-1">API Key (Opsional)</label>
        <input
          id="prov-key"
          type="password"
          bind:value={newProvider.apiKey}
          placeholder="sk-..."
          class="w-full px-2.5 py-1.5 rounded-lg bg-background border border-border text-foreground text-xs font-mono focus:outline-hidden"
        />
      </div>
      <div>
        <label for="prov-mod" class="block text-muted-foreground text-[10px] mb-1">Default Model ID</label>
        <input
          id="prov-mod"
          type="text"
          bind:value={newProvider.defaultModel}
          placeholder="qwen2.5-coder:7b"
          class="w-full px-2.5 py-1.5 rounded-lg bg-background border border-border text-foreground text-xs font-mono focus:outline-hidden"
        />
      </div>
    </div>

    <div class="flex justify-end pt-1">
      <Button
        size="sm"
        class="bg-cyan-600 hover:bg-cyan-500 text-white cursor-pointer gap-1.5"
        onclick={handleAddProvider}
        disabled={!newProvider.name.trim() || !newProvider.baseUrl.trim()}
      >
        <Plus class="w-3.5 h-3.5" />
        <span>Simpan Provider</span>
      </Button>
    </div>
  </div>
</div>
