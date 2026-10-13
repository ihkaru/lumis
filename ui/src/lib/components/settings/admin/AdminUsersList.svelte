<script lang="ts">
  import { onMount } from "svelte";
  import Users from "lucide-svelte/icons/users";
  import UserPlus from "lucide-svelte/icons/user-plus";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Check from "lucide-svelte/icons/check";
  import Shield from "lucide-svelte/icons/shield";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import { api } from "$lib/api";
  import type { User } from "$lib/types";

  interface Props {
    currentUser: User | null;
  }

  let { currentUser }: Props = $props();

  let users = $state<User[]>([]);
  let isLoading = $state(true);
  let isSubmitting = $state(false);
  let updatingUsername = $state<string | null>(null);

  let newUsername = $state("");
  let newRole = $state("member");
  let statusMessage = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);

  const isSuperAdmin = $derived(currentUser?.role === "super_admin");

  const availableRoles = [
    { value: "guest", label: "Guest (Clearance Basic)" },
    { value: "member", label: "Member (Clearance Standard)" },
    { value: "power_user", label: "Power User (Clearance Reasoning)" },
    { value: "tenant_admin", label: "Tenant Admin (Kelola Satker)" },
    { value: "super_admin", label: "Super Admin (Enterprise Root)" },
  ];

  async function loadUsers() {
    isLoading = true;
    errorMessage = null;
    try {
      users = await api.listAdminUsers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat daftar pengguna";
    } finally {
      isLoading = false;
    }
  }

  async function handleAssignRole(e: SubmitEvent) {
    e.preventDefault();
    const targetUser = newUsername.trim();
    if (!targetUser) return;

    if (newRole === "super_admin" && !isSuperAdmin) {
      errorMessage = "Hanya Super Admin yang berwenang memberikan hak akses Super Admin.";
      return;
    }

    isSubmitting = true;
    errorMessage = null;
    statusMessage = null;

    try {
      await api.updateUserRole(targetUser, newRole);
      statusMessage = `Peran untuk ${targetUser} berhasil diperbarui menjadi ${newRole}.`;
      newUsername = "";
      await loadUsers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal memperbarui peran pengguna";
    } finally {
      isSubmitting = false;
    }
  }

  async function handleRoleChange(username: string, nextRole: string) {
    if (nextRole === "super_admin" && !isSuperAdmin) {
      errorMessage = "Hanya Super Admin yang berwenang mempromosikan ke Super Admin.";
      return;
    }

    updatingUsername = username;
    errorMessage = null;
    statusMessage = null;

    try {
      await api.updateUserRole(username, nextRole);
      statusMessage = `Peran untuk ${username} berhasil diubah ke ${nextRole}.`;
      await loadUsers();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal mengubah peran pengguna";
    } finally {
      updatingUsername = null;
    }
  }

  onMount(() => {
    loadUsers();
  });
</script>

<div class="space-y-4 font-sans text-xs">
  <div class="flex items-center justify-between gap-2">
    <div>
      <div class="font-semibold text-xs text-foreground [overflow-wrap:anywhere] break-words">
        Manajemen Identitas & Hak Akses Pengguna
      </div>
      <div class="text-[11px] text-muted-foreground [overflow-wrap:anywhere] break-words">
        Role-based Access Control (RBAC) terhubung langsung ke basis data relasional.
      </div>
    </div>
    <Button
      type="button"
      variant="outline"
      size="sm"
      disabled={isLoading}
      onclick={loadUsers}
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

  {#if statusMessage}
    <div class="p-3 rounded-xl border border-emerald-800/40 bg-emerald-950/40 text-emerald-400 text-xs flex items-center gap-2 [overflow-wrap:anywhere] break-words">
      <Check class="h-4 w-4 shrink-0" />
      <span>{statusMessage}</span>
    </div>
  {/if}

  <!-- Add / Promote User Form -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-3">
    <div class="font-semibold text-xs text-foreground flex items-center gap-2">
      <UserPlus class="h-4 w-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Tetapkan / Perbarui Peran Akun</span>
    </div>

    <form onsubmit={handleAssignRole} class="flex flex-col sm:flex-row gap-2">
      <div class="flex-1 min-w-0">
        <Input
          type="text"
          placeholder="Username / NIP / SSO ID..."
          bind:value={newUsername}
          required
          disabled={isSubmitting}
          class="h-9 text-xs"
        />
      </div>
      <select
        bind:value={newRole}
        disabled={isSubmitting}
        class="h-9 px-2.5 rounded-md border border-input bg-background text-xs font-sans text-foreground focus:outline-hidden focus:ring-1 focus:ring-ring shrink-0"
      >
        {#each availableRoles as r}
          <option value={r.value} disabled={r.value === "super_admin" && !isSuperAdmin}>
            {r.label}
          </option>
        {/each}
      </select>
      <Button
        type="submit"
        variant="default"
        size="sm"
        disabled={isSubmitting || !newUsername.trim()}
        class="gap-1.5 text-xs min-h-[44px] sm:min-h-9 touch-manipulation whitespace-normal break-words shrink-0"
      >
        {#if isSubmitting}
          <Loader2 class="h-3.5 w-3.5 animate-spin" />
        {:else}
          <Shield class="h-3.5 w-3.5" />
        {/if}
        <span>Simpan Peran</span>
      </Button>
    </form>
  </div>

  <!-- Users Table -->
  <div class="p-4 rounded-xl border border-border bg-card space-y-3">
    <div class="font-semibold text-xs text-foreground flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Users class="h-4 w-4 text-primary shrink-0" />
        <span class="[overflow-wrap:anywhere] break-words">Pengguna Terdaftar ({users.length})</span>
      </div>
    </div>

    {#if isLoading}
      <div class="p-8 flex flex-col items-center justify-center gap-2 text-muted-foreground">
        <Loader2 class="h-6 w-6 animate-spin text-primary" />
        <span class="text-xs">Memuat data pengguna...</span>
      </div>
    {:else if users.length === 0}
      <div class="p-6 text-center text-muted-foreground">
        Belum ada data pengguna yang tersimpan di basis data.
      </div>
    {:else}
      <div class="overflow-x-auto rounded-lg border border-border/80">
        <table class="w-full text-left text-[11px]">
          <thead class="bg-muted/50 border-b border-border/80 font-semibold text-muted-foreground uppercase tracking-wider">
            <tr>
              <th class="p-2.5">Username</th>
              <th class="p-2.5">Tenant ID</th>
              <th class="p-2.5">Peran Saat Ini</th>
              <th class="p-2.5 text-right">Ubah Peran</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border/60">
            {#each users as u (u.id || u.username)}
              <tr class="hover:bg-muted/30 transition-colors">
                <td class="p-2.5 font-medium text-foreground [overflow-wrap:anywhere] break-words">
                  {u.username}
                </td>
                <td class="p-2.5 text-muted-foreground font-mono text-[10px]">
                  {u.tenant_id || "default"}
                </td>
                <td class="p-2.5">
                  <Badge
                    variant="outline"
                    class="font-mono text-[10px] capitalize {u.role === 'super_admin' ? 'border-primary/60 text-primary bg-primary/10' : u.role === 'tenant_admin' ? 'border-blue-500/60 text-blue-400 bg-blue-950/30' : ''}"
                  >
                    {u.role}
                  </Badge>
                </td>
                <td class="p-2.5 text-right">
                  <div class="inline-flex items-center gap-1.5 justify-end">
                    {#if updatingUsername === u.username}
                      <Loader2 class="h-3.5 w-3.5 animate-spin text-primary" />
                    {:else}
                      <select
                        value={u.role}
                        onchange={(e) => handleRoleChange(u.username, (e.target as HTMLSelectElement).value)}
                        class="h-8 px-2 rounded border border-input bg-background text-[11px] font-sans text-foreground focus:outline-hidden focus:ring-1 focus:ring-ring"
                      >
                        {#each availableRoles as r}
                          <option value={r.value} disabled={r.value === "super_admin" && !isSuperAdmin}>
                            {r.value}
                          </option>
                        {/each}
                      </select>
                    {/if}
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
