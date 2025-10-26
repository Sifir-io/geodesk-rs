// Example: Custom GOQL queries
use geodesk_rs::{BoundingBox, GeoDesk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the GOL file
    let geodesk = GeoDesk::open("../planet-latest.osm.gol")?;

    // Define bounding box for a city (using Montreal as example)
    let bbox = BoundingBox::new(-73.9781, 45.4042, -73.4766, 45.7042);

    println!("Demonstrating various GOQL queries...\n");

    // 1. Query all restaurants, cafes, and bars
    println!("1. Restaurants, cafes, and bars:");
    let food_places = geodesk.query("na[amenity=restaurant,cafe,bar,pub]", bbox)?;
    println!("   Found {} establishments", food_places.count());

    // 2. Query parks and green spaces
    println!("\n2. Parks and green spaces:");
    let parks = geodesk.query("a[leisure=park]", bbox)?;
    println!("   Found {} parks", parks.count());

    // 3. Query schools
    println!("\n3. Schools:");
    let schools = geodesk.query("na[amenity=school]", bbox)?;
    println!("   Found {} schools", schools.count());

    // 4. Query hospitals
    println!("\n4. Hospitals:");
    let hospitals = geodesk.query("na[amenity=hospital]", bbox)?;
    println!("   Found {} hospitals", hospitals.count());

    // 5. Query parking facilities
    println!("\n5. Parking facilities:");
    let parking = geodesk.query("na[amenity=parking]", bbox)?;
    println!("   Found {} parking areas", parking.count());

    // 6. Query bicycle parking
    println!("\n6. Bicycle parking:");
    let bike_parking = geodesk.query("na[amenity=bicycle_parking]", bbox)?;
    println!("   Found {} bicycle parking spots", bike_parking.count());

    // 7. Query water features
    println!("\n7. Water features (rivers, lakes):");
    let water = geodesk.query("wa[natural=water,waterway]", bbox)?;
    println!("   Found {} water features", water.count());

    // 8. Query buildings
    println!("\n8. Buildings:");
    let buildings = geodesk.query("a[building]", bbox)?;
    println!("   Found {} buildings", buildings.count());

    // 9. Query traffic signals
    println!("\n9. Traffic signals:");
    let signals = geodesk.query("n[highway=traffic_signals]", bbox)?;
    println!("   Found {} traffic signals", signals.count());

    // 10. Query crossings
    println!("\n10. Pedestrian crossings:");
    let crossings = geodesk.query("n[highway=crossing]", bbox)?;
    println!("   Found {} crossings", crossings.count());

    // 10. Query crossings
    println!("\n10. Road Network:");
    let start = std::time::Instant::now();
    let road_network = geodesk.query("nwa[highway]", bbox)?;
    let duration = start.elapsed();
    println!("   Found {} road network elements", road_network.count());
    println!("   Query took: {:?}", duration);

    let road_features = road_network.to_vec()?;
    println!("   First 4 ways:");
    for (i, feature) in road_features.iter().take(4).enumerate() {
        println!(
            "     {}: {} ({:.4}°, {:.4}°) - {} nodes",
            i + 1,
            feature.name,
            feature.lon,
            feature.lat,
            feature.nodes.len()
        );
    }

    // Show details of first hospital found
    let hospital_features = hospitals.to_vec()?;
    if let Some(hospital) = hospital_features.first() {
        println!("\n--- Sample Hospital Details ---");
        println!("Name: {}", hospital.name);
        println!("Location: {:.4}°, {:.4}°", hospital.lon, hospital.lat);
        println!("Tags:");
        for (key, value) in &hospital.tags {
            println!("  {}: {}", key, value);
        }
    }

    Ok(())
}
