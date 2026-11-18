// Rust library for GeoDESK C++ bindings using cxx.rs

pub mod ffi {
    #[cxx::bridge(namespace = "geodesk_bridge")]
    pub mod bridge {

        // Node data for way geometry
        #[derive(Debug, Clone, Copy)]
        pub struct NodeData {
            pub id: i64,
            pub lon: f64,
            pub lat: f64,
        }

        // A single coordinate of a polygon ring (no node identity).
        #[derive(Debug, Clone, Copy)]
        pub struct CoordData {
            pub lon: f64,
            pub lat: f64,
        }

        // A polygon ring of an area feature. Outer rings describe the filled
        // area; inner rings (is_outer = false) are holes. Coordinates are
        // closed (the first vertex is repeated as the last).
        #[derive(Debug, Clone)]
        pub struct RingData {
            pub is_outer: bool,
            pub coords: Vec<CoordData>,
        }

        // Shared structs between Rust and C++
        #[derive(Debug, Clone)]
        pub struct FeatureData {
            pub id: i64,
            pub type_name: String,
            pub name: String,
            pub lon: f64,
            pub lat: f64,
            pub tag_keys: Vec<String>,
            pub tag_values: Vec<String>,
            pub nodes: Vec<NodeData>,  // Way nodes (empty for non-ways)
            pub is_area: bool,         // true for area ways and area relations
            pub rings: Vec<RingData>,  // Polygon rings (empty for non-areas)
        }

        // Opaque C++ types
        unsafe extern "C++" {
            include!("geodesk-rs/src/bridge/geodesk_bridge.h");

            type FeatureStore;
            type FeatureResult;
            type CompactWays;

            // Factory functions
            fn create_feature_store(gol_path: &str) -> Result<UniquePtr<FeatureStore>>;

            fn query_amenities_in_bbox(
                store: &FeatureStore,
                amenity_type: &str,
                west: f64,
                south: f64,
                east: f64,
                north: f64,
            ) -> Result<UniquePtr<FeatureResult>>;

            fn query_with_goql(
                store: &FeatureStore,
                goql_query: &str,
                west: f64,
                south: f64,
                east: f64,
                north: f64,
            ) -> Result<UniquePtr<FeatureResult>>;

            fn query_ways_compact(
                store: &FeatureStore,
                goql_query: &str,
                west: f64,
                south: f64,
                east: f64,
                north: f64,
                keys: &[&str],
            ) -> Result<UniquePtr<CompactWays>>;

            fn way_ids(self: &CompactWays) -> &[i64];
            fn node_ends(self: &CompactWays) -> &[u32];
            fn lons(self: &CompactWays) -> &[f64];
            fn lats(self: &CompactWays) -> &[f64];
            fn tag_bytes(self: &CompactWays) -> &[u8];
            fn tag_ends(self: &CompactWays) -> &[u32];
            fn tag_present(self: &CompactWays) -> &[u8];

            fn result_count(result: &FeatureResult) -> usize;

            fn result_to_vec(result: &FeatureResult) -> Result<UniquePtr<CxxVector<FeatureData>>>;
        }
    }
}

// SAFETY (FeatureStore: Send + Sync): the C++ `FeatureStore` wraps one
// libgeodesk `Features` collection and is only used through `const` member
// functions (`query`, `query_amenities`, `query_ways_compact`) after
// construction. libgeodesk supports concurrent queries on one GOL when built
// with GEODESK_MULTITHREADED, which CMakeLists.txt and build.rs both set (the
// bridge refuses to compile without it):
//   - the store's and matchers' reference counts are atomic;
//   - the GOL is memory-mapped read-only and tile pages are located by pure
//     arithmetic (`FreeStore::pagePointer`);
//   - matcher compilation (`MatcherCompiler::getMatcher`) and the string
//     table lookups it does keep no mutable state;
//   - the store's query thread pool is mutex-guarded, and a query only posts
//     with `tryPost` (running a tile inline when the queue is full), so
//     concurrent consumers cannot deadlock each other;
//   - each query's result queue is owned by that query.
// Dropping the store on another thread is fine: its destructor joins the
// pool's threads and takes the open-stores mutex, neither of which is tied to
// the creating thread. Nothing handed to Rust (`FeatureResult`,
// `CompactWays`) points into the store.
unsafe impl Send for ffi::bridge::FeatureStore {}
unsafe impl Sync for ffi::bridge::FeatureStore {}

// SAFETY (CompactWays: Send + Sync): a bag of `std::vector`s of plain
// numbers, filled before it is returned and never mutated afterwards; it
// holds no references into the store.
unsafe impl Send for ffi::bridge::CompactWays {}
unsafe impl Sync for ffi::bridge::CompactWays {}

use cxx::UniquePtr;
use std::path::Path;

/// Represents a bounding box in WGS84 coordinates
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west,
            south,
            east,
            north,
        }
    }

    /// Create a bounding box from center point and radius in degrees
    pub fn from_center(lon: f64, lat: f64, radius_deg: f64) -> Self {
        Self {
            west: lon - radius_deg,
            south: lat - radius_deg,
            east: lon + radius_deg,
            north: lat + radius_deg,
        }
    }
}

/// A node in a way's geometry
#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub id: i64,
    pub lon: f64,
    pub lat: f64,
}

impl From<ffi::bridge::NodeData> for Node {
    fn from(data: ffi::bridge::NodeData) -> Self {
        Node {
            id: data.id,
            lon: data.lon,
            lat: data.lat,
        }
    }
}

/// A single coordinate of a polygon ring (longitude/latitude, WGS-84).
#[derive(Debug, Clone, Copy)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

impl From<ffi::bridge::CoordData> for Coord {
    fn from(data: ffi::bridge::CoordData) -> Self {
        Coord {
            lon: data.lon,
            lat: data.lat,
        }
    }
}

/// A polygon ring of an area feature.
///
/// Outer rings (`is_outer == true`) describe the filled area; inner rings are
/// holes. Coordinates are closed: the first vertex is repeated as the last.
/// For an area mapped as a closed way there is a single outer ring; for a
/// multipolygon relation the member ways are stitched into one or more rings.
#[derive(Debug, Clone)]
pub struct Ring {
    pub is_outer: bool,
    pub coords: Vec<Coord>,
}

impl From<ffi::bridge::RingData> for Ring {
    fn from(data: ffi::bridge::RingData) -> Self {
        Ring {
            is_outer: data.is_outer,
            coords: data.coords.into_iter().map(|c| c.into()).collect(),
        }
    }
}

/// A feature from OpenStreetMap data
#[derive(Debug, Clone)]
pub struct Feature {
    pub id: i64,
    pub type_name: String,
    pub name: String,
    pub lon: f64,
    pub lat: f64,
    pub tags: Vec<(String, String)>,
    pub nodes: Vec<Node>,  // Way nodes (empty for non-ways)
    pub is_area: bool,     // true for area ways and area relations
    pub rings: Vec<Ring>,  // Polygon rings (empty for non-areas)
}

impl From<ffi::bridge::FeatureData> for Feature {
    fn from(data: ffi::bridge::FeatureData) -> Self {
        let tags = data
            .tag_keys
            .into_iter()
            .zip(data.tag_values.into_iter())
            .collect();

        // Convert NodeData to Node
        let nodes: Vec<Node> = data.nodes.into_iter().map(|n| n.into()).collect();

        // Convert polygon rings (areas only)
        let rings: Vec<Ring> = data.rings.into_iter().map(|r| r.into()).collect();

        Feature {
            id: data.id,
            type_name: data.type_name,
            name: data.name,
            lon: data.lon,
            lat: data.lat,
            tags,
            nodes,
            is_area: data.is_area,
            rings,
        }
    }
}

impl Feature {
    /// Get a tag value by key
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Check if a tag exists
    pub fn has_tag(&self, key: &str) -> bool {
        self.tags.iter().any(|(k, _)| k == key)
    }

    /// Check if this feature is a way
    pub fn is_way(&self) -> bool {
        self.type_name == "way"
    }

    /// Check if this feature is a node
    pub fn is_node(&self) -> bool {
        self.type_name == "node"
    }

    /// Check if this feature is a relation
    pub fn is_relation(&self) -> bool {
        self.type_name == "relation"
    }

    /// Check if this feature is an area (a closed-way area or an area relation
    /// such as a multipolygon / administrative boundary). Area features carry
    /// polygon geometry in [`Feature::rings`].
    pub fn is_area(&self) -> bool {
        self.is_area
    }

    /// The outer rings of this area feature (excludes holes).
    pub fn outer_rings(&self) -> impl Iterator<Item = &Ring> {
        self.rings.iter().filter(|r| r.is_outer)
    }

    /// The inner rings (holes) of this area feature.
    pub fn inner_rings(&self) -> impl Iterator<Item = &Ring> {
        self.rings.iter().filter(|r| !r.is_outer)
    }
}

/// Result of a GeoDESK query
pub struct QueryResult {
    result: UniquePtr<ffi::bridge::FeatureResult>,
}

impl QueryResult {
    fn new(result: UniquePtr<ffi::bridge::FeatureResult>) -> Self {
        Self { result }
    }

    /// Get the number of features in the result
    pub fn count(&self) -> usize {
        ffi::bridge::result_count(&self.result)
    }

    /// Convert result to a vector of features
    pub fn to_vec(&self) -> Result<Vec<Feature>, Box<dyn std::error::Error>> {
        let cpp_vec = ffi::bridge::result_to_vec(&self.result)?;
        let features: Vec<Feature> = cpp_vec.iter().map(|f| f.clone().into()).collect();
        Ok(features)
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// Ways from [`GeoDesk::query_ways_compact`], in flat arrays: way ids, node
/// coordinates, and only the tag keys that were asked for.
///
/// Compared to `query` + `to_vec`, nothing is allocated per feature and the
/// data is not copied again after the query: the arrays are filled once on
/// the C++ side and read in place through [`CompactWays::view`].
///
/// Ways are in the order the query yields them, which follows libgeodesk's
/// tile completion order and is **not** stable from run to run. Sort (e.g. by
/// way id) if order matters.
pub struct CompactWays {
    inner: UniquePtr<ffi::bridge::CompactWays>,
    key_count: usize,
}

impl CompactWays {
    /// Borrow the arrays. Cheap (a handful of FFI calls); hold on to the
    /// view rather than calling this per way.
    pub fn view(&self) -> CompactWaysView<'_> {
        let w = &*self.inner;
        CompactWaysView {
            way_ids: w.way_ids(),
            node_ends: w.node_ends(),
            lons: w.lons(),
            lats: w.lats(),
            tag_bytes: w.tag_bytes(),
            tag_ends: w.tag_ends(),
            tag_present: w.tag_present(),
            key_count: self.key_count,
        }
    }

    /// Number of ways.
    pub fn len(&self) -> usize {
        self.inner.way_ids().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Bytes held by the arrays (lengths, not capacities).
    pub fn byte_len(&self) -> usize {
        let v = self.view();
        v.way_ids.len() * 8
            + v.node_ends.len() * 4
            + (v.lons.len() + v.lats.len()) * 8
            + v.tag_bytes.len()
            + v.tag_ends.len() * 4
            + v.tag_present.len()
    }
}

/// Borrowed view of a [`CompactWays`] result. Way indices run `0..len()`;
/// key indices follow the `keys` slice passed to the query.
#[derive(Clone, Copy)]
pub struct CompactWaysView<'a> {
    way_ids: &'a [i64],
    node_ends: &'a [u32],
    lons: &'a [f64],
    lats: &'a [f64],
    tag_bytes: &'a [u8],
    tag_ends: &'a [u32],
    tag_present: &'a [u8],
    key_count: usize,
}

impl<'a> CompactWaysView<'a> {
    /// Number of ways.
    pub fn len(&self) -> usize {
        self.way_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.way_ids.is_empty()
    }

    /// All way ids, in result order.
    pub fn way_ids(&self) -> &'a [i64] {
        self.way_ids
    }

    /// OSM id of way `w`.
    pub fn way_id(&self, w: usize) -> i64 {
        self.way_ids[w]
    }

    fn node_range(&self, w: usize) -> std::ops::Range<usize> {
        let start = if w == 0 { 0 } else { self.node_ends[w - 1] as usize };
        start..self.node_ends[w] as usize
    }

    /// Longitudes of way `w`'s nodes, in way order.
    pub fn lons(&self, w: usize) -> &'a [f64] {
        &self.lons[self.node_range(w)]
    }

    /// Latitudes of way `w`'s nodes, in way order.
    pub fn lats(&self, w: usize) -> &'a [f64] {
        &self.lats[self.node_range(w)]
    }

    /// Number of nodes of way `w`.
    pub fn node_count(&self, w: usize) -> usize {
        self.node_range(w).len()
    }

    /// Nodes across all ways.
    pub fn total_nodes(&self) -> usize {
        self.lons.len()
    }

    /// Value of requested key `key` (index into the query's `keys`) on way
    /// `w`. Same answer as [`Feature::tag`] for that key on the same way:
    /// `Some` whenever the way carries the key — including values such as
    /// `"no"`, `"0"` and `""` — and `None` only when it does not.
    pub fn tag(&self, w: usize, key: usize) -> Option<&'a str> {
        assert!(key < self.key_count, "key index {key} out of range");
        let i = w * self.key_count + key;
        if self.tag_present[i] == 0 {
            return None;
        }
        let start = if i == 0 { 0 } else { self.tag_ends[i - 1] as usize };
        // Validated as UTF-8 when the result was built.
        std::str::from_utf8(&self.tag_bytes[start..self.tag_ends[i] as usize]).ok()
    }
}

/// Main interface to GeoDESK GOL files
///
/// `GeoDesk` is `Send + Sync`: one opened store can be queried from several
/// threads at once (libgeodesk is built with `GEODESK_MULTITHREADED`), so
/// parallel readers share its tile mapping and thread pool instead of each
/// opening the file again.
pub struct GeoDesk {
    store: UniquePtr<ffi::bridge::FeatureStore>,
}

impl GeoDesk {
    /// Open a GOL file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_str = path.as_ref().to_str().ok_or("Invalid path")?;
        let store = ffi::bridge::create_feature_store(path_str)?;
        Ok(Self { store })
    }

    /// Query amenities of a specific type within a bounding box
    ///
    /// # Arguments
    /// * `amenity_type` - The type of amenity (e.g., "restaurant", "cafe", "bar")
    /// * `bbox` - The bounding box to search within
    ///
    /// # Example
    /// ```no_run
    /// use geodesk_rs::{GeoDesk, BoundingBox};
    ///
    /// let geodesk = GeoDesk::open("world.gol").unwrap();
    /// let bbox = BoundingBox::new(-73.9, 45.4, -73.5, 45.7); // Montreal
    /// let restaurants = geodesk.query_amenities("restaurant", bbox).unwrap();
    /// println!("Found {} restaurants", restaurants.count());
    /// ```
    pub fn query_amenities(
        &self,
        amenity_type: &str,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let result = ffi::bridge::query_amenities_in_bbox(
            &self.store,
            amenity_type,
            bbox.west,
            bbox.south,
            bbox.east,
            bbox.north,
        )?;
        Ok(QueryResult::new(result))
    }

    /// Query features using GOQL (Geographic Object Query Language)
    ///
    /// # Arguments
    /// * `goql_query` - A GOQL query string (e.g., "na[amenity=restaurant]", "w[highway]")
    /// * `bbox` - The bounding box to search within
    ///
    /// # Example
    /// ```no_run
    /// use geodesk_rs::{GeoDesk, BoundingBox};
    ///
    /// let geodesk = GeoDesk::open("world.gol").unwrap();
    /// let bbox = BoundingBox::new(12.45, 55.61, 12.65, 55.73); // Copenhagen
    /// let roads = geodesk.query("w[highway]", bbox).unwrap();
    /// println!("Found {} roads", roads.count());
    /// ```
    pub fn query(
        &self,
        goql_query: &str,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let result = ffi::bridge::query_with_goql(
            &self.store,
            goql_query,
            bbox.west,
            bbox.south,
            bbox.east,
            bbox.north,
        )?;
        Ok(QueryResult::new(result))
    }

    /// Query ways with GOQL and return them in compact form: way id, node
    /// coordinates, and the values of `keys` only (see [`CompactWays`]).
    ///
    /// Non-way features matched by the query are skipped. Tag values are
    /// read from the way's tag table exactly as [`GeoDesk::query`] reads them,
    /// so `view().tag(w, k)` equals `Feature::tag(keys[k])` for the same way.
    ///
    /// # Example
    /// ```no_run
    /// use geodesk_rs::{GeoDesk, BoundingBox};
    ///
    /// let geodesk = GeoDesk::open("world.gol").unwrap();
    /// let bbox = BoundingBox::new(12.45, 55.61, 12.65, 55.73); // Copenhagen
    /// let ways = geodesk
    ///     .query_ways_compact("w[highway]", bbox, &["highway", "oneway"])
    ///     .unwrap();
    /// let v = ways.view();
    /// for w in 0..v.len() {
    ///     let _ = (v.way_id(w), v.lons(w), v.lats(w), v.tag(w, 1));
    /// }
    /// ```
    pub fn query_ways_compact(
        &self,
        goql_query: &str,
        bbox: BoundingBox,
        keys: &[&str],
    ) -> Result<CompactWays, Box<dyn std::error::Error>> {
        let inner = ffi::bridge::query_ways_compact(
            &self.store,
            goql_query,
            bbox.west,
            bbox.south,
            bbox.east,
            bbox.north,
            keys,
        )?;
        let ways = CompactWays {
            inner,
            key_count: keys.len(),
        };
        // `Feature` tags come through `rust::String`, which rejects invalid
        // UTF-8 (the whole query fails); hold the compact form to the same
        // rule so `CompactWaysView::tag` can hand out `&str`.
        {
            let v = ways.view();
            let mut start = 0usize;
            for &end in v.tag_ends {
                std::str::from_utf8(&v.tag_bytes[start..end as usize])
                    .map_err(|e| format!("tag value is not UTF-8: {e}"))?;
                start = end as usize;
            }
        }
        Ok(ways)
    }

    /// Query all amenities within a bounding box (any type)
    pub fn query_all_amenities(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query("na[amenity]", bbox)
    }

    /// Query restaurants within a bounding box
    pub fn query_restaurants(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query_amenities("restaurant", bbox)
    }

    /// Query cafes within a bounding box
    pub fn query_cafes(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query_amenities("cafe", bbox)
    }

    /// Query bars and pubs within a bounding box
    pub fn query_bars(&self, bbox: BoundingBox) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query("na[amenity=bar,pub]", bbox)
    }

    /// Query bus stops within a bounding box
    pub fn query_bus_stops(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query("na[highway=bus_stop]", bbox)
    }

    /// Query roads within a bounding box
    pub fn query_roads(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        self.query("w[highway]", bbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_creation() {
        let bbox = BoundingBox::new(-73.9, 45.4, -73.5, 45.7);
        assert_eq!(bbox.west, -73.9);
        assert_eq!(bbox.south, 45.4);
        assert_eq!(bbox.east, -73.5);
        assert_eq!(bbox.north, 45.7);
    }

    #[test]
    fn store_and_compact_results_are_thread_safe() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<GeoDesk>();
        send_sync::<CompactWays>();
        send_sync::<CompactWaysView<'static>>();
    }

    #[test]
    fn test_bounding_box_from_center() {
        let bbox = BoundingBox::from_center(0.0, 0.0, 1.0);
        assert_eq!(bbox.west, -1.0);
        assert_eq!(bbox.south, -1.0);
        assert_eq!(bbox.east, 1.0);
        assert_eq!(bbox.north, 1.0);
    }
}
