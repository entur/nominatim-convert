/// Title-case a string: capitalize first letter of each word.
pub fn titleize(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    let lower: String = chars.map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
                    format!("{upper}{lower}")
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Round to 6 decimal places.
pub fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titleize_simple() {
        assert_eq!(titleize("hello world"), "Hello World");
    }

    #[test]
    fn test_titleize_uppercase_input() {
        assert_eq!(titleize("OSLO SENTRUM"), "Oslo Sentrum");
    }

    #[test]
    fn test_titleize_mixed_case() {
        assert_eq!(titleize("sKøYeN"), "Skøyen");
    }

    #[test]
    fn test_titleize_single_word() {
        assert_eq!(titleize("oslo"), "Oslo");
    }

    #[test]
    fn test_titleize_empty() {
        assert_eq!(titleize(""), "");
    }

    #[test]
    fn test_titleize_norwegian_chars() {
        assert_eq!(titleize("ØRSTA SENTRUM"), "Ørsta Sentrum");
        assert_eq!(titleize("ålesund"), "Ålesund");
    }

    #[test]
    fn test_round6() {
        assert_eq!(round6(59.912345678), 59.912346);
        assert_eq!(round6(10.0), 10.0);
        assert_eq!(round6(0.1234565), 0.123457); // rounds up
        assert_eq!(round6(0.1234564), 0.123456); // rounds down
    }

    #[test]
    fn test_round6_negative() {
        assert_eq!(round6(-10.123456789), -10.123457);
    }
}
