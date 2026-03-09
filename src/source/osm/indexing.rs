use std::collections::HashMap;

use crate::common::country::Country;

use super::admin::{AdministrativeBoundary, AdministrativeBoundaryIndex};
use super::coordinate::{Coordinate, CoordinateStore};
use super::geometry::BoundingBox;
use super::street::StreetIndex;

// ---------------------------------------------------------------------------
// Intermediate data collected across passes
// ---------------------------------------------------------------------------

pub(crate) struct AdminRelationData {
    pub(crate) name: String,
    pub(crate) admin_level: i32,
    pub(crate) ref_code: String,
    pub(crate) way_ids: Vec<i64>,
    pub(crate) country: Country,
}

pub(crate) struct StreetWayData {
    pub(crate) name: String,
    pub(crate) node_ids: Vec<i64>,
}

// ---------------------------------------------------------------------------
// Index builders
// ---------------------------------------------------------------------------

pub(crate) fn build_admin_boundary_index(
    admin_relations: &[AdminRelationData],
    admin_way_node_ids: &HashMap<i64, Vec<i64>>,
    nodes_coords: &CoordinateStore,
    index: &mut AdministrativeBoundaryIndex,
) {
    for relation in admin_relations {
        let all_node_coords: Vec<Coordinate> = relation
            .way_ids
            .iter()
            .flat_map(|way_id| {
                admin_way_node_ids
                    .get(way_id)
                    .map(|nids| {
                        nids.iter()
                            .filter_map(|&nid| nodes_coords.get(nid))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect();

        if all_node_coords.is_empty() {
            continue;
        }

        let centroid = Coordinate {
            lat: all_node_coords.iter().map(|c| c.lat).sum::<f64>()
                / all_node_coords.len() as f64,
            lon: all_node_coords.iter().map(|c| c.lon).sum::<f64>()
                / all_node_coords.len() as f64,
        };
        let bbox = BoundingBox::from_coordinates(&all_node_coords);

        let boundary = AdministrativeBoundary {
            name: relation.name.clone(),
            admin_level: relation.admin_level,
            ref_code: Some(relation.ref_code.clone()),
            country: relation.country.clone(),
            centroid,
            bbox,
            boundary_nodes: all_node_coords,
        };
        index.add_boundary(boundary);
    }
}

pub(crate) fn build_street_index(
    street_ways: &[StreetWayData],
    nodes_coords: &CoordinateStore,
    index: &mut StreetIndex,
) {
    eprintln!("  Building street index...");
    let mut skipped = 0;

    for street in street_ways {
        let coordinates: Vec<Coordinate> = street
            .node_ids
            .iter()
            .filter_map(|&nid| nodes_coords.get(nid))
            .collect();

        if coordinates.len() >= 2 {
            index.add_street(&street.name, &coordinates);
        } else {
            skipped += 1;
        }
    }

    if skipped > 0 {
        eprintln!(
            "  Warning: Skipped {} streets due to missing node coordinates",
            skipped
        );
    }
}
