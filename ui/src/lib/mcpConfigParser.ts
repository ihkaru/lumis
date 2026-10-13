export interface ParsedMcpConfig {
  name: string;
  displayTitle: string;
  description?: string;
  transportType: "sse" | "streamable_http";
  endpointUrl?: string;
  authType: "none" | "bearer" | "basic" | "oauth2";
  rawCredentials?: string;
  isMandatory?: boolean;
  rateLimitRpm?: number;
  dailyLimit?: number;
}

/**
 * Intelligent parser supporting:
 * 1. Claude Desktop / Cursor / LibreChat standard {"mcpServers": { "name": { "url": "...", ... } }}
 * 2. Flat single server object {"name": "...", "endpoint_url": "...", ...}
 */
export function parseMcpJson(rawJson: string): ParsedMcpConfig {
  const trimmed = rawJson.trim();
  if (!trimmed) {
    throw new Error("JSON konfigurasi tidak boleh kosong");
  }

  const parsed = JSON.parse(trimmed);

  // Format 1: Claude Desktop / Cursor `mcpServers` dictionary
  if (parsed.mcpServers && typeof parsed.mcpServers === "object") {
    const keys = Object.keys(parsed.mcpServers);
    if (keys.length === 0) {
      throw new Error("Objek 'mcpServers' tidak memuat server apapun");
    }
    const serverKey = keys[0];
    const s = parsed.mcpServers[serverKey];

    const endpointUrl = s.url || s.endpoint_url || s.endpoint || "";
    const transportType = (s.transport === "streamable_http" || s.transport_type === "streamable_http")
      ? "streamable_http"
      : "sse";

    // Detect credentials from headers if provided
    let authType: "none" | "bearer" | "basic" | "oauth2" = "none";
    let rawCredentials = "";

    if (s.headers && typeof s.headers === "object") {
      const authHeader = s.headers["Authorization"] || s.headers["authorization"] || "";
      if (authHeader.startsWith("Bearer ")) {
        authType = "bearer";
        rawCredentials = authHeader.replace("Bearer ", "").trim();
      } else if (authHeader.startsWith("Basic ")) {
        authType = "basic";
        rawCredentials = authHeader.replace("Basic ", "").trim();
      }
    }

    return {
      name: serverKey.toLowerCase().replace(/[^a-z0-9_-]/g, "_"),
      displayTitle: s.title || s.display_title || serverKey.charAt(0).toUpperCase() + serverKey.slice(1),
      description: s.description || undefined,
      transportType,
      endpointUrl,
      authType,
      rawCredentials: rawCredentials || undefined,
      isMandatory: !!s.is_mandatory,
      rateLimitRpm: s.rate_limit_rpm ? Number(s.rate_limit_rpm) : undefined,
      dailyLimit: s.daily_request_limit ? Number(s.daily_request_limit) : undefined,
    };
  }

  // Format 2: Direct Single Server Object
  if (typeof parsed === "object" && !Array.isArray(parsed)) {
    const name = parsed.name || parsed.id || "custom_mcp";
    const endpointUrl = parsed.endpoint_url || parsed.url || "";
    const transportType = (parsed.transport_type === "streamable_http" || parsed.transport === "streamable_http")
      ? "streamable_http"
      : "sse";
    const authType = parsed.auth_type || (parsed.raw_credentials ? "bearer" : "none");

    return {
      name: String(name).toLowerCase().replace(/[^a-z0-9_-]/g, "_"),
      displayTitle: parsed.display_title || parsed.title || name,
      description: parsed.description || undefined,
      transportType,
      endpointUrl,
      authType,
      rawCredentials: parsed.raw_credentials || undefined,
      isMandatory: !!parsed.is_mandatory,
      rateLimitRpm: parsed.rate_limit_rpm ? Number(parsed.rate_limit_rpm) : undefined,
      dailyLimit: parsed.daily_request_limit ? Number(parsed.daily_request_limit) : undefined,
    };
  }

  throw new Error("Format JSON tidak valid. Gunakan format 'mcpServers' atau objek server.");
}

export function getSampleMcpJson(): string {
  return JSON.stringify(
    {
      mcpServers: {
        propylon: {
          url: "https://mcp.dvlpid.my.id/sse",
          transport: "sse",
          description: "Propylon Enterprise Gateway Control Plane (27 tools)",
        },
      },
    },
    null,
    2
  );
}
