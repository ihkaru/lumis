import type { Skill, SkillSummary } from "./types";

export interface SkillListParams {
  search?: string;
  category?: string;
  limit?: number;
  offset?: number;
}

export const skillsApi = {
  async listSkills(params?: SkillListParams): Promise<{ skills: SkillSummary[] }> {
    const sp = new URLSearchParams();
    if (params?.search) sp.set("search", params.search);
    if (params?.category) sp.set("category", params.category);
    if (params?.limit) sp.set("limit", params.limit.toString());
    if (params?.offset) sp.set("offset", params.offset.toString());

    const url = `/api/skills${sp.toString() ? `?${sp.toString()}` : ""}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error("Gagal mengambil daftar skill");
    return res.json();
  },

  async searchSkills(q: string, limit = 10): Promise<{ skills: SkillSummary[] }> {
    const sp = new URLSearchParams({ q, limit: limit.toString() });
    const res = await fetch(`/api/skills/search?${sp.toString()}`);
    if (!res.ok) throw new Error("Gagal mencari skill");
    return res.json();
  },

  async getSkill(id: string): Promise<{ skill: Skill }> {
    const res = await fetch(`/api/skills/${id}`);
    if (!res.ok) throw new Error("Gagal mengambil detail skill");
    return res.json();
  },

  async createSkill(payload: {
    name: string;
    display_title?: string;
    description: string;
    body: string;
    category?: string;
    always_apply?: boolean;
    user_invocable?: boolean;
    allowed_tools?: string[];
  }): Promise<{ skill: Skill }> {
    const res = await fetch("/api/skills", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal membuat skill" }));
      throw new Error(err.error || "Gagal membuat skill");
    }
    return res.json();
  },

  async updateSkill(
    id: string,
    payload: {
      expected_version: number;
      display_title?: string;
      description?: string;
      body?: string;
      category?: string;
      always_apply?: boolean;
      user_invocable?: boolean;
      allowed_tools?: string[];
    }
  ): Promise<{ skill: Skill }> {
    const res = await fetch(`/api/skills/${id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: "Gagal memperbarui skill" }));
      throw new Error(err.error || "Gagal memperbarui skill");
    }
    return res.json();
  },

  async deleteSkill(id: string): Promise<boolean> {
    const res = await fetch(`/api/skills/${id}`, { method: "DELETE" });
    if (!res.ok) throw new Error("Gagal menghapus skill");
    return true;
  },

  async toggleSkill(id: string, enabled: boolean): Promise<{ success: boolean; is_enabled: boolean }> {
    const res = await fetch(`/api/skills/${id}/toggle`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ enabled }),
    });
    if (!res.ok) throw new Error("Gagal mengubah status aktif skill");
    return res.json();
  },
};
