<script lang="ts">
  import { onMount } from "svelte";
  import Server from "lucide-svelte/icons/server";
  import Plus from "lucide-svelte/icons/plus";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import Lock from "lucide-svelte/icons/lock";
  import KeyRound from "lucide-svelte/icons/key-round";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { mcpApi } from "$lib/mcpApi";
  import type { McpUserPreferenceItem } from "$lib/types";
  import UserPersonalMcpModal from "./UserPersonalMcpModal.svelte";
  import UserOAuthConnectModal from "./UserOAuthConnectModal.svelte";

  let servers = $state<McpUserPreferenceItem[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  let showPersonalModal = $state(false);
  let activeOAuthServer = $state<McpUserPreferenceItem | null>(null);

  async function loadPreferences() {
    isLoading = true;
    errorMessage = null;
    try {
      const res = await mcpApi.listUserPreferences();
      servers = res.servers;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat preferensi MCP";
    } finally {
      isLoading = false;
    }
  }

  async function handleToggle(srv: McpUserPreferenceItem) {
    if (srv.is_mandatory) return; // CISO Invariant: Tidak bisa dimatikan
    const nextState = !srv.is_enabled;
    try {
      await mcpApi.togglePreference(srv.id, nextState);
      srv.is_enabled = nextState;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memperbarui status server";
      await loadPreferences();
    }
  }

  async function handleStartOAuth(srv: McpUserPreferenceItem) {
    activeOAuthServer = srv;
    try {
      const auth = await mcpApi.authorizeOAuth(srv.id, window.location.origin + "/api/mcp/callback");
      window.open(auth.authorization_url, "_blank");
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memulai otorisasi OAuth";
    }
  }

  async function handleRevokeOAuth(srv: McpUserPreferenceItem) {
    if (!confirm(`Putuskan koneksi OAuth untuk "${srv.display_title}"?`)) return;
    try {
      await mcpApi.revokeOAuthToken(srv.id);
      successMessage = `Koneksi OAuth untuk ${srv.display_title} telah diputuskan.`;
      await loadPreferences();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memutuskan koneksi OAuth";
    }
  }

  async function handleDeletePersonalServer(id: string, title: string) {
    if (!confirm(`Hapus server pribadi "${title}"?`)) return;
    try {
      await mcpApi.deleteUserServer(id);
      successMessage = `Server pribadi "${title}" dihapus.`;
      await loadPreferences();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menghapus server pribadi";
    }
  }

  onMount(() => {
    loadPreferences();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  <!-- Banner Overview -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 rounded-xl border border-border bg-card">
    <div class="space-y-1 min-w-0">
      <div class="font-semibold text-sm text-foreground flex items-center gap-2">
        <Server class="h-4 w-4 text-primary shrink-0" />
        <span class="[overflow-wrap:anywhere] break-words">Integrasi Tools & Server MCP</span>
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Aktifkan atau nonaktifkan server MCP untuk percakapan AI Anda, serta kelola otorisasi akun OAuth 2.1.
      </div>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <Button variant="outline" size="sm" onclick={loadPreferences} disabled={isLoading} class="h-8 gap-1.5 min-h-[44px] sm:min-h-[32px]">
        <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
        <span>Segarkan</span>
      </Button>
      <Button size="sm" onclick={() => showPersonalModal = true} class="h-8 gap-1.5 bg-primary text-primary-foreground min-h-[44px] sm:min-h-[32px]">
        <Plus class="h-3.5 w-3.5" />
        <span>Tambah Pribadi</span>
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

  <!-- List Cards -->
  {#if isLoading && servers.length === 0}
    <div class="flex items-center justify-center p-8 text-muted-foreground gap-2">
      <Loader2 class="h-4 w-4 animate-spin text-primary" />
      <span>Memuat katalog server MCP...</span>
    </div>
  {:else if servers.length === 0}
    <div class="p-8 text-center rounded-xl border border-dashed border-border bg-muted/20 text-muted-foreground space-y-2">
      <Server class="h-8 w-8 mx-auto opacity-50" />
      <div class="font-medium text-xs">Tidak ada server MCP yang tersedia untuk profil Anda.</div>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-2.5">
      {#each servers as srv (srv.id)}
        <div class="p-3.5 rounded-xl border border-border bg-card transition-colors {srv.is_enabled ? 'border-border' : 'opacity-70 bg-muted/10'}">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div class="flex items-start sm:items-center gap-3 min-w-0">
              <div class="p-2 rounded-lg bg-muted text-primary shrink-0 mt-0.5 sm:mt-0">
                <Server class="h-4 w-4" />
              </div>
              <div class="min-w-0 space-y-0.5">
                <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words flex items-center gap-2 flex-wrap">
                  <span>{srv.display_title}</span>
                  <Badge variant="outline" class="font-mono text-[9px] uppercase px-1.5 py-0">
                    {srv.transport_type}
                  </Badge>
                  {#if srv.is_mandatory}
                    <Badge variant="default" class="bg-amber-600/30 text-amber-300 border-amber-600/50 text-[9px] px-1.5 py-0 flex items-center gap-1">
                      <Lock class="h-2.5 w-2.5" />
                      <span>Wajib Organisasi</span>
                    </Badge>
                  {/if}
                  {#if srv.scope_type === "user"}
                    <Badge variant="secondary" class="text-[9px] px-1.5 py-0">Server Pribadi</Badge>
                  {/if}
                  {#if srv.auth_type === "oauth2"}
                    {#if srv.is_oauth_connected}
                      <Badge variant="outline" class="text-emerald-400 border-emerald-800/40 text-[9px] px-1.5 py-0 flex items-center gap-1">
                        <ShieldCheck class="h-2.5 w-2.5" />
                        <span>OAuth Terkoneksi</span>
                      </Badge>
                    {:else}
                      <Badge variant="outline" class="text-amber-400 border-amber-800/40 text-[9px] px-1.5 py-0">
                        Perlu Login OAuth
                      </Badge>
                    {/if}
                  {/if}
                  <Badge variant="outline" class="text-purple-400 border-purple-800/40 text-[9px] px-1.5 py-0 font-mono">
                    {srv.quota_used_today || 0}/{srv.daily_request_limit || 1000} req ({srv.rate_limit_rpm || 60} RPM)
                  </Badge>
                </div>
                {#if srv.description}
                  <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">{srv.description}</div>
                {/if}
              </div>
            </div>

            <!-- Action Controls -->
            <div class="flex items-center gap-2 shrink-0 self-end sm:self-auto">
              {#if srv.auth_type === "oauth2"}
                {#if srv.is_oauth_connected}
                  <Button variant="outline" size="sm" onclick={() => handleRevokeOAuth(srv)} class="h-8 text-xs min-h-[44px] sm:min-h-[32px]">
                    Putuskan
                  </Button>
                {:else}
                  <Button variant="default" size="sm" onclick={() => handleStartOAuth(srv)} class="h-8 gap-1.5 text-xs bg-sky-600 hover:bg-sky-500 text-white min-h-[44px] sm:min-h-[32px]">
                    <KeyRound class="h-3.5 w-3.5" />
                    <span>Hubungkan OAuth</span>
                  </Button>
                {/if}
              {/if}

              {#if srv.scope_type === "user"}
                <Button variant="ghost" size="sm" onclick={() => handleDeletePersonalServer(srv.id, srv.display_title)} class="h-8 text-rose-400 min-h-[44px] sm:min-h-[32px]">
                  <Trash2 class="h-3.5 w-3.5" />
                </Button>
              {/if}

              <!-- Toggle Enable / Disable Switch -->
              <div class="flex items-center gap-1.5 pl-2 border-l border-border">
                {#if srv.is_mandatory}
                  <span class="text-[10px] text-muted-foreground font-mono">Terkunci Aktif</span>
                  <input type="checkbox" checked={true} disabled={true} class="h-4 w-4 rounded opacity-60 cursor-not-allowed" />
                {:else}
                  <span class="text-[10px] text-muted-foreground font-mono">{srv.is_enabled ? "Aktif" : "Nonaktif"}</span>
                  <input
                    type="checkbox"
                    checked={srv.is_enabled}
                    onchange={() => handleToggle(srv)}
                    class="h-4 w-4 rounded accent-primary cursor-pointer"
                  />
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <UserPersonalMcpModal
    open={showPersonalModal}
    onClose={() => showPersonalModal = false}
    onCreated={loadPreferences}
  />

  <UserOAuthConnectModal
    server={activeOAuthServer}
    onClose={() => activeOAuthServer = null}
    onConnected={loadPreferences}
  />
</div>
