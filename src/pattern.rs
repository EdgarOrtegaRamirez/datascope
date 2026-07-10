use regex::Regex;
use std::sync::LazyLock;

/// A detected pattern in string values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Pattern {
    Email,
    Url,
    IPv4,
    IPv6,
    UUID,
    Phone,
    CreditCard,
    ZipCode,
}

impl Pattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            Pattern::Email => "email",
            Pattern::Url => "url",
            Pattern::IPv4 => "ipv4",
            Pattern::IPv6 => "ipv6",
            Pattern::UUID => "uuid",
            Pattern::Phone => "phone",
            Pattern::CreditCard => "credit_card",
            Pattern::ZipCode => "zip_code",
        }
    }
}

// Pre-compiled regex patterns
static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(https?|ftp)://[a-zA-Z0-9.-]+(:\d+)?(/[\w./?%&=+-]*)?$").unwrap()
});

static IPV4_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$").unwrap());

static IPV6_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F:]+(:[0-9a-fA-F]+)*$|^::1$|^[0-9a-fA-F]+::[0-9a-fA-F]*$").unwrap()
});

static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .unwrap()
});

static PHONE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+?[\d\s().-]{10,15}$").unwrap());

static CREDIT_CARD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}$").unwrap());

static ZIP_CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{5}(-\d{4})?$").unwrap());

/// Detect if a value matches a known pattern.
pub fn detect_pattern(value: &str) -> Option<Pattern> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Order matters: check more specific patterns first
    if UUID_RE.is_match(trimmed) {
        return Some(Pattern::UUID);
    }
    if EMAIL_RE.is_match(trimmed) {
        return Some(Pattern::Email);
    }
    if URL_RE.is_match(trimmed) {
        return Some(Pattern::Url);
    }
    if is_valid_ipv4(trimmed) {
        return Some(Pattern::IPv4);
    }
    if IPV6_RE.is_match(trimmed) && trimmed.contains(':') {
        return Some(Pattern::IPv6);
    }
    if ZIP_CODE_RE.is_match(trimmed) {
        return Some(Pattern::ZipCode);
    }
    if CREDIT_CARD_RE.is_match(trimmed) {
        return Some(Pattern::CreditCard);
    }
    if PHONE_RE.is_match(trimmed) {
        return Some(Pattern::Phone);
    }

    None
}

/// Validate IPv4 address (each octet 0-255).
fn is_valid_ipv4(s: &str) -> bool {
    if !IPV4_RE.is_match(s) {
        return false;
    }
    s.split('.')
        .all(|octet| octet.parse::<u32>().map(|n| n <= 255).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert_eq!(detect_pattern("user@example.com"), Some(Pattern::Email));
        assert_eq!(
            detect_pattern("test.name+tag@domain.co"),
            Some(Pattern::Email)
        );
        assert_eq!(detect_pattern("not-an-email"), None);
    }

    #[test]
    fn test_url() {
        assert_eq!(detect_pattern("https://example.com"), Some(Pattern::Url));
        assert_eq!(
            detect_pattern("http://localhost:8080/path?q=1"),
            Some(Pattern::Url)
        );
        assert_eq!(detect_pattern("example.com"), None);
    }

    #[test]
    fn test_ipv4() {
        assert_eq!(detect_pattern("192.168.1.1"), Some(Pattern::IPv4));
        assert_eq!(detect_pattern("10.0.0.1"), Some(Pattern::IPv4));
        assert_eq!(detect_pattern("256.1.1.1"), None);
        assert_eq!(detect_pattern("1.2.3"), None);
    }

    #[test]
    fn test_uuid() {
        assert_eq!(
            detect_pattern("550e8400-e29b-41d4-a716-446655440000"),
            Some(Pattern::UUID)
        );
        assert_eq!(detect_pattern("not-a-uuid"), None);
    }

    #[test]
    fn test_zip_code() {
        assert_eq!(detect_pattern("12345"), Some(Pattern::ZipCode));
        assert_eq!(detect_pattern("12345-6789"), Some(Pattern::ZipCode));
    }

    #[test]
    fn test_empty() {
        assert_eq!(detect_pattern(""), None);
        assert_eq!(detect_pattern("   "), None);
    }
}
