<script lang="ts">
  import { onMount } from "svelte";
  import Server from "lucide-svelte/icons/server";
  import Plus from "lucide-svelte/icons/plus";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Wrench from "lucide-svelte/icons/wrench";
  import Eye from "lucide-svelte/icons/eye";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { api } from "$lib/api";
  import type { McpServerItem, McpToolItem } from "$lib/types";

  let servers = $state<McpServerItem[]>([]);
  let isLoading = $state(true);
  let isTesting = $state(false);
  let isSaving = $state(false);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);
  let showCreateModal = $state(false);
  let activeDetailsServer = $state<McpServerItem | null>(null);
  let serverTools = $state<McpToolItem[]>([]);
  let isLoadingTools = $state(false);

  // Form state
  let formName = $state("");
  let formTitle = $state("");
  let formDesc = $state("");
  let formUrl = $state("");
  let formTransport = $state<"sse" | "streamable_http">("sse");
  let formAuth = $state<"none" | "bearer" | "basic">("bearer");
  let formCred = $state("");
  let pingResult = $state<{ success: boolean; count: number } | null>(null);

  async function loadServers() {
    isLoading = true;
    errorMessage = null;
    try {
      const res = await api.listMcpServers();
      servers = res.servers;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat server MCP";
    } finally {
      isLoading = false;
    }
  }

  async function handleTestEndpoint() {
    if (!formUrl) {
      errorMessage = "URL endpoint wajib diisi untuk pengujian";
      return;
    }
    isTesting = true;
    errorMessage = null;
    pingResult = null;
    try {
      const res = await api.testMcpEndpoint({
        endpoint_url: formUrl,
        auth_type: formAuth,
        raw_credentials: formCred || undefined,
      });
      pingResult = res;
      successMessage = `Uji koneksi berhasil! Ditemukan ${res.count} tool.`;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Uji koneksi gagal";
    } finally {
      isTesting = false;
    }
  }

  async function handleCreateServer() {
    if (!formName || !formTitle || !formUrl) {
      errorMessage = "Nama, Judul, dan URL Endpoint wajib diisi";
      return;
    }
    isSaving = true;
    errorMessage = null;
    successMessage = null;
    try {
      const res = await api.createMcpServer({
        name: formName,
        display_title: formTitle,
        description: formDesc || undefined,
        transport_type: formTransport,
        endpoint_url: formUrl,
        auth_type: formAuth,
        raw_credentials: formCred || undefined,
        risk_tier: 1,
      });
      successMessage = `Server '${res.server.display_title}' berhasil didaftarkan dengan ${res.tools_count} tool.`;
      showCreateModal = false;
      resetForm();
      await loadServers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal mendaftarkan server";
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteServer(id: string, title: string) {
    if (!confirm(`Hapus konfigurasi server MCP '${title}'? Seluruh tool terkait akan dicopot.`)) return;
    try {
      await api.deleteMcpServer(id);
      successMessage = `Server '${title}' berhasil dihapus.`;
      await loadServers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menghapus server";
    }
  }

  async function viewServerDetails(srv: McpServerItem) {
    activeDetailsServer = srv;
    isLoadingTools = true;
    try {
      const res = await api.getMcpServer(srv.id);
      serverTools = res.tools;
    } catch {
      serverTools = [];
    } finally {
      isLoadingTools = false;
    }
  }

  function resetForm() {
    formName = "";
    formTitle = "";
    formDesc = "";
    formUrl = "";
    formCred = "";
    pingResult = null;
  }

  onMount(() => {
    loadServers();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  <div class="flex items-center justify-between gap-2">
    <div>
      <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
        Server Model Context Protocol (MCP Gateway)
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Integrasi alat eksternal enterprise dengan perlindungan Anti-SSRF dan enkripsi rahasia AES-256-GCM.
      </div>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" onclick={loadServers} disabled={isLoading} class="h-8 gap-1.5 min-h-[44px] sm:min-h-[32px]">
        <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
        <span class="hidden sm:inline">Segarkan</span>
      </Button>
      <Button size="sm" onclick={() => { showCreateModal = true; errorMessage = null; }} class="h-8 gap-1.5 bg-primary min-h-[44px] sm:min-h-[32px]">
        <Plus class="h-3.5 w-3.5" />
        <span>Tambah Server</span>
      </Button>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-3 rounded-lg bg-rose-950/40 border border-rose-800/40 text-rose-300 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <AlertTriangle class="h-4 w-4 shrink-0 text-rose-400" />
      <span>{errorMessage}</span>
    </div>
  {/if}

  {#if successMessage}
    <div class="p-3 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-300 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <CheckCircle class="h-4 w-4 shrink-0 text-emerald-400" />
      <span>{successMessage}</span>
    </div>
  {/if}

  {#if isLoading}
    <div class="flex flex-col items-center justify-center p-8 text-muted-foreground gap-2">
      <Loader2 class="h-6 w-6 animate-spin text-primary" />
      <span class="text-xs">Memuat daftar server MCP...</span>
    </div>
  {:else if servers.length === 0}
    <div class="p-8 text-center rounded-xl border border-dashed border-border bg-card/40 space-y-2">
      <Server class="h-8 w-8 text-muted-foreground mx-auto stroke-[1.5]" />
      <div class="font-medium text-xs text-foreground">Belum Ada Server MCP Terdaftar</div>
      <div class="text-[11px] text-muted-foreground max-w-sm mx-auto">
        Daftarkan server eksternal untuk memperluas kapabilitas agen dengan basis data atau API enterprise.
      </div>
    </div>
  {:else}
    <div class="grid gap-3">
      {#each servers as srv}
        <div class="p-3.5 rounded-xl border border-border bg-card hover:border-border/80 transition-colors space-y-2.5">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <div class="p-2 rounded-lg bg-primary/10 text-primary border border-primary/20 shrink-0">
                <Server class="h-4 w-4" />
              </div>
              <div class="min-w-0">
                <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words flex items-center gap-2">
                  <span>{srv.display_title}</span>
                  <Badge variant="outline" class="font-mono text-[9px] uppercase px-1.5 py-0">
                    {srv.transport_type}
                  </Badge>
                </div>
                <div class="text-[11px] text-muted-foreground font-mono truncate">
                  {srv.endpoint_url || "stdio process"}
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1.5 shrink-0 self-end sm:self-auto">
              <Button variant="ghost" size="sm" onclick={() => viewServerDetails(srv)} class="h-8 gap-1 text-[11px] min-h-[44px] sm:min-h-[32px]">
                <Eye class="h-3.5 w-3.5" />
                <span>Detail</span>
              </Button>
              <Button variant="ghost" size="sm" onclick={() => handleDeleteServer(srv.id, srv.display_title)} class="h-8 text-rose-400 hover:text-rose-300 hover:bg-rose-950/30 min-h-[44px] sm:min-h-[32px]">
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if showCreateModal}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs">
      <div class="bg-card border border-border rounded-2xl w-full max-w-lg p-5 space-y-4 shadow-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between">
          <div class="font-semibold text-sm text-foreground flex items-center gap-2">
            <Server class="h-4 w-4 text-primary" />
            <span>Tambah Server MCP Baru</span>
          </div>
          <button type="button" onclick={() => showCreateModal = false} class="text-muted-foreground hover:text-foreground p-1 rounded-md" aria-label="Tutup modal">
            <X class="h-4 w-4" />
          </button>
        </div>

        <div class="space-y-3">
          <div>
            <label for="mcp-srv-name" class="block text-[11px] font-medium text-foreground mb-1">Nama Identifier (Slug)</label>
            <input id="mcp-srv-name" type="text" bind:value={formName} placeholder="misal: database_ledger" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
          </div>
          <div>
            <label for="mcp-srv-title" class="block text-[11px] font-medium text-foreground mb-1">Judul Tampilan</label>
            <input id="mcp-srv-title" type="text" bind:value={formTitle} placeholder="misal: General Ledger System" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
          </div>
          <div>
            <label for="mcp-srv-url" class="block text-[11px] font-medium text-foreground mb-1">Endpoint URL (SSE / Streamable HTTP)</label>
            <input id="mcp-srv-url" type="url" bind:value={formUrl} placeholder="https://mcp.enterprise.internal/sse" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs font-mono" />
          </div>
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label for="mcp-srv-transport" class="block text-[11px] font-medium text-foreground mb-1">Transport</label>
              <select id="mcp-srv-transport" bind:value={formTransport} class="w-full h-9 px-2 rounded-lg border border-border bg-background text-xs">
                <option value="sse">Server-Sent Events (SSE)</option>
                <option value="streamable_http">Streamable HTTP</option>
              </select>
            </div>
            <div>
              <label for="mcp-srv-auth" class="block text-[11px] font-medium text-foreground mb-1">Autentikasi</label>
              <select id="mcp-srv-auth" bind:value={formAuth} class="w-full h-9 px-2 rounded-lg border border-border bg-background text-xs">
                <option value="bearer">Bearer Token</option>
                <option value="basic">Basic Auth</option>
                <option value="none">Tanpa Autentikasi</option>
              </select>
            </div>
          </div>
          {#if formAuth !== "none"}
            <div>
              <label for="mcp-srv-cred" class="block text-[11px] font-medium text-foreground mb-1">Kredensial / Token Rahasia</label>
              <input id="mcp-srv-cred" type="password" bind:value={formCred} placeholder="Dienkripsi dengan AES-256-GCM" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs font-mono" />
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-between pt-2 border-t border-border">
          <Button type="button" variant="outline" size="sm" onclick={handleTestEndpoint} disabled={isTesting} class="h-9 gap-1.5 min-h-[44px] sm:min-h-[36px]">
            <ShieldCheck class="h-3.5 w-3.5" />
            <span>{isTesting ? "Menguji..." : "Uji Koneksi"}</span>
          </Button>
          <div class="flex items-center gap-2">
            <Button type="button" variant="ghost" size="sm" onclick={() => showCreateModal = false} class="h-9 min-h-[44px] sm:min-h-[36px]">Batal</Button>
            <Button type="button" size="sm" onclick={handleCreateServer} disabled={isSaving} class="h-9 bg-primary min-h-[44px] sm:min-h-[36px]">
              {isSaving ? "Menyimpan..." : "Daftarkan Server"}
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if activeDetailsServer}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs">
      <div class="bg-card border border-border rounded-2xl w-full max-w-lg p-5 space-y-4 shadow-2xl max-h-[85vh] overflow-y-auto">
        <div class="flex items-center justify-between">
          <div class="font-semibold text-sm text-foreground flex items-center gap-2">
            <Wrench class="h-4 w-4 text-primary" />
            <span>Tool Terpasang: {activeDetailsServer.display_title}</span>
          </div>
          <button type="button" onclick={() => activeDetailsServer = null} class="text-muted-foreground hover:text-foreground p-1 rounded-md" aria-label="Tutup modal">
            <X class="h-4 w-4" />
          </button>
        </div>

        {#if isLoadingTools}
          <div class="flex items-center justify-center p-6 text-muted-foreground gap-2">
            <Loader2 class="h-4 w-4 animate-spin text-primary" />
            <span>Memuat katalog tool...</span>
          </div>
        {:else if serverTools.length === 0}
          <div class="p-6 text-center text-muted-foreground text-xs">Tidak ada tool aktif yang ditemukan pada server ini.</div>
        {:else}
          <div class="space-y-2">
            {#each serverTools as tool}
              <div class="p-3 rounded-lg border border-border bg-background space-y-1">
                <div class="font-mono font-semibold text-xs text-foreground flex items-center justify-between">
                  <span>{tool.name}</span>
                  <Badge variant="outline" class="text-[9px] px-1.5 py-0 text-emerald-400 border-emerald-800/40">Ready</Badge>
                </div>
                {#if tool.description}
                  <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">{tool.description}</div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
