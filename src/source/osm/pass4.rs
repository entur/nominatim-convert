use std::collections::{HashMap, HashSet};
use std::path::Path;

use osmpbf::{Element, ElementReader};

use crate::target::nominatim_place::NominatimPlace;

use super::coordinate::{Coordinate, CoordinateStore};
use super::entity::OsmEntityConverter;
use super::geometry::calculate_centroid;
use super::popularity::OsmPopularityCalculator;

// ---------------------------------------------------------------------------
// Pass 4 intermediate data structures
// ---------------------------------------------------------------------------

pub(crate) struct NodePoiData {
    pub(crate) ids: Vec<i64>,
    pub(crate) coords: HashMap<i64, Coordinate>,
    pub(crate) tags: HashMap<i64, Vec<(String, String)>>,
}

pub(crate) struct WayPassData {
    pub(crate) ids: Vec<i64>,
    pub(crate) way_node_ids: HashMap<i64, Vec<i64>>,
    pub(crate) way_tags: HashMap<i64, Vec<(String, String)>>,
}

pub(crate) struct RelationPassData {
    pub(crate) ids: Vec<i64>,
    pub(crate) member_node_ids: HashMap<i64, Vec<i64>>,
    pub(crate) member_way_ids: HashMap<i64, Vec<i64>>,
    pub(crate) tags: HashMap<i64, Vec<(String, String)>>,
}

// ---------------------------------------------------------------------------
// Data collection
// ---------------------------------------------------------------------------

pub(crate) fn collect_pass4_data(
    input: &Path,
    all_needed_way_ids: &HashSet<i64>,
    popularity_calculator: &OsmPopularityCalculator,
) -> Result<(NodePoiData, WayPassData, RelationPassData), Box<dyn std::error::Error>> {
    let mut node_data = NodePoiData {
        ids: Vec::new(),
        coords: HashMap::new(),
        tags: HashMap::new(),
    };
    let mut way_data = WayPassData {
        ids: Vec::new(),
        way_node_ids: HashMap::new(),
        way_tags: HashMap::new(),
    };
    let mut rel_data = RelationPassData {
        ids: Vec::new(),
        member_node_ids: HashMap::new(),
        member_way_ids: HashMap::new(),
        tags: HashMap::new(),
    };

    let reader = ElementReader::from_path(input)?;
    reader.for_each(|element| {
        match element {
            Element::Node(node) => {
                collect_poi_node(&node, popularity_calculator, &mut node_data);
            }
            Element::DenseNode(node) => {
                collect_poi_dense_node(&node, popularity_calculator, &mut node_data);
            }
            Element::Way(way) => {
                collect_way(&way, all_needed_way_ids, &mut way_data);
            }
            Element::Relation(relation) => {
                collect_poi_relation(&relation, popularity_calculator, &mut rel_data);
            }
        }
    })?;

    Ok((node_data, way_data, rel_data))
}

fn collect_poi_node(
    node: &osmpbf::Node,
    popularity_calculator: &OsmPopularityCalculator,
    node_data: &mut NodePoiData,
) {
    let tags: HashMap<&str, &str> = node.tags().collect();
    if is_poi_entity(&tags, popularity_calculator) {
        let owned_tags: Vec<(String, String)> =
            node.tags().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        node_data.ids.push(node.id());
        node_data.coords.insert(node.id(), Coordinate { lat: node.lat(), lon: node.lon() });
        node_data.tags.insert(node.id(), owned_tags);
    }
}

fn collect_poi_dense_node(
    node: &osmpbf::DenseNode,
    popularity_calculator: &OsmPopularityCalculator,
    node_data: &mut NodePoiData,
) {
    let tags: HashMap<&str, &str> = node.tags().collect();
    if is_poi_entity(&tags, popularity_calculator) {
        let owned_tags: Vec<(String, String)> =
            node.tags().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        node_data.ids.push(node.id);
        node_data.coords.insert(node.id, Coordinate { lat: node.lat(), lon: node.lon() });
        node_data.tags.insert(node.id, owned_tags);
    }
}

fn collect_way(
    way: &osmpbf::Way,
    all_needed_way_ids: &HashSet<i64>,
    way_data: &mut WayPassData,
) {
    if all_needed_way_ids.contains(&way.id()) {
        let node_ids: Vec<i64> = way.refs().collect();
        way_data.ids.push(way.id());
        way_data.way_node_ids.insert(way.id(), node_ids);
        let owned_tags: Vec<(String, String)> =
            way.tags().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        way_data.way_tags.insert(way.id(), owned_tags);
    }
}

fn collect_poi_relation(
    relation: &osmpbf::Relation,
    popularity_calculator: &OsmPopularityCalculator,
    rel_data: &mut RelationPassData,
) {
    let tags: HashMap<&str, &str> = relation.tags().collect();
    if !is_poi_entity(&tags, popularity_calculator) {
        return;
    }

    let mut member_nodes = Vec::new();
    let mut member_ways = Vec::new();
    for member in relation.members() {
        match member.member_type {
            osmpbf::RelMemberType::Node => {
                member_nodes.push(member.member_id);
            }
            osmpbf::RelMemberType::Way => {
                member_ways.push(member.member_id);
            }
            _ => {}
        }
    }
    rel_data.ids.push(relation.id());
    rel_data.member_node_ids.insert(relation.id(), member_nodes);
    rel_data.member_way_ids.insert(relation.id(), member_ways);
    let owned_tags: Vec<(String, String)> = relation
        .tags()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    rel_data.tags.insert(relation.id(), owned_tags);
}

// ---------------------------------------------------------------------------
// Centroid computation
// ---------------------------------------------------------------------------

pub(crate) fn compute_way_centroids(
    way_data: &WayPassData,
    nodes_coords: &CoordinateStore,
    way_centroids: &mut CoordinateStore,
) {
    for &way_id in &way_data.ids {
        if let Some(node_ids) = way_data.way_node_ids.get(&way_id) {
            let way_node_coords: Vec<Coordinate> = node_ids
                .iter()
                .filter_map(|&nid| nodes_coords.get(nid))
                .collect();
            if let Some(centroid) = calculate_centroid(&way_node_coords) {
                way_centroids.put(way_id, centroid);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entity conversion
// ---------------------------------------------------------------------------

pub(crate) fn convert_poi_nodes(
    node_data: &NodePoiData,
    converter: &mut OsmEntityConverter,
    results: &mut Vec<NominatimPlace>,
) {
    for &node_id in &node_data.ids {
        if let (Some(&coord), Some(owned_tags)) =
            (node_data.coords.get(&node_id), node_data.tags.get(&node_id))
        {
            let tags: HashMap<&str, &str> = owned_tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            if let Some(place) =
                converter.convert_node(node_id, coord.lat, coord.lon, &tags)
            {
                results.push(place);
            }
        }
    }
}

pub(crate) fn convert_poi_ways(
    way_data: &WayPassData,
    poi_way_ids: &HashSet<i64>,
    converter: &mut OsmEntityConverter,
    results: &mut Vec<NominatimPlace>,
) {
    for &way_id in &way_data.ids {
        if poi_way_ids.contains(&way_id)
            && let Some(owned_tags) = way_data.way_tags.get(&way_id) {
                let tags: HashMap<&str, &str> = owned_tags
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                if let Some(place) = converter.convert_way(way_id, &tags) {
                    results.push(place);
                }
            }
    }
}

pub(crate) fn convert_poi_relations(
    rel_data: &RelationPassData,
    converter: &mut OsmEntityConverter,
    results: &mut Vec<NominatimPlace>,
) {
    for &rel_id in &rel_data.ids {
        if let Some(owned_tags) = rel_data.tags.get(&rel_id) {
            let tags: HashMap<&str, &str> = owned_tags
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let member_nodes = rel_data
                .member_node_ids
                .get(&rel_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let member_ways = rel_data
                .member_way_ids
                .get(&rel_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            if let Some(place) =
                converter.convert_relation(rel_id, member_nodes, member_ways, &tags)
            {
                results.push(place);
            }
        }
    }
}

/// Check if an entity has a name and at least one matching filter tag.
fn is_poi_entity(
    tags: &HashMap<&str, &str>,
    popularity_calculator: &OsmPopularityCalculator,
) -> bool {
    tags.contains_key("name")
        && tags
            .iter()
            .any(|(k, v)| popularity_calculator.has_filter(k, v))
}
