# Upgrade Guide & Downstream Compatibility Contract

This document provides explicit instructions for upgrading **Lumis AI Gateway** across releases, adhering to the **Semantic Versioning 2.0.0** and **Zero-Breakage Downstream Contract** modeled after Laravel.

---

## 🧭 Downstream Compatibility Guarantee

Lumis provides three fundamental guarantees to downstream consumers:
1. **Automated Schema Migrations (Expand & Contract)**:
   - Upgrading between Patch and Minor versions requires **zero manual SQL execution**.
   - The database driver automatically runs idempotent migrations on startup (`driver.run_migrations()`).
   - Existing chat sessions, user accounts, and memories are strictly preserved.
2. **Environment Variable Fallback Safety**:
   - New environment variables introduced in Minor releases always include safe defaults.
   - Older `.env` configurations will not crash new container releases.
3. **Runtime Configuration Separation**:
   - Agency branding, model mappings, and theme preferences live outside source code (via `.env`, volume mounts, or admin APIs).

> [!NOTE]
> For comprehensive scenario-by-scenario instructions (Docker, Coolify, Git Fork with custom branches, MCP Tool Extenders, UI Whitelabeling, Air-Gapped enclaves), refer to [**docs/DOWNSTREAM_UPDATE_GUIDE.md**](docs/DOWNSTREAM_UPDATE_GUIDE.md).

---

## 📦 Upgrading to v1.0.0 (Baseline Release)

### Estimated Upgrade Time: < 2 Minutes
**Breaking Changes: None (Initial Certified Release)**

### 1. Docker / Coolify Deployments (Recommended)
Simply pull the official certified container image or click **Redeploy** in Coolify:
```bash
docker compose pull
docker compose up -d
```
The container will boot, run database schema migrations for `user_identities`, `scim_users`, and `scim_groups` automatically within < 50ms, and serve traffic immediately.

### 2. Upstream Git Sync for Downstream Forks
If maintaining a Git fork (e.g. `ipds6104/lumis`):
```bash
git fetch origin main
git merge origin/main -m "chore(sync): fast-forward merge from upstream v1.0.0"
git push bps main
```

---

## 🛠️ Verification Checklist After Upgrade
After deploying an upgrade, verify service health using the built-in diagnostic endpoints:
1. `GET /health` -> HTTP 200 `OK`
2. `GET /health/db` -> Verify database connection pool and latency:
   ```json
   {"dialect":"sqlite","is_alive":true,"latency_ms":0,"active_connections":3,"max_connections":10}
   ```
3. `GET /api/system/doctor` -> Verify pre-flight system diagnostics and configuration health.
