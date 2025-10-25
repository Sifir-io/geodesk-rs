// Rust library for GeoDESK C++ bindings using cxx.rs

pub mod ffi {
    use cxx::UniquePtr;

    #[cxx::bridge(namespace = "geodesk_bridge")]
    pub mod bridge {
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
        }

        // Opaque C++ types
        unsafe extern "C++" {
            include!("geodesk-rs/src/bridge/geodesk_bridge.h");

            type FeatureStore;
            type FeatureResult;

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

            fn result_count(result: &FeatureResult) -> usize;

            fn result_to_vec(result: &FeatureResult) -> Result<UniquePtr<CxxVector<FeatureData>>>;
        }
    }
}

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

/// A feature from OpenStreetMap data
#[derive(Debug, Clone)]
pub struct Feature {
    pub id: i64,
    pub type_name: String,
    pub name: String,
    pub lon: f64,
    pub lat: f64,
    pub tags: Vec<(String, String)>,
}

impl From<ffi::bridge::FeatureData> for Feature {
    fn from(data: ffi::bridge::FeatureData) -> Self {
        let tags = data
            .tag_keys
            .into_iter()
            .zip(data.tag_values.into_iter())
            .collect();

        Feature {
            id: data.id,
            type_name: data.type_name,
            name: data.name,
            lon: data.lon,
            lat: data.lat,
            tags,
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

/// Main interface to GeoDESK GOL files
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
    pub fn query_bars(
        &self,
        bbox: BoundingBox,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
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
    fn test_bounding_box_from_center() {
        let bbox = BoundingBox::from_center(0.0, 0.0, 1.0);
        assert_eq!(bbox.west, -1.0);
        assert_eq!(bbox.south, -1.0);
        assert_eq!(bbox.east, 1.0);
        assert_eq!(bbox.north, 1.0);
    }
}
