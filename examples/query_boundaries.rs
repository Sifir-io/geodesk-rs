// Explore neighbourhood / administrative boundary geometry.
//
// Run with:
//   OSM_GOL_PATH=/path/to/planet.gol cargo run --example query_boundaries

use geodesk_rs::{BoundingBox, GeoDesk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gol_path =
        std::env::var("OSM_GOL_PATH").unwrap_or_else(|_| "planet-latest.osm.gol".to_string());
    let geodesk = GeoDesk::open(&gol_path)?;

    // Central Dublin
    let bbox = BoundingBox::new(-6.32, 53.32, -6.20, 53.38);

    for query in [
        "na[place=neighbourhood,suburb,quarter]",
        "a[boundary=administrative]",
    ] {
        println!("\n=== query: {query} ===");
        let result = geodesk.query(query, bbox)?;
        let features = result.to_vec()?;
        println!("  {} features", features.len());

        let mut shown = 0;
        for f in &features {
            let total_pts: usize = f.rings.iter().map(|r| r.coords.len()).sum();
            let outers = f.outer_rings().count();
            let inners = f.inner_rings().count();
            if f.is_area && total_pts > 0 && shown < 8 {
                let first = f.rings[0].coords.first().copied();
                let last = f.rings[0].coords.last().copied();
                let closed = match (first, last) {
                    (Some(a), Some(b)) => (a.lon - b.lon).abs() < 1e-9 && (a.lat - b.lat).abs() < 1e-9,
                    _ => false,
                };
                println!(
                    "  [{}] {} '{}' place={:?} admin_level={:?} area={} outers={} inners={} pts={} closed={}",
                    f.type_name,
                    f.id,
                    f.name,
                    f.tag("place"),
                    f.tag("admin_level"),
                    f.is_area,
                    outers,
                    inners,
                    total_pts,
                    closed,
                );
                shown += 1;
            }
        }
    }

    Ok(())
}
