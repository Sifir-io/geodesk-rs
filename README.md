# geodesk-rs

Rust bindings for the [GeoDESK C++ library](https://www.geodesk.com) using [cxx.rs](https://cxx.rs/).

GeoDESK is a fast and efficient library for querying OpenStreetMap data from GOL (Geographic Object Library) files. This crate provides safe Rust bindings to the GeoDESK C++ API, allowing you to query OSM features using GOQL (Geographic Object Query Language).

## Features

- 🚀 Fast queries of OpenStreetMap data
- 🗺️ Support for GOQL (Geographic Object Query Language)
- 📦 Query features by type, tags, and bounding box
- 🔍 Built-in convenience methods for common queries (restaurants, roads, bus stops, etc.)
- 🦀 Safe Rust API with zero-cost abstractions
- 🔗 Direct bindings to C++ for maximum performance

## Prerequisites

This library automatically downloads and builds GeoDESK using CMake's FetchContent feature! You only need:

1. **CMake 3.14 or higher**
   - Linux: `sudo apt install cmake` (Debian/Ubuntu) or `sudo yum install cmake` (RHEL/CentOS)
   - macOS: `brew install cmake`
   - Windows: Download from https://cmake.org/download/

2. **C++20 compatible compiler**
   - GCC 10+, Clang 11+, or MSVC 2019+

3. **A GOL file** containing OpenStreetMap data
   - Build using the `gol` command-line tool:
     ```bash
     gol build world planet-latest.osm.pbf
     ```
   - Or download pre-built GOL files from GeoDESK

**That's it!** No need to manually install GeoDESK - it's fetched automatically during build.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
geodesk-rs = "0.1.0"
```

The first build will take a few minutes as CMake downloads and compiles GeoDESK. Subsequent builds will be much faster.

## Quick Start

```rust
use geodesk_rs::{GeoDesk, BoundingBox};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a GOL file
    let geodesk = GeoDesk::open("world.gol")?;

    // Define a bounding box for Montreal, Quebec
    let bbox = BoundingBox::new(-73.9781, 45.4042, -73.4766, 45.7042);

    // Query restaurants
    let restaurants = geodesk.query_amenities("restaurant", bbox)?;
    println!("Found {} restaurants", restaurants.count());

    // Get results as a vector
    let features = restaurants.to_vec()?;
    for restaurant in features.iter().take(5) {
        println!("- {} at ({:.4}, {:.4})",
            restaurant.name, restaurant.lon, restaurant.lat);
    }

    Ok(())
}
```

## Usage Examples

### Query Amenities by Type

```rust
let bbox = BoundingBox::new(-73.9, 45.4, -73.5, 45.7);

// Query specific amenity types
let restaurants = geodesk.query_amenities("restaurant", bbox)?;
let cafes = geodesk.query_amenities("cafe", bbox)?;
let hotels = geodesk.query_amenities("hotel", bbox)?;

// Or use convenience methods
let restaurants = geodesk.query_restaurants(bbox)?;
let cafes = geodesk.query_cafes(bbox)?;
let bars = geodesk.query_bars(bbox)?;
```

### Query Roads and Transportation

```rust
let copenhagen = BoundingBox::new(12.45, 55.61, 12.65, 55.73);

// Query all roads
let roads = geodesk.query_roads(copenhagen)?;

// Query bus stops
let bus_stops = geodesk.query_bus_stops(copenhagen)?;

// Query specific road types using GOQL
let motorways = geodesk.query("w[highway=motorway]", copenhagen)?;
let residential = geodesk.query("w[highway=residential]", copenhagen)?;
```

### Custom GOQL Queries

GOQL (Geographic Object Query Language) allows powerful filtering:

```rust
// Multiple amenity types
let food = geodesk.query("na[amenity=restaurant,cafe,bar]", bbox)?;

// Parks
let parks = geodesk.query("a[leisure=park]", bbox)?;

// Schools
let schools = geodesk.query("na[amenity=school]", bbox)?;

// Water features
let water = geodesk.query("wa[natural=water,waterway]", bbox)?;

// Traffic signals
let signals = geodesk.query("n[highway=traffic_signals]", bbox)?;
```

### Working with Results

```rust
let results = geodesk.query_restaurants(bbox)?;

// Get count
println!("Found {} restaurants", results.count());

// Check if empty
if results.is_empty() {
    println!("No results found");
}

// Convert to vector and iterate
let features = results.to_vec()?;
for feature in features {
    println!("Name: {}", feature.name);
    println!("Location: ({}, {})", feature.lon, feature.lat);
    println!("Type: {}", feature.type_name);

    // Access tags
    if let Some(cuisine) = feature.tag("cuisine") {
        println!("Cuisine: {}", cuisine);
    }

    // Check if tag exists
    if feature.has_tag("wheelchair") {
        println!("Wheelchair accessible");
    }
}
```

### Bounding Box Creation

```rust
// Create from explicit coordinates (West, South, East, North)
let bbox = BoundingBox::new(-73.9, 45.4, -73.5, 45.7);

// Create from center point and radius
let bbox = BoundingBox::from_center(-73.7, 45.5, 0.2); // 0.2 degrees radius
```

## GOQL Query Syntax

GeoDESK uses GOQL (Geographic Object Query Language) for filtering features:

### Feature Types
- `n` - Nodes (points)
- `w` - Ways (lines, excluding areas)
- `a` - Areas (polygons)
- `r` - Relations (excluding areas)
- `*` - Any type

Types can be combined: `na` selects nodes and areas.

### Tag Filters

```goql
na[amenity=restaurant]              # Exact match
w[highway]                          # Tag exists
w[highway][!oneway]                 # Tag doesn't exist or is not 'yes'
na[amenity=bar,pub,cafe]           # Multiple values
w[highway=residential][name]        # Multiple conditions
na[name=The*]                       # Wildcard match
na[population>=1000000]             # Numeric comparison
```

### Examples

```goql
na[amenity=restaurant]                      # Restaurants
w[highway]                                  # All roads
na[tourism=hotel][stars>=4]                # 4+ star hotels
w[highway=residential][!oneway]            # Non-oneway residential streets
na[amenity=restaurant,cafe,bar]           # Food & drink establishments
a[leisure=park][name]                      # Named parks
n[highway=traffic_signals]                 # Traffic lights
```

## API Reference

### `GeoDesk`

Main interface to GOL files.

- `open(path)` - Open a GOL file
- `query(goql, bbox)` - Execute a GOQL query
- `query_amenities(type, bbox)` - Query amenities by type
- `query_restaurants(bbox)` - Query restaurants
- `query_cafes(bbox)` - Query cafes
- `query_bars(bbox)` - Query bars and pubs
- `query_bus_stops(bbox)` - Query bus stops
- `query_roads(bbox)` - Query roads
- `query_all_amenities(bbox)` - Query all amenities

### `BoundingBox`

Represents a geographic bounding box.

- `new(west, south, east, north)` - Create from coordinates
- `from_center(lon, lat, radius)` - Create from center and radius

### `QueryResult`

Result of a query operation.

- `count()` - Number of features
- `is_empty()` - Check if empty
- `to_vec()` - Convert to vector of features

### `Feature`

Represents an OSM feature.

- `id` - OSM ID
- `type_name` - Feature type ("node", "way", "relation")
- `name` - Name tag value
- `lon`, `lat` - Coordinates
- `tags` - All tags as key-value pairs
- `tag(key)` - Get tag value
- `has_tag(key)` - Check if tag exists

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/geodesk-rs
cd geodesk-rs

# Build (GeoDESK will be automatically fetched and built via CMake)
cargo build --release

# Run examples
cargo run --example query_restaurants
cargo run --example query_roads_copenhagen
cargo run --example custom_queries
```

**Note:** The first build will take several minutes as CMake downloads and compiles the GeoDESK C++ library automatically using FetchContent. Subsequent builds will be much faster as the library is cached.

## Examples

The `examples/` directory contains several examples:

- `query_restaurants.rs` - Query restaurants in Montreal
- `query_roads_copenhagen.rs` - Query roads and bus stops in Copenhagen
- `custom_queries.rs` - Various GOQL query examples

Run them with:

```bash
cargo run --example query_restaurants
```

## Performance

GeoDESK is designed for high-performance queries:

- Queries are executed lazily
- Spatial indexing for fast bounding box lookups
- Zero-copy access to GOL data
- Efficient tag filtering

## Troubleshooting

### "geodesk.h not found"

Make sure GeoDESK is installed and either:
- Set `GEODESK_PATH` environment variable
- Install GeoDESK headers to `/usr/local/include`

### "cannot find -lgeodesk"

Ensure the GeoDESK library is in your library path:

```bash
export LD_LIBRARY_PATH=$GEODESK_PATH/lib:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=$GEODESK_PATH/lib:$DYLD_LIBRARY_PATH  # macOS
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the same terms as the GeoDESK library.

## Resources

- [GeoDESK Website](https://www.geodesk.com)
- [GeoDESK Documentation](https://docs.geodesk.com)
- [GeoDESK C++ API](https://cppdoc.geodesk.com)
- [GOQL Reference](https://docs.geodesk.com/goql)
- [OpenStreetMap](https://www.openstreetmap.org)
- [cxx.rs Documentation](https://cxx.rs)

## Acknowledgments

- GeoDESK C++ library by Clarisma
- OpenStreetMap contributors
- cxx.rs by David Tolnay

