pub const OSM_TAG_SEPARATOR: &str = ";";

pub fn join_osm_values(values: &[String]) -> Option<String> {
    let filtered: Vec<&str> = values.iter().map(|s| s.as_str()).filter(|s| !s.is_empty()).collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered.join(OSM_TAG_SEPARATOR))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_osm_values() {
        let vals = vec!["bus".to_string(), "tram".to_string()];
        assert_eq!(join_osm_values(&vals), Some("bus;tram".to_string()));
    }

    #[test]
    fn test_join_osm_values_filters_empty() {
        let vals = vec!["bus".to_string(), "".to_string(), "tram".to_string()];
        assert_eq!(join_osm_values(&vals), Some("bus;tram".to_string()));
    }

    #[test]
    fn test_join_osm_values_all_empty() {
        let vals = vec!["".to_string(), "".to_string()];
        assert_eq!(join_osm_values(&vals), None);
    }

    #[test]
    fn test_join_osm_values_empty_vec() {
        let vals: Vec<String> = vec![];
        assert_eq!(join_osm_values(&vals), None);
    }

    #[test]
    fn test_join_osm_values_single() {
        let vals = vec!["bus".to_string()];
        assert_eq!(join_osm_values(&vals), Some("bus".to_string()));
    }
}
