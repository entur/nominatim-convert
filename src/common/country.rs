use iso3166_static::{Alpha2, Alpha3};

#[derive(Debug, Clone, PartialEq)]
pub struct Country {
    pub name: String,              // 2-letter lowercase (e.g. "no")
    pub three_letter_code: String, // 3-letter uppercase (e.g. "NOR")
}

impl Country {
    pub fn no() -> Self {
        Self { name: "no".to_string(), three_letter_code: "NOR".to_string() }
    }

    pub fn parse(code: Option<&str>) -> Option<Self> {
        let code = code?;
        if code.is_empty() {
            return None;
        }
        let upper = code.to_uppercase();
        let alpha2 = Alpha2::try_from(upper.as_str()).ok()?;
        let alpha3 = Alpha3::try_from(alpha2).ok()?;
        Some(Country {
            name: code.to_lowercase(),
            three_letter_code: alpha3.as_str().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_no() {
        let c = Country::no();
        assert_eq!(c.name, "no");
        assert_eq!(c.three_letter_code, "NOR");
    }

    #[test]
    fn test_parse_norway() {
        let c = Country::parse(Some("no")).unwrap();
        assert_eq!(c.name, "no");
        assert_eq!(c.three_letter_code, "NOR");
    }

    #[test]
    fn test_parse_uppercase() {
        let c = Country::parse(Some("NO")).unwrap();
        assert_eq!(c.name, "no");
        assert_eq!(c.three_letter_code, "NOR");
    }

    #[test]
    fn test_parse_sweden() {
        let c = Country::parse(Some("se")).unwrap();
        assert_eq!(c.name, "se");
        assert_eq!(c.three_letter_code, "SWE");
    }

    #[test]
    fn test_parse_none() {
        assert!(Country::parse(None).is_none());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Country::parse(Some("xx")).is_none());
    }

    #[test]
    fn test_parse_empty() {
        assert!(Country::parse(Some("")).is_none());
    }
}
