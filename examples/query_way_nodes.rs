use geodesk_rs::{BoundingBox, GeoDesk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the GOL file
    let geodesk = GeoDesk::open("../planet-latest.osm.gol")?;

    // Define a bounding box for Copenhagen, Denmark
    let bbox = BoundingBox::new(12.45, 55.61, 12.65, 55.73);

    println!("Querying roads in Copenhagen...");

    // Query for roads (ways with highway tag)
    let roads = geodesk.query("w[highway]", bbox)?;

    println!("Found {} roads", roads.count());

    // Get the first 5 roads and display their nodes
    let roads_vec = roads.to_vec()?;
    let sample_size = roads_vec.len().min(5);

    println!("\nDisplaying nodes for first {} roads:", sample_size);

    for (i, road) in roads_vec.iter().take(sample_size).enumerate() {
        println!("\n--- Road {} ---", i + 1);
        println!("ID: {}", road.id);
        println!("Type: {}", road.type_name);

        // Get the highway type
        if let Some(highway_type) = road.tag("highway") {
            println!("Highway type: {}", highway_type);
        }

        // Get the name if available
        if !road.name.is_empty() {
            println!("Name: {}", road.name);
        }

        // Get the nodes (geometry) of the way
        // Nodes are now automatically populated in road.nodes for ways!
        if road.is_way() {
            let nodes = &road.nodes;
            println!("Number of nodes: {}", nodes.len());

            // Display first and last few nodes
            if !nodes.is_empty() {
                println!("\nFirst node:");
                let first = &nodes[0];
                println!("  ID: {}, Lon: {:.6}, Lat: {:.6}", first.id, first.lon, first.lat);

                if nodes.len() > 1 {
                    println!("\nLast node:");
                    let last = &nodes[nodes.len() - 1];
                    println!("  ID: {}, Lon: {:.6}, Lat: {:.6}", last.id, last.lon, last.lat);
                }

                // Calculate total length (approximate)
                if nodes.len() > 1 {
                    let mut total_length = 0.0;
                    for i in 0..nodes.len() - 1 {
                        let n1 = &nodes[i];
                        let n2 = &nodes[i + 1];
                        // Simple Euclidean distance (not accurate for large distances)
                        let dx = n2.lon - n1.lon;
                        let dy = n2.lat - n1.lat;
                        let segment_length = (dx * dx + dy * dy).sqrt();
                        total_length += segment_length;
                    }
                    println!("\nApproximate length: {:.6} degrees", total_length);
                }
            }
        }
    }

    Ok(())
}

