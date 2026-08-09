use starter_contracts::CreateNoteRequest;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("title must be between 1 and 120 characters")]
    InvalidTitle,
    #[error("body must be between 1 and 20000 characters")]
    InvalidBody,
}

pub fn validate_create_note(input: &CreateNoteRequest) -> Result<(), ValidationError> {
    let title_len = input.title.trim().chars().count();
    let body_len = input.body.trim().chars().count();
    if !(1..=120).contains(&title_len) { return Err(ValidationError::InvalidTitle); }
    if !(1..=20_000).contains(&body_len) { return Err(ValidationError::InvalidBody); }
    Ok(())
}

pub fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in value.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() { out.push(c); dash = false; }
        else if !dash && !out.is_empty() { out.push('-'); dash = true; }
    }
    out.trim_matches('-').to_string()
}

pub fn retry_delay_ms(attempt: u32) -> u64 {
    let exp = 2u64.saturating_pow(attempt.min(8));
    (250 * exp).min(60_000)
}

/// Per-owner storage quota for R2 attachments, in bytes (100 MiB).
pub const UPLOAD_QUOTA_BYTES: u64 = 100 * 1024 * 1024;

/// Whether an upload of `upload_size` bytes fits under the per-owner quota
/// given the bytes already stored. Saturating arithmetic keeps an
/// already-over-quota owner from wrapping around to "under" the limit.
pub fn within_upload_quota(current_used: u64, upload_size: u64) -> bool {
    current_used.saturating_add(upload_size) <= UPLOAD_QUOTA_BYTES
}

/// The only content types accepted for R2 attachment uploads. Keep in sync
/// with the magic-byte verification in [`verify_upload`].
pub const ALLOWED_UPLOAD_TYPES: [&str; 6] = [
    "text/plain",
    "text/markdown",
    "application/pdf",
    "image/png",
    "image/jpeg",
    "image/webp",
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UploadTypeError {
    #[error("unsupported file type")]
    UnsupportedType,
    #[error("file contents do not match the declared content type")]
    ContentTypeMismatch,
    #[error("declared text file is not valid text")]
    NotText,
}

fn is_png(b: &[u8]) -> bool {
    b.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
}

fn is_jpeg(b: &[u8]) -> bool {
    b.starts_with(&[0xFF, 0xD8, 0xFF])
}

fn is_webp(b: &[u8]) -> bool {
    b.len() >= 12 && b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP")
}

fn is_pdf(b: &[u8]) -> bool {
    b.starts_with(b"%PDF-")
}

/// Text uploads carry no reliable signature, so verification is heuristic:
/// the bytes must be valid UTF-8, contain no NUL bytes, and must not contain
/// HTML/script markers (the stored-XSS smuggling vector). This is
/// deliberately stricter than a plain text file would require.
fn is_valid_text(b: &[u8]) -> bool {
    !b.contains(&0) && std::str::from_utf8(b).is_ok()
}

fn looks_like_html(b: &[u8]) -> bool {
    const MARKERS: [&[u8]; 3] = [b"<script", b"<!doctype", b"<html"];
    MARKERS.iter().any(|m| b.windows(m.len()).any(|w| w.eq_ignore_ascii_case(m)))
}

/// Verify that the client-declared content type matches the file's actual
/// bytes. Binary formats are checked against magic signatures; text formats
/// against UTF-8/NUL/HTML heuristics. The declared type must also be in
/// [`ALLOWED_UPLOAD_TYPES`].
pub fn verify_upload(declared: &str, bytes: &[u8]) -> Result<(), UploadTypeError> {
    if !ALLOWED_UPLOAD_TYPES.contains(&declared) {
        return Err(UploadTypeError::UnsupportedType);
    }
    match declared {
        "image/png" if is_png(bytes) => Ok(()),
        "image/jpeg" if is_jpeg(bytes) => Ok(()),
        "image/webp" if is_webp(bytes) => Ok(()),
        "application/pdf" if is_pdf(bytes) => Ok(()),
        "text/plain" | "text/markdown" if is_valid_text(bytes) && !looks_like_html(bytes) => Ok(()),
        "text/plain" | "text/markdown" if !is_valid_text(bytes) => Err(UploadTypeError::NotText),
        _ => Err(UploadTypeError::ContentTypeMismatch),
    }
}

/// Length of a minted CSRF token: 64 hex characters from two UUIDv4 sources
/// (CSPRNG via Web Crypto in the Worker).
pub const CSRF_TOKEN_LENGTH: usize = 64;

/// Constant-time string comparison. CSRF tokens are secrets in the usual
/// sense, so a length leak on mismatch is acceptable but byte-timing must not
/// reveal how much of a guessed token is correct.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Whether a submitted CSRF token matches the session cookie token. Both must
/// be present and well-formed; anything else fails closed.
pub fn csrf_token_valid(submitted: Option<&str>, cookie: Option<&str>) -> bool {
    match (submitted, cookie) {
        (Some(s), Some(c)) if s.len() == CSRF_TOKEN_LENGTH => constant_time_eq(s, c),
        _ => false,
    }
}

pub fn summarize_deterministically(body: &str) -> String {
    let clean = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.chars().count() <= 180 { return clean; }
    let prefix: String = clean.chars().take(177).collect();
    format!("{prefix}...")
}

#[cfg(test)]
mod tests {
    use super::*;
    use starter_contracts::CreateNoteRequest;
    use uuid::Uuid;

    fn req(title: &str, body: &str) -> CreateNoteRequest {
        CreateNoteRequest { title: title.into(), body: body.into(), operation_id: Uuid::nil() }
    }

    #[test]
    fn rejects_blank_title() { assert_eq!(validate_create_note(&req(" ", "body")), Err(ValidationError::InvalidTitle)); }
    #[test]
    fn rejects_blank_body() { assert_eq!(validate_create_note(&req("title", " ")), Err(ValidationError::InvalidBody)); }
    #[test]
    fn slug_is_url_safe() { assert_eq!(slugify("Hello, Rust + HTMX!"), "hello-rust-htmx"); }
    #[test]
    fn backoff_is_capped() { assert_eq!(retry_delay_ms(99), 60_000); }
    #[test]
    fn summary_is_bounded() { assert!(summarize_deterministically(&"word ".repeat(100)).chars().count() <= 180); }
    #[test]
    fn quota_accepts_uploads_with_room() { assert!(within_upload_quota(0, 1024)); }
    #[test]
    fn quota_respects_exact_boundary() {
        assert!(within_upload_quota(UPLOAD_QUOTA_BYTES - 1024, 1024));
        assert!(!within_upload_quota(UPLOAD_QUOTA_BYTES - 1023, 1024));
    }
    #[test]
    fn quota_rejects_over_quota_owner() {
        assert!(!within_upload_quota(UPLOAD_QUOTA_BYTES, 1));
        assert!(!within_upload_quota(u64::MAX, 0));
    }

    // --- upload type verification ---

    #[test]
    fn png_accepts_real_signature() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
        assert_eq!(verify_upload("image/png", &png), Ok(()));
    }

    #[test]
    fn png_rejects_other_binary() {
        assert_eq!(
            verify_upload("image/png", b"\xFF\xD8\xFF\xE0"),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn jpeg_accepts_real_signature() {
        assert_eq!(verify_upload("image/jpeg", &[0xFF, 0xD8, 0xFF, 0xE0, 0x00]), Ok(()));
    }

    #[test]
    fn jpeg_rejects_short_signature() {
        // Only two of the three required magic bytes.
        assert_eq!(
            verify_upload("image/jpeg", &[0xFF, 0xD8]),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn webp_accepts_real_signature() {
        let mut webp = Vec::new();
        webp.extend_from_slice(b"RIFF");
        webp.extend_from_slice(&[0, 0, 0, 0]); // chunk size
        webp.extend_from_slice(b"WEBPVP8 ");
        assert_eq!(verify_upload("image/webp", &webp), Ok(()));
    }

    #[test]
    fn webp_rejects_riff_without_webp_marker() {
        let mut avi = Vec::new();
        avi.extend_from_slice(b"RIFF");
        avi.extend_from_slice(&[0, 0, 0, 0]);
        avi.extend_from_slice(b"AVI LIST");
        assert_eq!(
            verify_upload("image/webp", &avi),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn webp_rejects_too_short() {
        assert_eq!(
            verify_upload("image/webp", b"RIFFWEBP"),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn pdf_accepts_real_signature() {
        assert_eq!(verify_upload("application/pdf", b"%PDF-1.7\n%"), Ok(()));
    }

    #[test]
    fn pdf_rejects_non_pdf_prefix() {
        assert_eq!(
            verify_upload("application/pdf", b"%PDF 1.7"),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn plain_text_accepts_utf8() {
        assert_eq!(verify_upload("text/plain", "hello world\nsecond line".as_bytes()), Ok(()));
    }

    #[test]
    fn markdown_accepts_plain_markup() {
        assert_eq!(
            verify_upload("text/markdown", b"# Title\n\nSome *emphasis* and a [link](https://example.com)."),
            Ok(())
        );
    }

    #[test]
    fn text_rejects_embedded_script() {
        assert_eq!(
            verify_upload("text/plain", b"<script>alert(1)</script>"),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn text_rejects_html_document() {
        assert_eq!(
            verify_upload("text/plain", b"<!DOCTYPE html><html><body>x</body></html>"),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    #[test]
    fn text_rejects_nul_bytes() {
        assert_eq!(verify_upload("text/plain", b"hello\0world"), Err(UploadTypeError::NotText));
    }

    #[test]
    fn text_rejects_invalid_utf8() {
        assert_eq!(verify_upload("text/plain", &[0xFF, 0xFE, 0x00, 0x41]), Err(UploadTypeError::NotText));
    }

    #[test]
    fn html_smuggled_as_plain_text_is_rejected() {
        // The classic spoof: a real HTML document claiming to be text/plain.
        let html = b"<html><head><script src=https://evil.example/x></script></head><body>hi</body></html>";
        assert_eq!(verify_upload("text/plain", html), Err(UploadTypeError::ContentTypeMismatch));
    }

    #[test]
    fn declared_type_must_be_allowed() {
        assert_eq!(
            verify_upload("application/x-shockwave-flash", b"FWS"),
            Err(UploadTypeError::UnsupportedType)
        );
    }

    #[test]
    fn binary_mismatch_rejected() {
        // A PNG bytes buffer claiming to be a JPEG.
        assert_eq!(
            verify_upload("image/jpeg", &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Err(UploadTypeError::ContentTypeMismatch)
        );
    }

    // --- CSRF helpers ---

    #[test]
    fn constant_time_eq_matches() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }

    #[test]
    fn csrf_valid_requires_both_tokens() {
        let token = "a".repeat(CSRF_TOKEN_LENGTH);
        assert!(csrf_token_valid(Some(&token), Some(&token)));
        assert!(!csrf_token_valid(None, Some(&token)));
        assert!(!csrf_token_valid(Some(&token), None));
        assert!(!csrf_token_valid(None, None));
    }

    #[test]
    fn csrf_valid_rejects_wrong_token() {
        let a = "a".repeat(CSRF_TOKEN_LENGTH);
        let b = "b".repeat(CSRF_TOKEN_LENGTH);
        assert!(!csrf_token_valid(Some(&a), Some(&b)));
    }

    #[test]
    fn csrf_valid_rejects_malformed_length() {
        let short = "abc";
        let long = "a".repeat(CSRF_TOKEN_LENGTH + 1);
        assert!(!csrf_token_valid(Some(short), Some(short)));
        assert!(!csrf_token_valid(Some(&long), Some(&long)));
    }
}
