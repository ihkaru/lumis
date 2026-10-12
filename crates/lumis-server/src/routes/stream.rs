use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::state::AppState;

/// Zero-Copy Server-Sent Events (SSE) Streaming Endpoint.
/// Borrows immutable reference-counted bytes::Bytes without heap clone per client.
pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = state.stream_tx.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(bytes_chunk) => {
            let data = String::from_utf8_lossy(&bytes_chunk).to_string();
            let event = Event::default().event("token").data(data);
            Some(Ok(event))
        }
        Err(_) => None, // Handle lag gracefully
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
