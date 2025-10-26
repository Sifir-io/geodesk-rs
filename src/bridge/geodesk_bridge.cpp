#include "geodesk_bridge.h"
#include "geodesk-rs/src/lib.rs.h" // Include generated cxx.rs header for FeatureData definition
#include <geodesk/geodesk.h>
#include <sstream>
#include <stdexcept>

using namespace geodesk;

namespace geodesk_bridge {

// FeatureStore implementation
class FeatureStore::Impl {
public:
  Impl(const std::string &gol_path) : features(gol_path.c_str()) {}

  Features features;
};

FeatureStore::FeatureStore(const std::string &gol_path)
    : pImpl(std::make_unique<Impl>(gol_path)) {}

FeatureStore::~FeatureStore() = default;

std::unique_ptr<FeatureResult>
FeatureStore::query_amenities(const std::string &amenity_type,
                              const BoundingBox &bbox) const {
  // Build GOQL query for amenities
  std::string goql_query = "na[amenity=" + amenity_type + "]";
  return query(goql_query, bbox);
}

std::unique_ptr<FeatureResult>
FeatureStore::query(const std::string &goql_query,
                    const BoundingBox &bbox) const {
  auto result = std::make_unique<FeatureResult>();

  try {
    // Create bounding box
    Box box = Box::ofWSEN(bbox.west, bbox.south, bbox.east, bbox.north);

    // Apply query filter and bounding box
    Features filtered = pImpl->features(goql_query.c_str())(box);

    // Iterate through results and collect data
    for (Feature feature : filtered) {
      FeatureData data;
      data.id = feature.id();
      data.type_name = feature.typeName();
      data.lon = feature.lon();
      data.lat = feature.lat();

      // Get name tag if available
      TagValue nameTag = feature["name"];
      if (nameTag) {
        data.name = std::string(nameTag);
      } else {
        data.name = "";
      }

      // Collect all tags
      Tags tags = feature.tags();
      for (Tag tag : tags) {
        data.tag_keys.push_back(std::string(tag.key()));
        data.tag_values.push_back(std::string(tag.value()));
      }

      // If this is a way, collect its nodes (geometry) immediately
      // This avoids the need for a separate query later
      if (feature.isWay()) {
        Nodes nodes = feature.nodes();
        for (Node node : nodes) {
          NodeData node_data;
          node_data.id = node.id();
          node_data.lon = node.lon();
          node_data.lat = node.lat();
          data.nodes.push_back(node_data);
        }
      }

      result->add_feature(std::move(data));
    }
  } catch (const std::exception &e) {
    throw std::runtime_error(std::string("Query failed: ") + e.what());
  }

  return result;
}

// FeatureResult implementation
FeatureResult::FeatureResult() = default;
FeatureResult::~FeatureResult() = default;

size_t FeatureResult::count() const { return features.size(); }

std::unique_ptr<FeatureData> FeatureResult::get(size_t index) const {
  if (index >= features.size()) {
    throw std::out_of_range("Feature index out of range");
  }
  return std::make_unique<FeatureData>(features[index]);
}

std::vector<FeatureData> FeatureResult::to_vector() const { return features; }

void FeatureResult::add_feature(FeatureData &&feature) {
  features.push_back(std::move(feature));
}

// Factory functions for cxx.rs
std::unique_ptr<FeatureStore> create_feature_store(rust::Str gol_path) {
  return std::make_unique<FeatureStore>(std::string(gol_path));
}

std::unique_ptr<FeatureResult>
query_amenities_in_bbox(const FeatureStore &store, rust::Str amenity_type,
                        double west, double south, double east, double north) {
  BoundingBox bbox{west, south, east, north};
  return store.query_amenities(std::string(amenity_type), bbox);
}

std::unique_ptr<FeatureResult> query_with_goql(const FeatureStore &store,
                                               rust::Str goql_query,
                                               double west, double south,
                                               double east, double north) {
  BoundingBox bbox{west, south, east, north};
  return store.query(std::string(goql_query), bbox);
}

size_t result_count(const FeatureResult &result) { return result.count(); }

std::unique_ptr<std::vector<FeatureData>>
result_to_vec(const FeatureResult &result) {
  return std::make_unique<std::vector<FeatureData>>(result.to_vector());
}

} // namespace geodesk_bridge
