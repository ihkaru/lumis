import type { CursorPage, DriverHealth, Message, RoleHierarchyInfo, Session, User } from "./types";

export const api = {
  async getMe(): Promise<{ authenticated: boolean; user?: User; role: string; role_level: number; max_clearance_tier: string }> {
    const res = await fetch("/api/auth/me");
    if (!res.ok) throw new Error("Authentication failed");
    return res.json();
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

  async createSession(title: string, modelId = "default"): Promise<Session> {
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
};
