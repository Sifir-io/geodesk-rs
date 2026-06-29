// Measure how many "place" features exist worldwide and how big a name index
// would be. Run with:
//   OSM_GOL_PATH=/path/planet.gol cargo run --release --example count_places

use geodesk_rs::{BoundingBox, GeoDesk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gol_path =
        std::env::var("OSM_GOL_PATH").unwrap_or_else(|_| "planet-latest.osm.gol".to_string());
    let geodesk = GeoDesk::open(&gol_path)?;

    // Whole world (Web-Mercator valid latitude range).
    let world = BoundingBox::new(-180.0, -85.0, 180.0, 85.0);

    // Measure each tier separately so we can see where the row count explodes.
    for place in ["city", "town", "municipality", "village"] {
        let q = format!("n[place={place}]");
        let result = geodesk.query(&q, world)?;
        let features = result.to_vec()?;

        let total = features.len();
        let named: Vec<&geodesk_rs::Feature> =
            features.iter().filter(|f| !f.name.is_empty()).collect();
        let with_pop = named.iter().filter(|f| f.has_tag("population")).count();

        // Bytes of UTF-8 name text (the variable part of a row, and what a
        // trigram index is built over).
        let name_bytes: usize = named.iter().map(|f| f.name.len()).sum();
        let avg_name = if named.is_empty() {
            0.0
        } else {
            name_bytes as f64 / named.len() as f64
        };

        println!(
            "place={place:<13} total={total:>8}  named={:>8}  with_population={:>8}  name_bytes={:>10}  avg_name={:.1}",
            named.len(),
            with_pop,
            name_bytes,
            avg_name,
        );
    }

    Ok(())
}
