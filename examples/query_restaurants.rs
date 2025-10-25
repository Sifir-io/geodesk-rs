// Example: Query restaurants in Montreal
use geodesk_rs::{GeoDesk, BoundingBox};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the GOL file
    let geodesk = GeoDesk::open("../planet-latest.osm.gol")?;

    // Define bounding box for Montreal, Quebec
    // West: -73.9781°, South: 45.4042°, East: -73.4766°, North: 45.7042°
    let montreal_bbox = BoundingBox::new(-73.9781, 45.4042, -73.4766, 45.7042);

    println!("Querying restaurants in Montreal...");

    // Query restaurants
    let restaurants = geodesk.query_amenities("restaurant", montreal_bbox)?;

    println!("Found {} restaurants", restaurants.count());

    // Convert to vector and print first 10
    let features = restaurants.to_vec()?;
    println!("\nFirst 10 restaurants:");
    for (i, restaurant) in features.iter().take(10).enumerate() {
        println!("\n{}. {}", i + 1, restaurant.name);
        println!("   ID: {}", restaurant.id);
        println!("   Type: {}", restaurant.type_name);
        println!("   Location: {:.4}°, {:.4}°", restaurant.lon, restaurant.lat);

        // Print some interesting tags
        if let Some(cuisine) = restaurant.tag("cuisine") {
            println!("   Cuisine: {}", cuisine);
        }
        if let Some(phone) = restaurant.tag("phone") {
            println!("   Phone: {}", phone);
        }
        if let Some(website) = restaurant.tag("website") {
            println!("   Website: {}", website);
        }
    }

    Ok(())
}

