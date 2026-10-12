/// Semantic text chunker preserving paragraph and sentence flow.
pub struct TextChunker;

impl TextChunker {
    /// Splits input text into overlapping chunks of approximately target_size characters.
    pub fn chunk_text(content: &str, target_size: usize, overlap: usize) -> Vec<String> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        // First attempt splitting by paragraphs
        let paragraphs: Vec<&str> = trimmed.split("\n\n").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for para in paragraphs {
            if !current_chunk.is_empty() && current_chunk.len() + para.len() + 2 > target_size {
                chunks.push(current_chunk.trim().to_string());
                // Retain overlap from end of current chunk
                let overlap_start = current_chunk.len().saturating_sub(overlap);
                current_chunk = current_chunk[overlap_start..].trim_start().to_string();
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
            }

            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        if chunks.is_empty() {
            chunks.push(trimmed.to_string());
        }

        chunks
    }
}
