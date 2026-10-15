// ==============================================================================
// Date and Time Formatting Utilities for Lumis UI (<350 lines)
// Always parses canonical UTC timestamps from backend and formats with
// the user's local system timezone via browser Intl.DateTimeFormat.
// ==============================================================================

/**
 * Formats an ISO UTC timestamp into localized short time (HH:mm)
 * using the user's client system timezone.
 */
export function formatLocalTime(iso?: string | null): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return "";
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(d);
  } catch {
    return "";
  }
}

/**
 * Formats an ISO UTC timestamp into localized date & time
 * using the user's client system timezone.
 */
export function formatLocalDateTime(iso?: string | null): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return "";
    return new Intl.DateTimeFormat(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(d);
  } catch {
    return "";
  }
}
