<script lang="ts">
  import { onMount } from "svelte";
  import Server from "lucide-svelte/icons/server";
  import Plus from "lucide-svelte/icons/plus";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Wrench from "lucide-svelte/icons/wrench";
  import Eye from "lucide-svelte/icons/eye";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { mcpApi } from "$lib/mcpApi";
  import type { McpServerItem, McpToolItem } from "$lib/types";
  import AdminMcpServerCreateModal from "./AdminMcpServerCreateModal.svelte";

  let servers = $state<McpServerItem[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);
  let showCreateModal = $state(false);
  let activeDetailsServer = $state<McpServerItem | null>(null);
  let serverTools = $state<McpToolItem[]>([]);
  let isLoadingTools = $state(false);

  async function loadServers() {
    isLoading = true;
    errorMessage = null;
    try {
      const res = await mcpApi.listServers();
      servers = res.servers;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat server MCP";
    } finally {
      isLoading = false;
    }
  }

  async function handleDeleteServer(id: string, title: string) {
    if (!confirm(`Yakin ingin menghapus server "${title}"? Seluruh tool terdaftar akan dicopot.`)) {
      return;
    }
    try {
      await mcpApi.deleteServer(id);
      successMessage = `Server "${title}" berhasil dihapus.`;
      await loadServers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menghapus server";
    }
  }

  async function viewServerDetails(srv: McpServerItem) {
    activeDetailsServer = srv;
    isLoadingTools = true;
    try {
      const res = await mcpApi.getServer(srv.id);
      serverTools = res.tools;
    } catch (e) {
      serverTools = [];
    } finally {
      isLoadingTools = false;
    }
  }

  onMount(() => {
    loadServers();
  });
</script>

<div class="space-y-4 text-xs font-sans">
  <!-- Header Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-xl border border-border bg-card">
    <div class="space-y-1 min-w-0">
      <div class="font-semibold text-sm text-foreground flex items-center gap-2">
        <Server class="h-4 w-4 text-primary shrink-0" />
        <span class="[overflow-wrap:anywhere] break-words">Manajemen Integrasi Model Context Protocol (MCP 2.1)</span>
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Kelola server MCP enterprise, penegakan kebijakan CISO (Mandatory), dan otorisasi OAuth 2.1 PKCE.
      </div>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <Button variant="outline" size="sm" onclick={loadServers} disabled={isLoading} class="h-8 gap-1.5 min-h-[44px] sm:min-h-[32px]">
        <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
        <span>Segarkan</span>
      </Button>
      <Button size="sm" onclick={() => showCreateModal = true} class="h-8 gap-1.5 bg-primary text-primary-foreground min-h-[44px] sm:min-h-[32px]">
        <Plus class="h-3.5 w-3.5" />
        <span>Tambah Server</span>
      </Button>
    </div>
  </div>

  {#if successMessage}
    <div class="p-3 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2">
      <CheckCircle class="h-4 w-4 shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">{successMessage}</span>
    </div>
  {/if}

  {#if errorMessage}
    <div class="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
      <AlertTriangle class="h-4 w-4 shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">{errorMessage}</span>
    </div>
  {/if}

  <!-- Server Cards List -->
  {#if isLoading && servers.length === 0}
    <div class="flex items-center justify-center p-8 text-muted-foreground gap-2">
      <Loader2 class="h-4 w-4 animate-spin text-primary" />
      <span>Memuat server MCP terdaftar...</span>
    </div>
  {:else if servers.length === 0}
    <div class="p-8 text-center rounded-xl border border-dashed border-border bg-muted/20 text-muted-foreground space-y-2">
      <Server class="h-8 w-8 mx-auto opacity-50" />
      <div class="font-medium text-xs">Belum ada server MCP yang terdaftar.</div>
      <div class="text-[11px]">Daftarkan server eksternal untuk memperluas kapabilitas agen AI.</div>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-2.5">
      {#each servers as srv (srv.id)}
        <div class="p-3.5 rounded-xl border border-border bg-card hover:border-primary/40 transition-colors">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div class="flex items-start sm:items-center gap-3 min-w-0">
              <div class="p-2 rounded-lg bg-muted text-primary shrink-0 mt-0.5 sm:mt-0">
                <Server class="h-4 w-4" />
              </div>
              <div class="min-w-0">
                <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words flex items-center gap-2 flex-wrap">
                  <span>{srv.display_title}</span>
                  <Badge variant="outline" class="font-mono text-[9px] uppercase px-1.5 py-0">
                    {srv.transport_type}
                  </Badge>
                  {#if srv.is_mandatory}
                    <Badge variant="default" class="bg-amber-600/30 text-amber-300 border-amber-600/50 text-[9px] px-1.5 py-0 flex items-center gap-1">
                      <ShieldAlert class="h-2.5 w-2.5" />
                      <span>CISO Mandatory</span>
                    </Badge>
                  {/if}
                  {#if srv.auth_type === "oauth2"}
                    <Badge variant="outline" class="text-sky-400 border-sky-800/40 text-[9px] px-1.5 py-0">OAuth 2.1</Badge>
                  {/if}
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

  <AdminMcpServerCreateModal
    open={showCreateModal}
    onClose={() => showCreateModal = false}
    onCreated={loadServers}
  />

  {#if activeDetailsServer}
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-xs">
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
