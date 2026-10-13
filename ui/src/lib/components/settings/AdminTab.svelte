<script lang="ts">
  import Users from "lucide-svelte/icons/users";
  import UserPlus from "lucide-svelte/icons/user-plus";
  import Activity from "lucide-svelte/icons/activity";
  import Check from "lucide-svelte/icons/check";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import UserGrid from "./UserGrid.svelte";

  interface ManagedUser {
    id: string;
    username: string;
    role: string;
    isActive: boolean;
  }

  let users = $state<ManagedUser[]>([
    { id: "1", username: "admin.bps", role: "super_admin", isActive: true },
    { id: "2", username: "power.user", role: "power_user", isActive: true },
    { id: "3", username: "analyst.satker", role: "member", isActive: true },
  ]);

  let newUsername = $state("");
  let newRole = $state("member");
  let statusMessage = $state<string | null>(null);

  function handleCreateUser(e: SubmitEvent) {
    e.preventDefault();
    if (!newUsername.trim()) return;

    users = [
      ...users,
      {
        id: String(Date.now()),
        username: newUsername.trim(),
        role: newRole,
        isActive: true,
      },
    ];

    statusMessage = `Pengguna ${newUsername} berhasil ditambahkan.`;
    newUsername = "";
    setTimeout(() => {
      statusMessage = null;
    }, 3000);
  }
</script>

<div class="space-y-6 font-sans text-xs">
  <!-- System Health Card -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between p-3.5 rounded-xl border border-border bg-card gap-2.5">
    <div class="flex items-center gap-2.5 min-w-0">
      <div class="p-2 rounded-lg bg-emerald-950/50 text-emerald-400 border border-emerald-800/40 shrink-0">
        <Activity class="h-4 w-4" />
      </div>
      <div class="min-w-0">
        <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">Status Engine Lumis</div>
        <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">Koneksi Database WAL & Model Gateway 100% Aktif</div>
      </div>
    </div>
    <Badge variant="outline" class="text-emerald-400 border-emerald-800/40 bg-emerald-950/40 font-mono text-[10px] shrink-0 self-start sm:self-auto">
      Ready
    </Badge>
  </div>

  <!-- Create User Section -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-3">
    <div class="font-semibold text-xs text-foreground flex items-center gap-2">
      <UserPlus class="h-4 w-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Tambah Pengguna Baru</span>
    </div>

    {#if statusMessage}
      <div class="p-2.5 rounded-lg bg-emerald-950/40 border border-emerald-800/40 text-emerald-400 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
        <Check class="h-3.5 w-3.5 shrink-0" />
        <span>{statusMessage}</span>
      </div>
    {/if}

    <form onsubmit={handleCreateUser} class="flex flex-col sm:flex-row gap-2">
      <div class="flex-1 min-w-0">
        <Input
          type="text"
          placeholder="Username / NIP..."
          bind:value={newUsername}
          required
          class="h-9 text-xs"
        />
      </div>
      <select
        bind:value={newRole}
        class="h-9 px-2.5 rounded-md border border-input bg-background text-xs font-sans text-foreground focus:outline-hidden focus:ring-1 focus:ring-ring shrink-0"
      >
        <option value="member">Member</option>
        <option value="power_user">Power User</option>
        <option value="tenant_admin">Tenant Admin</option>
        <option value="super_admin">Super Admin</option>
      </select>
      <Button type="submit" variant="default" size="sm" class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words shrink-0">
        <UserPlus class="h-3.5 w-3.5 shrink-0" />
        <span>Tambah</span>
      </Button>
    </form>
  </div>

  <!-- User Grid Table -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-3">
    <div class="font-semibold text-xs text-foreground flex items-center gap-2">
      <Users class="h-4 w-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Daftar Pengguna Aktif</span>
    </div>

    <UserGrid {users} />
  </div>
</div>
