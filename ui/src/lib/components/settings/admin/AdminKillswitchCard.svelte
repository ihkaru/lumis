<script lang="ts">
  import { onMount } from "svelte";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Power from "lucide-svelte/icons/power";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import { api } from "$lib/api";
  import type { KillswitchStatus } from "$lib/types";

  let status = $state<KillswitchStatus | null>(null);
  let isLoading = $state(true);
  let isSubmitting = $state(false);
  let reason = $state("");
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  async function loadStatus() {
    isLoading = true;
    errorMessage = null;
    try {
      status = await api.getKillswitch();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat status killswitch";
    } finally {
      isLoading = false;
    }
  }

  async function handleToggleKillswitch() {
    if (!status) return;
    const targetState = !status.active;
    const cleanReason = reason.trim() || (targetState ? "Aktivasi manual oleh Administrator" : "Pemulihan operasional oleh Administrator");

    isSubmitting = true;
    errorMessage = null;
    successMessage = null;
    try {
      status = await api.setKillswitch(targetState, cleanReason);
      successMessage = targetState
        ? "Emergency Killswitch berhasil diaktifkan. Akses non-admin telah dikunci."
        : "Emergency Killswitch dinonaktifkan. Operasional normal dipulihkan.";
      reason = "";
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memperbarui status killswitch";
    } finally {
      isSubmitting = false;
    }
  }

  onMount(() => {
    loadStatus();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  {#if errorMessage}
    <div class="p-3 rounded-xl border border-destructive/50 bg-destructive/10 text-destructive text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <AlertTriangle class="h-4 w-4 shrink-0" />
      <span>{errorMessage}</span>
    </div>
  {/if}

  {#if successMessage}
    <div class="p-3 rounded-xl border border-emerald-800/40 bg-emerald-950/40 text-emerald-400 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <ShieldCheck class="h-4 w-4 shrink-0" />
      <span>{successMessage}</span>
    </div>
  {/if}

  <!-- Killswitch Status Banner -->
  <div class="p-4 rounded-xl border {status?.active ? 'border-destructive/60 bg-destructive/10' : 'border-border bg-card'} space-y-3">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="flex items-center gap-3 min-w-0">
        <div class="p-2.5 rounded-lg {status?.active ? 'bg-destructive text-destructive-foreground' : 'bg-emerald-950/50 text-emerald-400 border border-emerald-800/40'} shrink-0">
          {#if isLoading}
            <Loader2 class="h-5 w-5 animate-spin" />
          {:else if status?.active}
            <ShieldAlert class="h-5 w-5" />
          {:else}
            <ShieldCheck class="h-5 w-5" />
          {/if}
        </div>
        <div class="min-w-0">
          <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
            SOC Emergency Killswitch
          </div>
          <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
            {#if status?.active}
              Karantina darurat aktif. Seluruh inferensi non-admin diblokir secara terpusat.
            {:else}
              Gateway beroperasi normal. Akses terbuka sesuai kebijakan clearance model.
            {/if}
          </div>
        </div>
      </div>

      <Badge
        variant={status?.active ? "destructive" : "outline"}
        class="shrink-0 self-start sm:self-auto font-mono text-[10px] {status?.active ? '' : 'text-emerald-400 border-emerald-800/40 bg-emerald-950/40'}"
      >
        {status?.active ? "LOCKDOWN AKTIF" : "OPERASIONAL NORMAL"}
      </Badge>
    </div>

    {#if status?.active && status.reason}
      <div class="p-2.5 rounded-lg bg-background/50 border border-border/60 text-[11px] space-y-1">
        <div class="text-muted-foreground font-medium">Alasan Karantina:</div>
        <div class="text-foreground font-mono [overflow-wrap:anywhere] break-words">{status.reason}</div>
        {#if status.activated_by}
          <div class="text-[10px] text-muted-foreground">Oleh: {status.activated_by}</div>
        {/if}
      </div>
    {/if}

    <!-- Killswitch Action Form -->
    <div class="pt-2 border-t border-border/60 flex flex-col sm:flex-row gap-2">
      <div class="flex-1 min-w-0">
        <Input
          type="text"
          placeholder={status?.active ? "Alasan pemulihan (opsional)..." : "Alasan penguncian darurat (wajib dicatat di SIEM)..."}
          bind:value={reason}
          disabled={isSubmitting || isLoading}
          class="h-9 text-xs"
        />
      </div>
      <Button
        type="button"
        variant={status?.active ? "outline" : "destructive"}
        size="sm"
        disabled={isSubmitting || isLoading}
        onclick={handleToggleKillswitch}
        class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words shrink-0"
      >
        {#if isSubmitting}
          <Loader2 class="h-3.5 w-3.5 animate-spin" />
          <span>Memproses...</span>
        {:else}
          <Power class="h-3.5 w-3.5" />
          <span>{status?.active ? "Pulihkan Akses Normal" : "Aktifkan Killswitch Darurat"}</span>
        {/if}
      </Button>
    </div>
  </div>
</div>
