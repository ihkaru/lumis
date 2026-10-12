use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// High-performance Keyset (Cursor) Query parameters.
/// Guarantees O(log N) seek time regardless of pagination depth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorQuery {
    pub cursor_time: Option<DateTime<Utc>>,
    pub cursor_id: Option<String>,
    pub limit: usize,
}

impl Default for CursorQuery {
    fn default() -> Self {
        Self {
            cursor_time: None,
            cursor_id: None,
            limit: 20,
        }
    }
}

/// Keyset Paginated Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor_time: Option<DateTime<Utc>>,
    pub next_cursor_id: Option<String>,
    pub has_more: bool,
}
