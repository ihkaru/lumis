<script lang="ts">
  import Server from "lucide-svelte/icons/server";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import LayoutList from "lucide-svelte/icons/layout-list";
  import Code2 from "lucide-svelte/icons/code-2";
  import Sparkles from "lucide-svelte/icons/sparkles";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { mcpApi } from "$lib/mcpApi";
  import { parseMcpJson, getSampleMcpJson } from "$lib/mcpConfigParser";
  import AdminMcpSecurityFields from "./AdminMcpSecurityFields.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
    onCreated: () => void;
  }

  let { open, onClose, onCreated }: Props = $props();

  let activeTab = $state<"form" | "json">("form");
  let jsonContent = $state(getSampleMcpJson());

  let formName = $state(""), formTitle = $state(""), formDesc = $state(""), formUrl = $state("");
  let formTransport = $state<"sse" | "streamable_http">("sse");
  let formAuth = $state<"none" | "bearer" | "basic" | "oauth2">("none");
  let formCred = $state(""), formIsMandatory = $state(false);

  // OAuth 2.1 fields
  let formOAuthClientId = $state(""), formOAuthAuthEndpoint = $state("");
  let formOAuthTokenEndpoint = $state(""), formOAuthScopes = $state(""), formOAuthResource = $state("");

  // CISO Security Governance fields
  let formSsrfPolicy = $state<"strict" | "custom_allowlist">("strict");
  let formEgressAllowlist = $state(""), formSandboxType = $state<"restricted_process" | "docker_ephemeral">("restricted_process");
  let formMaxMemory = $state(256), formTimeoutSecs = $state(30), formDlpEnabled = $state(true);
  let formRateLimitRpm = $state(60), formDailyLimit = $state(1000), formKmsProvider = $state<"local" | "vault" | "aws_kms">("local");

  let isTesting = $state(false), isSaving = $state(false);
  let errorMessage = $state<string | null>(null);
  let pingResult = $state<{ success: boolean; count: number } | null>(null);

  function applyJsonToForm() {
    try {
      const parsed = parseMcpJson(jsonContent);
      formName = parsed.name;
      formTitle = parsed.displayTitle;
      formDesc = parsed.description || "";
      formUrl = parsed.endpointUrl || "";
      formTransport = parsed.transportType;
      formAuth = parsed.authType;
      formCred = parsed.rawCredentials || "";
      formIsMandatory = parsed.isMandatory || false;
      if (parsed.rateLimitRpm) formRateLimitRpm = parsed.rateLimitRpm;
      if (parsed.dailyLimit) formDailyLimit = parsed.dailyLimit;
      errorMessage = null;
      activeTab = "form";
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "JSON tidak valid";
    }
  }

  function loadSamplePropylon() {
    jsonContent = getSampleMcpJson();
    applyJsonToForm();
  }

  async function handleTestEndpoint() {
    if (activeTab === "json") {
      try {
        const parsed = parseMcpJson(jsonContent);
        formUrl = parsed.endpointUrl || "";
        formAuth = parsed.authType;
        formCred = parsed.rawCredentials || "";
      } catch (e) {
        errorMessage = e instanceof Error ? e.message : "JSON tidak valid";
        return;
      }
    }

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
    if (activeTab === "json") {
      try {
        const parsed = parseMcpJson(jsonContent);
        formName = parsed.name;
        formTitle = parsed.displayTitle;
        formDesc = parsed.description || "";
        formUrl = parsed.endpointUrl || "";
        formTransport = parsed.transportType;
        formAuth = parsed.authType;
        formCred = parsed.rawCredentials || "";
        formIsMandatory = parsed.isMandatory || false;
      } catch (e) {
        errorMessage = e instanceof Error ? e.message : "JSON tidak valid";
        return;
      }
    }

    if (!formName.trim() || !formTitle.trim()) {
      errorMessage = "Nama identifier dan judul wajib diisi";
      return;
    }
    isSaving = true;
    errorMessage = null;
    try {
      const oauthConfig = formAuth === "oauth2" ? {
        client_id: formOAuthClientId, authorization_endpoint: formOAuthAuthEndpoint,
        token_endpoint: formOAuthTokenEndpoint,
        scopes: formOAuthScopes.split(",").map(s => s.trim()).filter(Boolean),
        resource_indicator: formOAuthResource || undefined, registration_mode: "pre_registered",
      } : undefined;

      const egressAllowlist = formEgressAllowlist ? formEgressAllowlist.split(",").map(s => s.trim()).filter(Boolean) : undefined;

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

      <!-- Mode Switcher Tabs: Visual Form vs Paste JSON -->
      <div class="flex items-center justify-between p-1 rounded-xl bg-muted/40 border border-border/60">
        <div class="flex gap-1">
          <button
            type="button"
            onclick={() => (activeTab = "form")}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer {activeTab === 'form' ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'}"
          >
            <LayoutList class="h-3.5 w-3.5" />
            <span>Form Visual</span>
          </button>
          <button
            type="button"
            onclick={() => (activeTab = "json")}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer {activeTab === 'json' ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'}"
          >
            <Code2 class="h-3.5 w-3.5" />
            <span>Paste JSON (Claude / Cursor)</span>
          </button>
        </div>

        {#if activeTab === "json"}
          <button
            type="button"
            onclick={loadSamplePropylon}
            class="flex items-center gap-1 text-[11px] text-cyan-400 hover:text-cyan-300 px-2 py-1 rounded transition-colors cursor-pointer font-medium"
            title="Muat konfigurasi sampel Propylon Gateway"
          >
            <Sparkles class="h-3 w-3" />
            <span>Contoh mcp.dvlpid.my.id</span>
          </button>
        {/if}
      </div>

      {#if errorMessage}
        <div class="p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-[11px] flex items-center gap-2">
          <AlertTriangle class="h-4 w-4 shrink-0" />
          <span class="[overflow-wrap:anywhere] break-words">{errorMessage}</span>
        </div>
      {/if}

      {#if activeTab === "json"}
        <!-- JSON Paste Mode -->
        <div class="space-y-3">
          <div class="flex items-center justify-between text-muted-foreground text-[11px]">
            <span>Paste konfigurasi standard <code>mcpServers</code> atau objek server tunggal:</span>
            <button
              type="button"
              onclick={applyJsonToForm}
              class="text-primary hover:underline font-medium cursor-pointer"
            >
              Parse ke Form Visual &rarr;
            </button>
          </div>
          <textarea
            bind:value={jsonContent}
            rows={12}
            placeholder={`{\n  "mcpServers": {\n    "propylon": {\n      "url": "https://mcp.dvlpid.my.id/sse",\n      "transport": "sse"\n    }\n  }\n}`}
            class="w-full p-3 rounded-xl border border-border bg-slate-950 text-slate-100 font-mono text-[11px] leading-relaxed resize-y focus:outline-none focus:ring-1 focus:ring-primary"
          ></textarea>
        </div>
      {:else}
        <!-- Visual Form Mode -->
        <div class="space-y-3">
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label for="mcp-create-name" class="block text-[11px] font-medium text-foreground mb-1">Slug Identifier</label>
              <input id="mcp-create-name" type="text" bind:value={formName} placeholder="misal: propylon" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
            </div>
            <div>
              <label for="mcp-create-title" class="block text-[11px] font-medium text-foreground mb-1">Judul Tampilan</label>
              <input id="mcp-create-title" type="text" bind:value={formTitle} placeholder="misal: Propylon Gateway" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
            </div>
          </div>

          <div>
            <label for="mcp-create-url" class="block text-[11px] font-medium text-foreground mb-1">Endpoint URL (Anti-SSRF Validated)</label>
            <input id="mcp-create-url" type="text" bind:value={formUrl} placeholder="https://mcp.dvlpid.my.id/sse" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
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
                <option value="none">Tanpa Autentikasi</option>
                <option value="bearer">Bearer Token (AES-256-GCM)</option>
                <option value="oauth2">OAuth 2.1 (RFC 6749bis PKCE)</option>
                <option value="basic">HTTP Basic Auth</option>
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

          <!-- Collapsible CISO Governance Fields Component -->
          <AdminMcpSecurityFields
            bind:ssrfPolicy={formSsrfPolicy}
            bind:egressAllowlist={formEgressAllowlist}
            bind:sandboxType={formSandboxType}
            bind:rateLimitRpm={formRateLimitRpm}
            bind:dailyLimit={formDailyLimit}
            bind:dlpEnabled={formDlpEnabled}
          />
        </div>
      {/if}

      {#if pingResult}
        <div class="p-2.5 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2">
          <CheckCircle class="h-4 w-4 shrink-0" />
          <span>Koneksi berhasil! Terdeteksi {pingResult.count} tool aktif dari server MCP.</span>
        </div>
      {/if}

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
