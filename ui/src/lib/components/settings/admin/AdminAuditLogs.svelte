<script lang="ts">
  import { onMount } from "svelte";
  import FileText from "lucide-svelte/icons/file-text";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { api } from "$lib/api";
  import type { SiemAuditEvent } from "$lib/types";

  let events = $state<SiemAuditEvent[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state<string | null>(null);

  async function loadAuditLogs() {
    isLoading = true;
    errorMessage = null;
    try {
      events = await api.getAuditEvents(30);
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat log audit SIEM";
    } finally {
      isLoading = false;
    }
  }

  function formatTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    } catch {
      return iso;
    }
  }

  onMount(() => {
    loadAuditLogs();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  <div class="flex items-center justify-between gap-2">
    <div>
      <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
        Log Keamanan & Audit SIEM Terpusat
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Audit trail immutable format Elastic Common Schema (ECS) & CEF untuk compliance SOC.
      </div>
    </div>
    <Button
      type="button"
      variant="outline"
      size="sm"
      disabled={isLoading}
      onclick={loadAuditLogs}
      class="gap-1.5 h-8 text-xs shrink-0 touch-manipulation"
    >
      <RefreshCw class="h-3.5 w-3.5 {isLoading ? 'animate-spin' : ''}" />
      <span class="hidden sm:inline">Segarkan</span>
    </Button>
  </div>

  {#if errorMessage}
    <div class="p-3 rounded-xl border border-destructive/50 bg-destructive/10 text-destructive text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <AlertTriangle class="h-4 w-4 shrink-0" />
      <span>{errorMessage}</span>
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 rounded-xl border border-border bg-card flex flex-col items-center justify-center gap-2 text-muted-foreground">
      <Loader2 class="h-6 w-6 animate-spin text-primary" />
      <span class="text-xs">Mengambil jejak audit SIEM...</span>
    </div>
  {:else if events.length === 0}
    <div class="p-6 rounded-xl border border-border bg-card text-center space-y-1">
      <div class="text-xs font-medium text-foreground">Belum ada catatan log audit</div>
      <div class="text-[11px] text-muted-foreground">
        Aktivitas administratif dan intervensi keamanan akan tercatat secara otomatis di sini.
      </div>
    </div>
  {:else}
    <div class="overflow-x-auto rounded-lg border border-border/80 bg-card">
      <table class="w-full text-left text-[11px]">
        <thead class="bg-muted/50 border-b border-border/80 font-semibold text-muted-foreground uppercase tracking-wider">
          <tr>
            <th class="p-2.5">Waktu</th>
            <th class="p-2.5">Aktor</th>
            <th class="p-2.5">Aksi / Event</th>
            <th class="p-2.5">Status</th>
            <th class="p-2.5">Rincian</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-border/60">
          {#each events as ev (ev.event_id || ev.id || Math.random())}
            {@const actionName = ev.action || ev.event_type || "security_event"}
            {@const statusStr = String(ev.status || "").toLowerCase()}
            <tr class="hover:bg-muted/30 transition-colors">
              <td class="p-2.5 font-mono text-[10px] text-muted-foreground whitespace-nowrap">
                {formatTime(ev.timestamp)}
              </td>
              <td class="p-2.5 font-medium text-foreground [overflow-wrap:anywhere] break-words">
                {ev.actor_id}
                <span class="text-[10px] text-muted-foreground block font-normal">({ev.actor_role})</span>
              </td>
              <td class="p-2.5 font-mono text-[10px] text-foreground [overflow-wrap:anywhere] break-words">
                {actionName}
              </td>
              <td class="p-2.5">
                <Badge
                  variant="outline"
                  class="font-mono text-[10px] capitalize {statusStr === 'success' ? 'border-emerald-800/40 text-emerald-400 bg-emerald-950/40' : statusStr === 'denied' ? 'border-amber-800/40 text-amber-400 bg-amber-950/40' : 'border-destructive/40 text-destructive bg-destructive/10'}"
                >
                  {statusStr || "info"}
                </Badge>
              </td>
              <td class="p-2.5 text-muted-foreground font-mono text-[10px] max-w-[200px] truncate [overflow-wrap:anywhere]">
                {JSON.stringify(ev.details || {})}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
