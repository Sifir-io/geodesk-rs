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

// Note: FeatureData is defined by cxx.rs in the generated bridge code
struct FeatureData;

// Structure to represent a bounding box
struct BoundingBox {
  double west;
  double south;
  double east;
  double north;
};

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

private:
  class Impl;
  std::unique_ptr<Impl> pImpl;
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

size_t result_count(const FeatureResult &result);

std::unique_ptr<std::vector<FeatureData>>
result_to_vec(const FeatureResult &result);

} // namespace geodesk_bridge
