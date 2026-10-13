<script lang="ts">
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import ChevronUp from "lucide-svelte/icons/chevron-up";

  interface Props {
    ssrfPolicy: "strict" | "custom_allowlist";
    egressAllowlist: string;
    sandboxType: "restricted_process" | "docker_ephemeral";
    rateLimitRpm: number;
    dailyLimit: number;
    dlpEnabled: boolean;
  }

  let {
    ssrfPolicy = $bindable("strict"),
    egressAllowlist = $bindable(""),
    sandboxType = $bindable("restricted_process"),
    rateLimitRpm = $bindable(60),
    dailyLimit = $bindable(1000),
    dlpEnabled = $bindable(true),
  }: Props = $props();

  let showAdvanced = $state(false);
</script>

<div class="pt-1 font-sans text-xs">
  <button
    type="button"
    onclick={() => (showAdvanced = !showAdvanced)}
    class="flex items-center justify-between w-full p-2.5 rounded-xl border border-border bg-muted/30 text-xs font-medium text-foreground cursor-pointer hover:bg-muted/50 transition-colors"
  >
    <span class="flex items-center gap-1.5">
      <ShieldCheck class="h-3.5 w-3.5 text-primary" />
      <span>Tata Kelola CISO & Keamanan Lanjutan</span>
    </span>
    {#if showAdvanced}<ChevronUp class="h-3.5 w-3.5" />{:else}<ChevronDown class="h-3.5 w-3.5" />{/if}
  </button>

  {#if showAdvanced}
    <div class="p-3 mt-2 rounded-xl border border-border bg-card space-y-3">
      <div class="grid grid-cols-2 gap-2">
        <div>
          <label for="mcp-ssrf-pol" class="block text-[10px] font-medium text-foreground mb-1">Kebijakan Anti-SSRF</label>
          <select id="mcp-ssrf-pol" bind:value={ssrfPolicy} class="w-full h-8 px-2 rounded-lg border border-border bg-background text-[11px]">
            <option value="strict">Ketat (Blokir Private LAN & Cloud Metadata)</option>
            <option value="custom_allowlist">Izinkan Domain Tertentu (Allowlist)</option>
          </select>
        </div>
        <div>
          <label for="mcp-sandbox" class="block text-[10px] font-medium text-foreground mb-1">Isolasi Sandboxing</label>
          <select id="mcp-sandbox" bind:value={sandboxType} class="w-full h-8 px-2 rounded-lg border border-border bg-background text-[11px]">
            <option value="restricted_process">Restricted Process Jail</option>
            <option value="docker_ephemeral">Docker Ephemeral Container</option>
          </select>
        </div>
      </div>

      {#if ssrfPolicy === "custom_allowlist"}
        <div>
          <label for="mcp-egress-list" class="block text-[10px] font-medium text-foreground mb-1">Egress Allowlist (Koma dipisah)</label>
          <input id="mcp-egress-list" type="text" bind:value={egressAllowlist} placeholder="mcp.dvlpid.my.id, internal.corp" class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
        </div>
      {/if}

      <div class="grid grid-cols-2 gap-2">
        <div>
          <label for="mcp-rpm-limit" class="block text-[10px] font-medium text-foreground mb-1">Rate Limit (RPM)</label>
          <input id="mcp-rpm-limit" type="number" bind:value={rateLimitRpm} class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
        </div>
        <div>
          <label for="mcp-daily-limit" class="block text-[10px] font-medium text-foreground mb-1">Kuota Harian (Panggilan/Hari)</label>
          <input id="mcp-daily-limit" type="number" bind:value={dailyLimit} class="w-full h-8 px-2.5 rounded-lg border border-border bg-background text-[11px]" />
        </div>
      </div>

      <div class="flex items-center justify-between p-2 rounded-lg bg-muted/40">
        <span class="text-[11px] font-medium text-foreground">Inline DLP Scrubbing (Mask NIK & Kartu Kredit)</span>
        <input type="checkbox" bind:checked={dlpEnabled} class="h-4 w-4 rounded accent-primary cursor-pointer" />
      </div>
    </div>
  {/if}
</div>
