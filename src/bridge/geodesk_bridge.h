#pragma once

#include "rust/cxx.h"
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace geodesk_bridge {

// Forward declarations
class FeatureStore;
class FeatureResult;

// Note: FeatureData and NodeData are defined by cxx.rs in the generated bridge code
struct FeatureData;
struct NodeData;

// Structure to represent a bounding box
struct BoundingBox {
  double west;
  double south;
  double east;
  double north;
};

class CompactWays;

// Main wrapper class for Features collection
class FeatureStore {
public:
  FeatureStore(const std::string &gol_path);
  ~FeatureStore();

  // Query amenities within a bounding box
  std::unique_ptr<FeatureResult>
  query_amenities(const std::string &amenity_type,
                  const BoundingBox &bbox) const;

  // Generic query with GOQL
  std::unique_ptr<FeatureResult> query(const std::string &goql_query,
                                       const BoundingBox &bbox) const;

  // Ways matching a GOQL query, as flat arrays: way ids, node coordinates
  // and the values of the requested tag keys only (see CompactWays).
  std::unique_ptr<CompactWays>
  query_ways_compact(const std::string &goql_query, const BoundingBox &bbox,
                     const std::vector<std::string> &keys) const;

private:
  class Impl;
  std::unique_ptr<Impl> pImpl;
};

// Result of FeatureStore::query_ways_compact. Immutable once built; holds
// plain copies (no pointers into the store), so it outlives the store and can
// move between threads.
//
// Layout, for way w (0-based) and requested key k (0-based, K keys):
//   way_ids[w]
//   node coordinates: lons/lats[node_ends[w-1] .. node_ends[w]]
//                     (node_ends[-1] = 0)
//   tag value bytes:  tag_bytes[tag_ends[i-1] .. tag_ends[i]], i = w*K + k
//                     (tag_ends[-1] = 0)
//   tag_present[i]:   1 if the way has a tag with key k (first occurrence
//                     wins), else 0. Presence is decided by the key, not by
//                     the value's truthiness.
class CompactWays {
public:
  rust::Slice<const int64_t> way_ids() const;
  rust::Slice<const uint32_t> node_ends() const;
  rust::Slice<const double> lons() const;
  rust::Slice<const double> lats() const;
  rust::Slice<const uint8_t> tag_bytes() const;
  rust::Slice<const uint32_t> tag_ends() const;
  rust::Slice<const uint8_t> tag_present() const;

  std::vector<int64_t> way_ids_;
  std::vector<uint32_t> node_ends_;
  std::vector<double> lons_;
  std::vector<double> lats_;
  std::vector<uint8_t> tag_bytes_;
  std::vector<uint32_t> tag_ends_;
  std::vector<uint8_t> tag_present_;
};

// Result set wrapper
class FeatureResult {
public:
  FeatureResult();
  ~FeatureResult();

  size_t count() const;
  std::unique_ptr<FeatureData> get(size_t index) const;
  std::vector<FeatureData> to_vector() const;

  // Internal method to add features
  void add_feature(FeatureData &&feature);

private:
  std::vector<FeatureData> features;
};

// C++ factory functions for cxx.rs
std::unique_ptr<FeatureStore> create_feature_store(rust::Str gol_path);

std::unique_ptr<FeatureResult>
query_amenities_in_bbox(const FeatureStore &store, rust::Str amenity_type,
                        double west, double south, double east, double north);

std::unique_ptr<FeatureResult> query_with_goql(const FeatureStore &store,
                                               rust::Str goql_query,
                                               double west, double south,
                                               double east, double north);

std::unique_ptr<CompactWays>
query_ways_compact(const FeatureStore &store, rust::Str goql_query,
                   double west, double south, double east, double north,
                   rust::Slice<const rust::Str> keys);

size_t result_count(const FeatureResult &result);

std::unique_ptr<std::vector<FeatureData>>
result_to_vec(const FeatureResult &result);

} // namespace geodesk_bridge
