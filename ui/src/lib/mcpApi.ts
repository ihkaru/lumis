import type { McpServerItem, McpToolItem, McpUserPreferenceItem } from "./types";

export const mcpApi = {
  async listServers(): Promise<{ servers: McpServerItem[] }> {
    const res = await fetch("/api/mcp/servers");
    if (!res.ok) throw new Error("Gagal memuat server MCP");
    return res.json();
  },

  async createServer(payload: unknown): Promise<{ server: McpServerItem; tools_count: number }> {
    const res = await fetch("/api/mcp/servers", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal membuat server MCP" }));
      throw new Error(err.error || "Gagal membuat server MCP");
    }
    return res.json();
  },

  async getServer(id: string): Promise<{ server: McpServerItem; tools: McpToolItem[] }> {
    const res = await fetch(`/api/mcp/servers/${id}`);
    if (!res.ok) throw new Error("Gagal mengambil detail server MCP");
    return res.json();
  },

  async updateServer(id: string, payload: unknown): Promise<{ server: McpServerItem }> {
    const res = await fetch(`/api/mcp/servers/${id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal memperbarui server MCP" }));
      throw new Error(err.error || "Gagal memperbarui server MCP");
    }
    return res.json();
  },

  async deleteServer(id: string): Promise<boolean> {
    const res = await fetch(`/api/mcp/servers/${id}`, { method: "DELETE" });
    if (!res.ok) throw new Error("Gagal menghapus server MCP");
    return true;
  },

  async testEndpoint(payload: { endpoint_url: string; auth_type?: string; raw_credentials?: string }): Promise<{ success: boolean; count: number }> {
    const res = await fetch("/api/mcp/servers/test-endpoint", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Uji koneksi gagal" }));
      throw new Error(err.error || "Uji koneksi gagal");
    }
    return res.json();
  },

  async syncServer(id: string): Promise<{ success: boolean; tools_count: number }> {
    const res = await fetch(`/api/mcp/servers/${id}/sync`, { method: "POST" });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal menyinkronkan tools server" }));
      throw new Error(err.error || "Gagal menyinkronkan tools server");
    }
    return res.json();
  },

  // User MCP Preferences & OAuth 2.1
  async listUserPreferences(): Promise<{ servers: McpUserPreferenceItem[] }> {
    const res = await fetch("/api/mcp/preferences");
    if (!res.ok) throw new Error("Gagal memuat preferensi MCP pengguna");
    return res.json();
  },

  async togglePreference(serverId: string, isEnabled: boolean): Promise<{ success: boolean; server_id: string; is_enabled: boolean }> {
    const res = await fetch(`/api/mcp/preferences/${serverId}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ is_enabled: isEnabled }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal memperbarui status server" }));
      throw new Error(err.error || "Gagal memperbarui status server");
    }
    return res.json();
  },

  async authorizeOAuth(serverId: string, redirectUri: string): Promise<{ authorization_url: string; state: string; code_verifier: string }> {
    const res = await fetch(`/api/mcp/servers/${serverId}/oauth/authorize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ redirect_uri: redirectUri }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal memulai otorisasi OAuth" }));
      throw new Error(err.error || "Gagal memulai otorisasi OAuth");
    }
    return res.json();
  },

  async callbackOAuth(serverId: string, payload: { access_token: string; refresh_token?: string; expires_in?: number }): Promise<{ success: boolean }> {
    const res = await fetch(`/api/mcp/servers/${serverId}/oauth/callback`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal menyimpan token OAuth" }));
      throw new Error(err.error || "Gagal menyimpan token OAuth");
    }
    return res.json();
  },

  async revokeOAuthToken(serverId: string): Promise<{ success: boolean }> {
    const res = await fetch(`/api/mcp/servers/${serverId}/oauth/token`, { method: "DELETE" });
    if (!res.ok) throw new Error("Gagal memutuskan koneksi OAuth");
    return res.json();
  },

  async createUserServer(payload: unknown): Promise<{ server: McpServerItem }> {
    const res = await fetch("/api/mcp/servers/user", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal membuat server pribadi" }));
      throw new Error(err.error || "Gagal membuat server pribadi");
    }
    return res.json();
  },

  async deleteUserServer(id: string): Promise<boolean> {
    const res = await fetch(`/api/mcp/servers/user/${id}`, { method: "DELETE" });
    if (!res.ok) throw new Error("Gagal menghapus server pribadi");
    return true;
  },
};
