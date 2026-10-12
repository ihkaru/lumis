export type MessageRole = "user" | "assistant" | "system" | "tool";

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: string;
  model_id?: string;
  token_count?: number;
  metadata?: Record<string, unknown>;
  created_at: string;
}

export interface Session {
  id: string;
  user_id: string;
  title: string;
  model_id: string;
  system_prompt?: string;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface CursorPage<T> {
  items: T[];
  next_cursor_time?: string;
  next_cursor_id?: string;
  has_more: boolean;
}

export interface User {
  id: string;
  username: string;
  email?: string;
  role: "super_admin" | "tenant_admin" | "power_user" | "member" | "guest";
  tenant_id?: string;
}
