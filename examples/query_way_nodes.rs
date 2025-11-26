use geodesk_rs::{BoundingBox, GeoDesk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the GOL file
    let geodesk = GeoDesk::open("/home/gus/data-mnt/gol-cl/planet-latest.osm.gol")?;

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

    // Query for subway/metro routes
    println!("\n\n=== QUERYING SUBWAY ROUTES ===\n");

    // Query for route relations with route=subway
    let subway_routes = geodesk.query("r[route=subway]", bbox)?;

    println!("Found {} subway routes", subway_routes.count());

    // Get the first 3 subway routes and display their information
    let routes_vec = subway_routes.to_vec()?;
    let route_sample_size = routes_vec.len().min(3);

    println!("\nDisplaying first {} subway routes:", route_sample_size);

    for (i, route) in routes_vec.iter().take(route_sample_size).enumerate() {
        println!("\n========== Subway Route {} ==========", i + 1);
        println!("ID: {}", route.id);
        println!("Type: {}", route.type_name);

        // Get route name
        if !route.name.is_empty() {
            println!("Route Name: {}", route.name);
        }

        // Get route reference (line number/code)
        if let Some(route_ref) = route.tag("ref") {
            println!("Route Reference: {}", route_ref);
        }

        // Get route color if available
        if let Some(color) = route.tag("colour") {
            println!("Route Color: {}", color);
        }

        // Get operator
        if let Some(operator) = route.tag("operator") {
            println!("Operator: {}", operator);
        }

        // Get network
        if let Some(network) = route.tag("network") {
            println!("Network: {}", network);
        }

        // Get the ways that make up this route
        // For relations, we need to query the member ways
        println!("\n--- Route Members ---");

        // Query ways that are part of this route relation
        // Note: This is a simplified approach. In a real implementation,
        // you would need to access relation members directly
        println!("Note: To get actual member ways, you would need to access");
        println!("the relation's member list through the API.");
        println!("Member ways typically include the track segments that form the route.");
    }

    // Custom query: Get all public transport routes (subway, metro, tram, light_rail)
    println!("\n\n=== CUSTOM QUERY: ALL PUBLIC TRANSPORT ROUTES ===\n");

    // Query for all public transport route types
    let transport_types = vec!["subway", "metro", "tram", "light_rail"];

    for transport_type in &transport_types {
        let query_str = format!("r[route={}]", transport_type);
        let transport_routes = geodesk.query(&query_str, bbox)?;
        let count = transport_routes.count();

        if count > 0 {
            println!("\n--- {} Routes: {} found ---", transport_type.to_uppercase(), count);

            let routes = transport_routes.to_vec()?;
            let display_count = routes.len().min(2);

            for (i, route) in routes.iter().take(display_count).enumerate() {
                println!("\n  Route {}: {}", i + 1,
                    if !route.name.is_empty() { &route.name } else { "Unnamed" });

                if let Some(route_ref) = route.tag("ref") {
                    println!("  Reference: {}", route_ref);
                }

                if let Some(color) = route.tag("colour") {
                    println!("  Color: {}", color);
                }

                // Query the ways that make up this route
                // Using a spatial query for ways with the same railway/public_transport tag
                let way_query = match *transport_type {
                    "subway" | "metro" => "w[railway=subway]",
                    "tram" => "w[railway=tram]",
                    "light_rail" => "w[railway=light_rail]",
                    _ => "w[railway]",
                };

                let route_ways = geodesk.query(way_query, bbox)?;
                let ways = route_ways.to_vec()?;

                if !ways.is_empty() {
                    println!("  Associated track segments: {} ways", ways.len());

                    // Calculate total approximate length
                    let mut total_route_length = 0.0;
                    for way in ways.iter().take(10) {
                        if way.is_way() {
                            let nodes = &way.nodes;
                            if nodes.len() > 1 {
                                for j in 0..nodes.len() - 1 {
                                    let n1 = &nodes[j];
                                    let n2 = &nodes[j + 1];
                                    let dx = n2.lon - n1.lon;
                                    let dy = n2.lat - n1.lat;
                                    total_route_length += (dx * dx + dy * dy).sqrt();
                                }
                            }
                        }
                    }
                    println!("  Approximate length (first 10 segments): {:.6} degrees", total_route_length);
                }
            }
        }
    }

    // Alternative: Query subway infrastructure directly
    println!("\n\n=== QUERYING SUBWAY INFRASTRUCTURE ===\n");

    // Query for subway tracks (ways with railway=subway)
    let subway_ways = geodesk.query("w[railway=subway]", bbox)?;

    println!("Found {} subway track segments (ways)", subway_ways.count());

    let ways_vec = subway_ways.to_vec()?;
    let way_sample_size = ways_vec.len().min(5);

    println!("\nDisplaying first {} subway track segments:", way_sample_size);

    for (i, way) in ways_vec.iter().take(way_sample_size).enumerate() {
        println!("\n--- Subway Track Segment {} ---", i + 1);
        println!("ID: {}", way.id);

        if !way.name.is_empty() {
            println!("Name: {}", way.name);
        }

        // Get tunnel information
        if let Some(tunnel) = way.tag("tunnel") {
            println!("Tunnel: {}", tunnel);
        }

        // Get usage information
        if let Some(usage) = way.tag("usage") {
            println!("Usage: {}", usage);
        }

        // Display node count and geometry
        if way.is_way() {
            let nodes = &way.nodes;
            println!("Number of nodes: {}", nodes.len());

            if !nodes.is_empty() {
                let first = &nodes[0];
                let last = &nodes[nodes.len() - 1];
                println!("First node: ID {}, Lon: {:.6}, Lat: {:.6}", first.id, first.lon, first.lat);
                println!("Last node: ID {}, Lon: {:.6}, Lat: {:.6}", last.id, last.lon, last.lat);

                // Calculate approximate length
                if nodes.len() > 1 {
                    let mut total_length = 0.0;
                    for j in 0..nodes.len() - 1 {
                        let n1 = &nodes[j];
                        let n2 = &nodes[j + 1];
                        let dx = n2.lon - n1.lon;
                        let dy = n2.lat - n1.lat;
                        let segment_length = (dx * dx + dy * dy).sqrt();
                        total_length += segment_length;
                    }
                    println!("Approximate length: {:.6} degrees", total_length);
                }
            }
        }
    }

    Ok(())
}

