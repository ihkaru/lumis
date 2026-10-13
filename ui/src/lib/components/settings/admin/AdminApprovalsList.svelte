<script lang="ts">
  import { onMount } from "svelte";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import Terminal from "lucide-svelte/icons/terminal";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { api } from "$lib/api";
  import type { PendingToolApproval } from "$lib/types";

  let approvals = $state<PendingToolApproval[]>([]);
  let isLoading = $state(true);
  let processingToken = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  async function loadApprovals() {
    isLoading = true;
    errorMessage = null;
    try {
      approvals = await api.getPendingApprovals();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat persetujuan tool";
    } finally {
      isLoading = false;
    }
  }

  async function handleResolve(token: string, approved: boolean) {
    processingToken = token;
    errorMessage = null;
    successMessage = null;
    try {
      await api.resolveApproval(token, approved);
      successMessage = approved
        ? `Eksekusi tool [Token: ${token.slice(0, 8)}...] berhasil disetujui.`
        : `Eksekusi tool [Token: ${token.slice(0, 8)}...] telah ditolak.`;
      await loadApprovals();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memproses persetujuan";
    } finally {
      processingToken = null;
    }
  }

  onMount(() => {
    loadApprovals();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  <div class="flex items-center justify-between gap-2">
    <div>
      <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
        Persetujuan Tool Execution (Human-in-the-Loop)
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Daftar panggilan MCP tool berisiko tinggi (CISO clearance) yang menunggu otorisasi admin.
      </div>
    </div>
    <Button
      type="button"
      variant="outline"
      size="sm"
      disabled={isLoading}
      onclick={loadApprovals}
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

  {#if successMessage}
    <div class="p-3 rounded-xl border border-emerald-800/40 bg-emerald-950/40 text-emerald-400 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <CheckCircle class="h-4 w-4 shrink-0" />
      <span>{successMessage}</span>
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 rounded-xl border border-border bg-card flex flex-col items-center justify-center gap-2 text-muted-foreground">
      <Loader2 class="h-6 w-6 animate-spin text-primary" />
      <span class="text-xs">Memeriksa antrean persetujuan...</span>
    </div>
  {:else if approvals.length === 0}
    <div class="p-6 rounded-xl border border-border bg-card text-center space-y-1">
      <div class="text-xs font-medium text-foreground">Tidak ada permohonan tertunda</div>
      <div class="text-[11px] text-muted-foreground">
        Semua pemanggilan tool berbahaya berada dalam batas aman atau telah diselesaikan.
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      {#each approvals as appr (appr.token)}
        <div class="p-3.5 rounded-xl border border-border bg-card space-y-2.5">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <div class="p-1.5 rounded-lg bg-amber-950/50 text-amber-400 border border-amber-800/40 shrink-0">
                <Terminal class="h-4 w-4" />
              </div>
              <div class="min-w-0">
                <div class="font-mono font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
                  {appr.tool_name}
                </div>
                <div class="text-[10px] text-muted-foreground [overflow-wrap:anywhere] break-words">
                  Pemohon: <span class="text-foreground">{appr.caller_id}</span> ({appr.caller_role})
                </div>
              </div>
            </div>

            <Badge variant="outline" class="font-mono text-[10px] uppercase text-amber-400 border-amber-800/40 bg-amber-950/40 shrink-0 self-start sm:self-auto">
              Risk: {appr.risk_level}
            </Badge>
          </div>

          <!-- Arguments Code Box -->
          {#if Object.keys(appr.args || {}).length > 0}
            <div class="p-2 rounded-lg bg-muted/40 border border-border/60 overflow-x-auto text-[10px] font-mono text-muted-foreground [overflow-wrap:anywhere] break-words">
              {JSON.stringify(appr.args)}
            </div>
          {/if}

          <!-- Action Buttons -->
          <div class="flex items-center justify-end gap-2 pt-1 border-t border-border/40">
            <Button
              type="button"
              variant="destructive"
              size="sm"
              disabled={processingToken === appr.token}
              onclick={() => handleResolve(appr.token, false)}
              class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words shrink-0"
            >
              <XCircle class="h-3.5 w-3.5" />
              <span>Tolak</span>
            </Button>
            <Button
              type="button"
              variant="default"
              size="sm"
              disabled={processingToken === appr.token}
              onclick={() => handleResolve(appr.token, true)}
              class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words shrink-0 bg-emerald-600 hover:bg-emerald-500 text-white"
            >
              {#if processingToken === appr.token}
                <Loader2 class="h-3.5 w-3.5 animate-spin" />
              {:else}
                <CheckCircle class="h-3.5 w-3.5" />
              {/if}
              <span>Setujui</span>
            </Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
