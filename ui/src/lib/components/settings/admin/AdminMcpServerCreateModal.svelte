<script lang="ts">
  import Server from "lucide-svelte/icons/server";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import ChevronUp from "lucide-svelte/icons/chevron-up";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { mcpApi } from "$lib/mcpApi";

  interface Props {
    open: boolean;
    onClose: () => void;
    onCreated: () => void;
  }

  let { open, onClose, onCreated }: Props = $props();

  let formName = $state("");
  let formTitle = $state("");
  let formDesc = $state("");
  let formUrl = $state("");
  let formTransport = $state<"sse" | "streamable_http">("sse");
  let formAuth = $state<"none" | "bearer" | "basic" | "oauth2">("bearer");
  let formCred = $state("");
  let formIsMandatory = $state(false);

  // OAuth 2.1 fields
  let formOAuthClientId = $state("");
  let formOAuthAuthEndpoint = $state("");
  let formOAuthTokenEndpoint = $state("");
  let formOAuthScopes = $state("");
  let formOAuthResource = $state("");

  // CISO Security Governance fields
  let formSsrfPolicy = $state<"strict" | "custom_allowlist">("strict");
  let formEgressAllowlist = $state("");
  let formSandboxType = $state<"restricted_process" | "docker_ephemeral">("restricted_process");
  let formMaxMemory = $state(256);
  let formTimeoutSecs = $state(30);
  let formDlpEnabled = $state(true);
  let formRateLimitRpm = $state(60);
  let formDailyLimit = $state(1000);
  let formKmsProvider = $state<"local" | "vault" | "aws_kms">("local");
  let showAdvancedSecurity = $state(false);

  let isTesting = $state(false);
  let isSaving = $state(false);
  let errorMessage = $state<string | null>(null);
  let pingResult = $state<{ success: boolean; count: number } | null>(null);

  async function handleTestEndpoint() {
    if (!formUrl) {
      errorMessage = "URL endpoint wajib diisi untuk pengujian";
      return;
    }
    isTesting = true;
    errorMessage = null;
    pingResult = null;
    try {
      const res = await mcpApi.testEndpoint({
        endpoint_url: formUrl,
        auth_type: formAuth,
        raw_credentials: formCred || undefined,
      });
      pingResult = res;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menguji endpoint";
    } finally {
      isTesting = false;
    }
  }

  async function handleSaveServer() {
    if (!formName.trim() || !formTitle.trim()) {
      errorMessage = "Nama identifier dan judul wajib diisi";
      return;
    }
    isSaving = true;
    errorMessage = null;
    try {
      const oauthConfig = formAuth === "oauth2" ? {
        client_id: formOAuthClientId,
        authorization_endpoint: formOAuthAuthEndpoint,
        token_endpoint: formOAuthTokenEndpoint,
        scopes: formOAuthScopes.split(",").map(s => s.trim()).filter(Boolean),
        resource_indicator: formOAuthResource || undefined,
        registration_mode: "pre_registered",
      } : undefined;

      const egressAllowlist = formEgressAllowlist
        ? formEgressAllowlist.split(",").map(s => s.trim()).filter(Boolean)
        : undefined;

      await mcpApi.createServer({
        name: formName.trim(),
        display_title: formTitle.trim(),
        description: formDesc.trim() || undefined,
        transport_type: formTransport,
        endpoint_url: formUrl.trim() || undefined,
        auth_type: formAuth,
        raw_credentials: formCred || undefined,
        is_mandatory: formIsMandatory,
        oauth_config: oauthConfig,
        ssrf_policy: formSsrfPolicy,
        egress_allowlist: egressAllowlist,
        sandbox_type: formSandboxType,
        max_memory_mb: formMaxMemory,
        timeout_seconds: formTimeoutSecs,
        dlp_enabled: formDlpEnabled,
        rate_limit_rpm: formRateLimitRpm,
        daily_request_limit: formDailyLimit,
        kms_provider: formKmsProvider,
      });

      onCreated();
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menyimpan server MCP";
    } finally {
      isSaving = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-xs">
    <div class="bg-card border border-border rounded-2xl w-full max-w-lg p-5 space-y-4 shadow-2xl max-h-[90vh] overflow-y-auto font-sans text-xs">
      <div class="flex items-center justify-between">
        <div class="font-semibold text-sm text-foreground flex items-center gap-2">
          <Server class="h-4 w-4 text-primary" />
          <span>Tambah Server MCP Enterprise</span>
        </div>
        <button type="button" onclick={onClose} class="text-muted-foreground hover:text-foreground p-1 rounded-md cursor-pointer" aria-label="Tutup modal">
          <X class="h-4 w-4" />
        </button>
      </div>

      {#if errorMessage}
        <div class="p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-[11px] flex items-center gap-2">
          <AlertTriangle class="h-4 w-4 shrink-0" />
          <span class="[overflow-wrap:anywhere] break-words">{errorMessage}</span>
        </div>
      {/if}

      <div class="space-y-3">
        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="mcp-create-name" class="block text-[11px] font-medium text-foreground mb-1">Slug Identifier</label>
            <input id="mcp-create-name" type="text" bind:value={formName} placeholder="misal: ledger_db" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
          </div>
          <div>
            <label for="mcp-create-title" class="block text-[11px] font-medium text-foreground mb-1">Judul Tampilan</label>
            <input id="mcp-create-title" type="text" bind:value={formTitle} placeholder="misal: PostgreSQL Ledger" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
          </div>
        </div>

        <div>
          <label for="mcp-create-url" class="block text-[11px] font-medium text-foreground mb-1">Endpoint URL (Anti-SSRF Validated)</label>
          <input id="mcp-create-url" type="text" bind:value={formUrl} placeholder="https://mcp.internal.domain/sse" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
        </div>

        <!-- CISO Mandatory Enforcement Switch -->
        <div class="p-3 rounded-xl border border-amber-800/40 bg-amber-950/20 flex items-center justify-between gap-3">
          <div class="space-y-0.5">
            <div class="font-medium text-xs text-foreground flex items-center gap-1.5">
              <ShieldAlert class="h-3.5 w-3.5 text-amber-400" />
              <span>Wajib oleh Organisasi (CISO Enforced)</span>
            </div>
            <div class="text-[10px] text-muted-foreground">Server wajib aktif untuk seluruh pegawai dan tidak dapat dinonaktifkan.</div>
          </div>
          <input type="checkbox" bind:checked={formIsMandatory} class="h-4 w-4 rounded accent-primary cursor-pointer shrink-0" />
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div>
            <label for="mcp-create-trans" class="block text-[11px] font-medium text-foreground mb-1">Transport</label>
            <select id="mcp-create-trans" bind:value={formTransport} class="w-full h-9 px-2.5 rounded-lg border border-border bg-background text-xs">
              <option value="sse">HTTP Server-Sent Events (SSE)</option>
              <option value="streamable_http">Streamable HTTP POST</option>
            </select>
          </div>
          <div>
            <label for="mcp-create-auth" class="block text-[11px] font-medium text-foreground mb-1">Autentikasi</label>
            <select id="mcp-create-auth" bind:value={formAuth} class="w-full h-9 px-2.5 rounded-lg border border-border bg-background text-xs">
              <option value="bearer">Bearer Token (AES-256-GCM)</option>
              <option value="oauth2">OAuth 2.1 (RFC 6749bis PKCE)</option>
              <option value="basic">HTTP Basic Auth</option>
              <option value="none">Tanpa Autentikasi</option>
            </select>
          </div>
        </div>

        {#if formAuth === "oauth2"}
          <div class="p-3 rounded-xl border border-primary/20 bg-primary/5 space-y-2 text-xs">
            <div class="font-semibold text-[11px] text-primary">Konfigurasi OAuth 2.1 & Resource Indicators</div>
            <input type="text" bind:value={formOAuthClientId} placeholder="Client ID (OAuth 2.1)" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
            <input type="text" bind:value={formOAuthAuthEndpoint} placeholder="Authorization Endpoint URL" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
            <input type="text" bind:value={formOAuthTokenEndpoint} placeholder="Token Endpoint URL" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
            <input type="text" bind:value={formOAuthScopes} placeholder="Scopes (pisahkan koma: read, write)" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
            <input type="text" bind:value={formOAuthResource} placeholder="RFC 8707 Resource URI (opsional)" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
          </div>
        {:else if formAuth !== "none"}
          <div>
            <label for="mcp-create-cred" class="block text-[11px] font-medium text-foreground mb-1">Secret / Token</label>
            <input id="mcp-create-cred" type="password" bind:value={formCred} placeholder="Token rahasia terenkripsi AES-256-GCM" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
          </div>
        {/if}

        <!-- Collapsible CISO Governance Section -->
        <div class="pt-1">
          <button
            type="button"
            onclick={() => (showAdvancedSecurity = !showAdvancedSecurity)}
            class="flex items-center justify-between w-full p-2.5 rounded-xl border border-border bg-muted/30 text-xs font-medium text-foreground cursor-pointer hover:bg-muted/50 transition-colors"
          >
            <span class="flex items-center gap-1.5">
              <ShieldCheck class="h-3.5 w-3.5 text-primary" />
              <span>Tata Kelola CISO & Keamanan Lanjutan</span>
            </span>
            {#if showAdvancedSecurity}<ChevronUp class="h-3.5 w-3.5" />{:else}<ChevronDown class="h-3.5 w-3.5" />{/if}
          </button>

          {#if showAdvancedSecurity}
            <div class="p-3 mt-2 rounded-xl border border-border bg-card space-y-3">
              <div class="grid grid-cols-2 gap-2">
                <div>
                  <label for="mcp-ssrf-pol" class="block text-[10px] font-medium text-foreground mb-1">Kebijakan Anti-SSRF</label>
                  <select id="mcp-ssrf-pol" bind:value={formSsrfPolicy} class="w-full h-8 px-2 rounded-lg border border-border bg-background text-[11px]">
                    <option value="strict">Ketat (Blokir Private LAN & Cloud Metadata)</option>
                    <option value="custom_allowlist">Izinkan Domain Tertentu (Allowlist)</option>
                  </select>
                </div>
                <div>
                  <label for="mcp-sandbox" class="block text-[10px] font-medium text-foreground mb-1">Isolasi Sandboxing</label>
                  <select id="mcp-sandbox" bind:value={formSandboxType} class="w-full h-8 px-2 rounded-lg border border-border bg-background text-[11px]">
                    <option value="restricted_process">Restricted Process Jail</option>
                    <option value="docker_ephemeral">Docker Ephemeral Container</option>
                  </select>
                </div>
              </div>

              {#if formSsrfPolicy === "custom_allowlist"}
                <div>
                  <label for="mcp-egress-list" class="block text-[10px] font-medium text-foreground mb-1">Egress Allowlist (Koma dipisah)</label>
                  <input id="mcp-egress-list" type="text" bind:value={formEgressAllowlist} placeholder="api.github.com, internal.corp" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
                </div>
              {/if}

              <div class="grid grid-cols-2 gap-2">
                <div>
                  <label for="mcp-rpm-limit" class="block text-[10px] font-medium text-foreground mb-1">Rate Limit (RPM)</label>
                  <input id="mcp-rpm-limit" type="number" bind:value={formRateLimitRpm} class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
                </div>
                <div>
                  <label for="mcp-daily-limit" class="block text-[10px] font-medium text-foreground mb-1">Kuota Harian (Panggilan/Hari)</label>
                  <input id="mcp-daily-limit" type="number" bind:value={formDailyLimit} class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
                </div>
              </div>

              <div class="flex items-center justify-between p-2 rounded-lg bg-muted/40">
                <span class="text-[11px] font-medium text-foreground">Inline DLP Scrubbing (Mask NIK & Kartu Kredit)</span>
                <input type="checkbox" bind:checked={formDlpEnabled} class="h-4 w-4 rounded accent-primary cursor-pointer" />
              </div>
            </div>
          {/if}
        </div>

        {#if pingResult}
          <div class="p-2.5 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2">
            <CheckCircle class="h-4 w-4 shrink-0" />
            <span>Koneksi berhasil! Terdeteksi {pingResult.count} tool.</span>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between pt-2 border-t border-border">
        <Button variant="outline" size="sm" onclick={handleTestEndpoint} disabled={isTesting}>
          {#if isTesting}<Loader2 class="h-3.5 w-3.5 animate-spin mr-1.5" />{:else}<RefreshCw class="h-3.5 w-3.5 mr-1.5" />{/if}
          <span>Uji Endpoint</span>
        </Button>
        <div class="flex gap-2">
          <Button variant="ghost" size="sm" onclick={onClose}>Batal</Button>
          <Button size="sm" onclick={handleSaveServer} disabled={isSaving}>
            {#if isSaving}<Loader2 class="h-3.5 w-3.5 animate-spin mr-1.5" />{/if}
            <span>Simpan Server</span>
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}
