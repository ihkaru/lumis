<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";

  export interface GridUser {
    id: string;
    username: string;
    role: string;
    isActive: boolean;
  }

  interface Props {
    users: GridUser[];
  }

  let { users = [] }: Props = $props();
</script>

<div class="overflow-x-auto rounded-lg border border-border/80 font-sans">
  <table class="w-full text-left text-[11px]">
    <thead class="bg-muted/50 border-b border-border/80 font-semibold text-muted-foreground uppercase tracking-wider">
      <tr>
        <th class="p-2.5">Username / NIP</th>
        <th class="p-2.5">Tingkat Peran</th>
        <th class="p-2.5 text-right">Status Akun</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-border/60">
      {#each users as u}
        <tr class="hover:bg-muted/30 transition-colors">
          <td class="p-2.5 font-medium text-foreground">{u.username}</td>
          <td class="p-2.5">
            <Badge variant="outline" class="font-mono text-[10px] capitalize">
              {u.role}
            </Badge>
          </td>
          <td class="p-2.5 text-right text-emerald-400 font-medium">Aktif</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
