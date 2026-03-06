use crate::common::coordinate::Coordinate;
use crate::common::country::Country;
use country_boundaries::{CountryBoundaries, LatLon};
use std::sync::LazyLock;

thread_local! {
    static UTM33_TO_WGS84: proj::Proj = proj::Proj::new(
        "+proj=pipeline \
         +step +inv +proj=utm +zone=33 +ellps=GRS80 \
         +step +proj=longlat +datum=WGS84 \
         +step +proj=unitconvert +xy_in=rad +xy_out=deg"
    ).expect("Failed to create UTM33N -> WGS84 projection");
}

/// Convert UTM zone 33N (EPSG:25833) to WGS84 lat/lon using the proj crate.
pub fn convert_utm33_to_lat_lon(easting: f64, northing: f64) -> Coordinate {
    UTM33_TO_WGS84.with(|proj| {
        let (lon, lat) = proj.convert((easting, northing)).expect("Failed to convert coordinates");
        Coordinate::new(lat, lon)
    })
}

/// Embedded country boundaries data (same file as used by Kotlin converter).
const BOUNDARIES_DATA: &[u8] = include_bytes!("../../data/boundaries60x30.ser");

static BOUNDARIES: LazyLock<CountryBoundaries> = LazyLock::new(|| {
    CountryBoundaries::from_reader(BOUNDARIES_DATA)
        .expect("Failed to load country boundaries")
});

/// Country detection from coordinates using country-boundaries crate.
pub fn get_country(coord: &Coordinate) -> Option<Country> {
    let latlon = LatLon::new(coord.lat, coord.lon).ok()?;
    let ids = BOUNDARIES.ids(latlon);
    ids.iter()
        .find(|id| id.len() == 2)
        .and_then(|id| Country::parse(Some(id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zagreb_is_croatia() {
        let coord = Coordinate::new(45.803417, 15.992278);
        let country = get_country(&coord);
        assert_eq!(country.unwrap().name, "hr");
    }

    #[test]
    fn test_oslo_is_norway() {
        let coord = Coordinate::new(59.9139, 10.7522);
        let country = get_country(&coord);
        assert_eq!(country.unwrap().name, "no");
    }

    #[test]
    fn test_stockholm_is_sweden() {
        let coord = Coordinate::new(59.3293, 18.0686);
        let country = get_country(&coord);
        assert_eq!(country.unwrap().name, "se");
    }

    #[test]
    fn test_ocean_returns_none() {
        // Middle of the Atlantic
        let coord = Coordinate::new(40.0, -30.0);
        assert!(get_country(&coord).is_none());
    }

    #[test]
    fn test_utm33_conversion_produces_degrees() {
        // UTM33N central meridian is 15°E; easting=500000 is on the central meridian
        let coord = convert_utm33_to_lat_lon(500000.0, 6500000.0);
        // Should produce coordinates in degrees, not radians
        assert!(coord.lat > 50.0 && coord.lat < 70.0,
            "lat should be in degrees (50-70), got {}", coord.lat);
        assert!((coord.lon - 15.0).abs() < 0.01,
            "lon should be ~15° on central meridian, got {}", coord.lon);
    }

    #[test]
    fn test_utm33_conversion_norway_range() {
        // A point roughly in southern Norway
        let coord = convert_utm33_to_lat_lon(262036.0, 6651208.0);
        // Should be in Norway's latitude/longitude range
        assert!(coord.lat > 57.0 && coord.lat < 72.0,
            "lat {} should be in Norway range", coord.lat);
        assert!(coord.lon > 4.0 && coord.lon < 32.0,
            "lon {} should be in Norway range", coord.lon);
    }

    #[test]
    fn test_utm33_deterministic() {
        let c1 = convert_utm33_to_lat_lon(500000.0, 7000000.0);
        let c2 = convert_utm33_to_lat_lon(500000.0, 7000000.0);
        assert_eq!(c1.lat, c2.lat);
        assert_eq!(c1.lon, c2.lon);
    }
}
