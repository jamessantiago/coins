use regex::Regex;

thread_local! {
    static BASE58_RE: Regex = Regex::new(r"[1-9A-HJ-NP-Za-km-z]{32,44}").unwrap();
}

pub fn extract_addresses(text: &str) -> Vec<String> {
    BASE58_RE.with(|re| {
        let mut addresses: Vec<String> = Vec::new();
        for m in re.find_iter(text) {
            let addr = m.as_str().to_string();
            if !addresses.contains(&addr) {
                addresses.push(addr);
            }
        }
        addresses
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR_44: &str = "H8hJ9jKkMmNnPpQqRrSsTtUuVvWwXxYyZzA1aB2bC3cD";

    #[test]
    fn test_extracts_solana_addresses() {
        let text = format!("Check out this token: {ADDR_44}");
        let addrs = extract_addresses(&text);
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], ADDR_44);
    }

    #[test]
    fn test_extracts_multiple_addresses() {
        let text = format!("Addr1: {ADDR_44}\nAddr2: KkMmNnPpQqRrSsTtUuVvWwXxYyZzA1aB2bC3cD4dE");
        let addrs = extract_addresses(&text);
        assert_eq!(addrs.len(), 2);
    }

    #[test]
    fn test_dedup_addresses() {
        let text = format!("Same address twice: {ADDR_44} and again {ADDR_44}");
        let addrs = extract_addresses(&text);
        assert_eq!(addrs.len(), 1);
    }

    #[test]
    fn test_no_addresses() {
        let text = "Just some random text without any Solana addresses.";
        let addrs = extract_addresses(text);
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_skips_too_short_base58() {
        let text = "short Aa1b2c3"; // 10 chars, below 32 threshold
        let addrs = extract_addresses(text);
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_skips_lowercase_l_no_match() {
        // 'l' (lowercase L) is invalid in base58 - regex should still match
        // but this tests that the pattern works
        let text = "l1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u"; // 43 chars with 'l'
        let addrs = extract_addresses(text);
        // 'l' is NOT in the base58 alphabet, but the regex [1-9A-HJ-NP-Za-km-z]{32,44}
        // explicitly excludes 'l' via a-km-z (skips l)
        assert!(addrs.is_empty());
    }
}
