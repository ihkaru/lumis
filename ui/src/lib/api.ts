import type {
  ArtifactIpcResponse,
  ArtifactSpecification,
  BrandingConfig,
  CursorPage,
  DriverHealth,
  KillswitchStatus,
  McpServerItem,
  McpToolItem,
  Message,
  ModelDescriptor,
  PendingToolApproval,
  RoleHierarchyInfo,
  Session,
  SiemAuditEvent,
  User,
  UserMemory,
} from "./types";

export const api = {
  async getBranding(): Promise<BrandingConfig> {
    const res = await fetch("/api/config/branding");
    if (!res.ok) throw new Error("Failed to fetch branding configuration");
    return res.json();
  },

  async getModels(): Promise<ModelDescriptor[]> {
    const res = await fetch("/api/models");
    if (!res.ok) throw new Error("Failed to fetch models");
    return res.json();
  },

  async getMe(): Promise<{ authenticated: boolean; user?: User; role: string; role_level: number; max_clearance_tier: string }> {
    const res = await fetch("/api/auth/me");
    if (!res.ok) throw new Error("Authentication failed");
    return res.json();
  },

  async login(username: string, password?: string): Promise<{ authenticated: boolean; user?: User; role: string; role_level: number; max_clearance_tier: string }> {
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Autentikasi gagal" }));
      throw new Error(err.error || "Autentikasi gagal");
    }
    return res.json();
  },

  async logout(): Promise<void> {
    await fetch("/api/auth/logout", { method: "POST" });
  },

  async getDbHealth(): Promise<DriverHealth> {
    const res = await fetch("/health/db");
    if (!res.ok) throw new Error("Failed to fetch database health");
    return res.json();
  },

  async listRoles(): Promise<RoleHierarchyInfo[]> {
    const res = await fetch("/api/admin/roles");
    if (!res.ok) throw new Error("Failed to list role hierarchy");
    return res.json();
  },


  async listSessions(cursorTime?: string, cursorId?: string, limit = 20): Promise<CursorPage<Session>> {
    const params = new URLSearchParams();
    if (cursorTime) params.set("cursor_time", cursorTime);
    if (cursorId) params.set("cursor_id", cursorId);
    params.set("limit", String(limit));

    const res = await fetch(`/api/sessions?${params.toString()}`);
    if (!res.ok) throw new Error("Failed to list sessions");
    return res.json();
  },

  async createSession(title: string, modelId = "my9model-free"): Promise<Session> {
    const res = await fetch("/api/sessions", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title, model_id: modelId }),
    });
    if (!res.ok) throw new Error("Failed to create session");
    return res.json();
  },

  async listMessages(sessionId: string, cursorTime?: string, cursorId?: string, limit = 20): Promise<CursorPage<Message>> {
    const params = new URLSearchParams();
    if (cursorTime) params.set("cursor_time", cursorTime);
    if (cursorId) params.set("cursor_id", cursorId);
    params.set("limit", String(limit));

    const res = await fetch(`/api/sessions/${sessionId}/messages?${params.toString()}`);
    if (!res.ok) throw new Error("Failed to list messages");
    return res.json();
  },

  async sendMessage(sessionId: string, content: string, modelId?: string): Promise<{ status: string }> {
    const res = await fetch(`/api/sessions/${sessionId}/messages`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ content, model_id: modelId }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Unknown error" }));
      throw new Error(err.error || "Failed to send message");
    }
    return res.json();
  },

  connectStream(onToken: (data: string) => void): EventSource {
    const es = new EventSource("/api/stream");
    es.addEventListener("token", (e) => onToken(e.data));
    return es;
  },

  async parseArtifacts(text: string): Promise<ArtifactSpecification[]> {
    const res = await fetch("/api/artifacts/parse", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ text }),
    });
    if (!res.ok) throw new Error("Failed to parse artifacts");
    const data = await res.json();
    return data.artifacts || [];
  },

  async sendArtifactIpc(
    artifactId: string,
    request: { request_id: string; action: string | Record<string, unknown>; payload?: unknown },
    sessionId?: string
  ): Promise<ArtifactIpcResponse> {
    const actionPayload =
      typeof request.action === "string"
        ? { action: request.action, ...(request.payload ? { payload: request.payload } : {}) }
        : request.action;

    const res = await fetch("/api/artifacts/ipc", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        artifact_id: artifactId,
        session_id: sessionId,
        request: {
          request_id: request.request_id,
          action: actionPayload,
        },
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Artifact IPC failed" }));
      throw new Error(err.error || "Artifact IPC failed");
    }
    return res.json();
  },

  async listUserMemories(): Promise<UserMemory[]> {
    const res = await fetch("/api/user/memories");
    if (!res.ok) throw new Error("Failed to list user memories");
    const data = await res.json();
    return data.memories || [];
  },

  async createUserMemory(content: string, category = "preference"): Promise<UserMemory> {
    const res = await fetch("/api/user/memories", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ content, category }),
    });
    if (!res.ok) throw new Error("Failed to create memory");
    return res.json();
  },

  async deleteUserMemory(id: string): Promise<void> {
    const res = await fetch(`/api/user/memories/${id}`, {
      method: "DELETE",
    });
    if (!res.ok) throw new Error("Failed to delete memory");
  },

  async getKillswitch(): Promise<KillswitchStatus> {
    const res = await fetch("/api/admin/emergency/killswitch");
    if (!res.ok) throw new Error("Failed to fetch killswitch status");
    return res.json();
  },

  async setKillswitch(active: boolean, reason: string): Promise<KillswitchStatus> {
    const res = await fetch("/api/admin/emergency/killswitch", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ active, reason }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Failed to update killswitch" }));
      throw new Error(err.error || "Failed to update killswitch");
    }
    return res.json();
  },

  async getAuditEvents(limit = 50): Promise<SiemAuditEvent[]> {
    const res = await fetch(`/api/admin/audit/events?limit=${limit}`);
    if (!res.ok) throw new Error("Failed to fetch audit events");
    return res.json();
  },

  async getPendingApprovals(): Promise<PendingToolApproval[]> {
    const res = await fetch("/api/mcp/approvals/pending");
    if (!res.ok) throw new Error("Failed to fetch pending approvals");
    return res.json();
  },

  async resolveApproval(token: string, approved: boolean, reason?: string): Promise<{ status: string }> {
    const endpoint = approved
      ? `/api/mcp/approvals/${token}/approve`
      : `/api/mcp/approvals/${token}/deny`;
    const res = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ reason: reason || (approved ? "Approved by admin" : "Denied by admin") }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Failed to resolve approval" }));
      throw new Error(err.error || "Failed to resolve approval");
    }
    return res.json();
  },

  async listAdminUsers(): Promise<User[]> {
    const res = await fetch("/api/admin/users");
    if (!res.ok) throw new Error("Failed to list users");
    return res.json();
  },

  async updateUserRole(username: string, role: string): Promise<User> {
    const res = await fetch("/api/admin/users/role", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, role }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Failed to update user role" }));
      throw new Error(err.error || "Failed to update user role");
    }
    return res.json();
  },

  async changePassword(oldPassword: string, newPassword: string): Promise<{ status: string }> {
    const res = await fetch("/api/auth/password", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ old_password: oldPassword || undefined, new_password: newPassword }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal memperbarui kata sandi" }));
      throw new Error(err.error || "Gagal memperbarui kata sandi");
    }
    return res.json();
  },

  async listMcpServers(): Promise<{ servers: McpServerItem[] }> {
    const res = await fetch("/api/mcp/servers");
    if (!res.ok) throw new Error("Gagal mengambil daftar server MCP");
    return res.json();
  },

  async createMcpServer(payload: any): Promise<{ server: McpServerItem; tools_count: number }> {
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

  async getMcpServer(id: string): Promise<{ server: McpServerItem; tools: McpToolItem[] }> {
    const res = await fetch(`/api/mcp/servers/${id}`);
    if (!res.ok) throw new Error("Gagal mengambil detail server MCP");
    return res.json();
  },

  async updateMcpServer(id: string, payload: any): Promise<{ server: McpServerItem }> {
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

  async deleteMcpServer(id: string): Promise<boolean> {
    const res = await fetch(`/api/mcp/servers/${id}`, { method: "DELETE" });
    if (!res.ok) throw new Error("Gagal menghapus server MCP");
    return true;
  },

  async testMcpEndpoint(payload: { endpoint_url: string; auth_type?: string; raw_credentials?: string }): Promise<{ success: boolean; count: number }> {
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
};
