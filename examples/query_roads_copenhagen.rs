// Example: Query road network and bus stops in Copenhagen
use geodesk_rs::{GeoDesk, BoundingBox};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the GOL file
    let geodesk = GeoDesk::open("../planet-latest.osm.gol")?;

    // Define bounding box for Copenhagen, Denmark
    // West: 12.45°E, South: 55.61°N, East: 12.65°E, North: 55.73°N
    let copenhagen_bbox = BoundingBox::new(12.45, 55.61, 12.65, 55.73);

    println!("Querying Copenhagen transportation infrastructure...\n");

    // Query roads
    println!("1. Querying roads...");
    let roads = geodesk.query_roads(copenhagen_bbox)?;
    println!("   Found {} road segments", roads.count());

    // Query bus stops
    println!("\n2. Querying bus stops...");
    let bus_stops = geodesk.query_bus_stops(copenhagen_bbox)?;
    println!("   Found {} bus stops", bus_stops.count());

    // Print first 5 bus stops
    let bus_stop_features = bus_stops.to_vec()?;
    println!("\n   First 5 bus stops:");
    for (i, stop) in bus_stop_features.iter().take(5).enumerate() {
        println!("\n   {}. {}", i + 1, stop.name);
        println!("      Location: {:.4}°, {:.4}°", stop.lon, stop.lat);
        if let Some(ref_tag) = stop.tag("ref") {
            println!("      Reference: {}", ref_tag);
        }
    }

    // Query specific road types
    println!("\n3. Querying motorways...");
    let motorways = geodesk.query("w[highway=motorway]", copenhagen_bbox)?;
    println!("   Found {} motorway segments", motorways.count());

    println!("\n4. Querying residential streets...");
    let residential = geodesk.query("w[highway=residential]", copenhagen_bbox)?;
    println!("   Found {} residential streets", residential.count());

    // Query all amenities
    println!("\n5. Querying all amenities...");
    let amenities = geodesk.query_all_amenities(copenhagen_bbox)?;
    println!("   Found {} total amenities", amenities.count());

    Ok(())
}

