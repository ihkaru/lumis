/// High-performance linear O(N) PII Scanner for enterprise credentials and Indonesian identity numbers (NIK, NIP).
/// Operates without regex backtracking to prevent ReDoS attacks.

pub struct PiiScanner;

impl PiiScanner {
    /// Masks structured PII (16-digit NIK, 18-digit NIP, and email addresses).
    pub fn mask_structured_pii(text: &str) -> String {
        let masked_numbers = Self::mask_numeric_identifiers(text);
        Self::mask_emails(&masked_numbers)
    }

    /// Identifies and masks isolated 16-digit (NIK) and 18-digit (NIP) sequences.
    fn mask_numeric_identifiers(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if chars[i].is_ascii_digit() {
                // Check if preceded by alphanumeric
                let prev_is_alnum = i > 0 && chars[i - 1].is_alphanumeric();
                if !prev_is_alnum {
                    let start = i;
                    while i < len && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let digit_count = i - start;
                    let next_is_alnum = i < len && chars[i].is_alphanumeric();

                    if !next_is_alnum {
                        if digit_count == 16 {
                            result.push_str("[REDACTED_NIK]");
                            continue;
                        } else if digit_count == 18 {
                            result.push_str("[REDACTED_NIP]");
                            continue;
                        }
                    }

                    // Push original digits if not matching 16 or 18
                    for ch in &chars[start..i] {
                        result.push(*ch);
                    }
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// Masks standard email addresses (e.g. user@domain.com -> [REDACTED_EMAIL]).
    fn mask_emails(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let words: Vec<&str> = text.split(' ').collect();

        for (idx, word) in words.iter().enumerate() {
            if idx > 0 {
                result.push(' ');
            }

            if Self::is_email_like(word) {
                result.push_str("[REDACTED_EMAIL]");
            } else {
                result.push_str(word);
            }
        }

        result
    }

    fn is_email_like(token: &str) -> bool {
        let trimmed = token.trim_matches(|c: char| !c.is_alphanumeric());
        if let Some(at_idx) = trimmed.find('@') {
            if at_idx > 0 && at_idx < trimmed.len() - 1 {
                let domain_part = &trimmed[at_idx + 1..];
                if domain_part.contains('.') && !domain_part.starts_with('.') && !domain_part.ends_with('.') {
                    return true;
                }
            }
        }
        false
    }
}
