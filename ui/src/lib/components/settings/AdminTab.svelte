<script lang="ts">
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import Terminal from "lucide-svelte/icons/terminal";
  import Users from "lucide-svelte/icons/users";
  import FileText from "lucide-svelte/icons/file-text";
  import Activity from "lucide-svelte/icons/activity";
  import Server from "lucide-svelte/icons/server";
  import { Badge } from "$lib/components/ui/badge";
  import AdminKillswitchCard from "./admin/AdminKillswitchCard.svelte";
  import AdminApprovalsList from "./admin/AdminApprovalsList.svelte";
  import AdminMcpServersList from "./admin/AdminMcpServersList.svelte";
  import AdminUsersList from "./admin/AdminUsersList.svelte";
  import AdminAuditLogs from "./admin/AdminAuditLogs.svelte";
  import type { User } from "$lib/types";

  interface Props {
    user: User | null;
  }

  let { user }: Props = $props();

  let activeSubTab = $state<"soc" | "approvals" | "mcp-servers" | "users" | "audit">("soc");

  const subTabs = [
    { id: "soc", label: "Kontrol SOC & Engine", icon: ShieldAlert },
    { id: "approvals", label: "Persetujuan MCP", icon: Terminal },
    { id: "mcp-servers", label: "Server MCP", icon: Server },
    { id: "users", label: "Manajemen Pengguna", icon: Users },
    { id: "audit", label: "Audit SIEM", icon: FileText },
  ] as const;
</script>

<div class="space-y-5 font-sans text-xs">
  <!-- System Overview Banner -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between p-3.5 rounded-xl border border-border bg-card gap-2.5">
    <div class="flex items-center gap-2.5 min-w-0">
      <div class="p-2 rounded-lg bg-emerald-950/50 text-emerald-400 border border-emerald-800/40 shrink-0">
        <Activity class="h-4 w-4" />
      </div>
      <div class="min-w-0">
        <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
          Pusat Kendali Administrasi Enterprise
        </div>
        <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
          Akses terotorisasi untuk Administrator ({user?.role || "Admin"}). Terkoneksi ke SQLite/Postgres & SIEM.
        </div>
      </div>
    </div>
    <Badge
      variant="outline"
      class="text-emerald-400 border-emerald-800/40 bg-emerald-950/40 font-mono text-[10px] shrink-0 self-start sm:self-auto"
    >
      Engine Ready
    </Badge>
  </div>

  <!-- Sub Navigation Bar -->
  <div class="flex items-center gap-1.5 p-1 rounded-xl bg-muted/40 border border-border/80 overflow-x-auto no-scrollbar">
    {#each subTabs as tab}
      {@const Icon = tab.icon}
      {@const isActive = activeSubTab === tab.id}
      <button
        type="button"
        onclick={() => (activeSubTab = tab.id)}
        class="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium cursor-pointer transition-colors shrink-0 min-h-[44px] sm:min-h-8 touch-manipulation {isActive ? 'bg-card text-foreground font-semibold shadow-xs border border-border/80' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
      >
        <Icon class="h-3.5 w-3.5 shrink-0" />
        <span class="whitespace-nowrap [overflow-wrap:anywhere]">
          {tab.label}
        </span>
      </button>
    {/each}
  </div>

  <!-- Sub Tab Contents -->
  <div class="min-w-0">
    {#if activeSubTab === "soc"}
      <AdminKillswitchCard />
    {:else if activeSubTab === "approvals"}
      <AdminApprovalsList />
    {:else if activeSubTab === "mcp-servers"}
      <AdminMcpServersList />
    {:else if activeSubTab === "users"}
      <AdminUsersList currentUser={user} />
    {:else if activeSubTab === "audit"}
      <AdminAuditLogs />
    {/if}
  </div>
</div>
