#include "geodesk_bridge.h"
#include "geodesk-rs/src/lib.rs.h" // Include generated cxx.rs header for FeatureData definition
#include <geodesk/geodesk.h>
#include <geodesk/feature/RelationPtr.h>
#include <geodesk/geom/Coordinate.h>
#include <geodesk/geom/Box.h>
#include <geodesk/geom/Mercator.h>
#include <geodesk/geom/polygon/Polygonizer.h>
#include "geom/polygon/RingCoordinateIterator.h" // internal header (src/ on include path)
#include <sstream>
#include <stdexcept>

using namespace geodesk;

namespace geodesk_bridge {

// Read every coordinate of a Polygonizer ring into a RingData. The iterator
// yields the ring's vertices in order, with the closing vertex repeated.
static RingData ring_to_data(const Polygonizer::Ring *ring, bool is_outer) {
  RingData out;
  out.is_outer = is_outer;
  RingCoordinateIterator iter(ring);
  int remaining = iter.coordinatesRemaining();
  for (int i = 0; i < remaining; i++) {
    Coordinate c = iter.next();
    CoordData coord;
    coord.lon = c.lon();
    coord.lat = c.lat();
    out.coords.push_back(coord);
  }
  return out;
}

// Populate FeatureData.rings with the polygon geometry of an area feature.
// Closed-way areas yield a single outer ring; area relations (multipolygons,
// administrative boundaries) are stitched from their member ways.
static void collect_rings(const Feature &feature, FeatureData &data) {
  if (!feature.isArea()) {
    return;
  }

  if (feature.isWay()) {
    // A closed-way area is a single outer ring made of the way's nodes.
    RingData ring;
    ring.is_outer = true;
    Nodes nodes = feature.nodes();
    for (Node node : nodes) {
      CoordData coord;
      coord.lon = node.lon();
      coord.lat = node.lat();
      ring.coords.push_back(coord);
    }
    data.rings.push_back(std::move(ring));
  } else if (feature.isRelation()) {
    // Stitch member ways into outer/inner rings.
    Polygonizer polygonizer;
    polygonizer.createRings(feature.store(), RelationPtr(feature.ptr()));
    for (const Polygonizer::Ring *r = polygonizer.outerRings(); r;
         r = r->next()) {
      data.rings.push_back(ring_to_data(r, true));
    }
    for (const Polygonizer::Ring *r = polygonizer.innerRings(); r;
         r = r->next()) {
      data.rings.push_back(ring_to_data(r, false));
    }
  }
}

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

      // For area features (closed ways and area relations) collect the
      // polygon rings so callers can recover boundaries (e.g. neighbourhoods,
      // administrative areas).
      data.is_area = feature.isArea();
      collect_rings(feature, data);

      // Feature bounding box (cheap — no geometry materialization). For a node
      // this is the point; for a boundary relation it is the area's extent.
      Box bounds = feature.bounds();
      data.min_lon = Mercator::lonFromX(bounds.minX());
      data.min_lat = Mercator::latFromY(bounds.minY());
      data.max_lon = Mercator::lonFromX(bounds.maxX());
      data.max_lat = Mercator::latFromY(bounds.maxY());

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
