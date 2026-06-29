// Survey how a city's *extent* shows up across countries: the admin-boundary
// stack containing a point, with each boundary's bbox via Feature::bounds().
//   OSM_GOL_PATH=/path/planet.gol cargo run --release --example city_extent

use geodesk_rs::{BoundingBox, GeoDesk};

fn km_lat(d: f64) -> f64 {
    d * 111.0
}
fn km_lon(d: f64, lat: f64) -> f64 {
    d * 111.0 * lat.to_radians().cos()
}

struct City {
    name: &'static str,
    lat: f64,
    lon: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gol_path =
        std::env::var("OSM_GOL_PATH").unwrap_or_else(|_| "planet-latest.osm.gol".to_string());
    let geodesk = GeoDesk::open(&gol_path)?;

    let cities = [
        City { name: "Montreal", lat: 45.5019, lon: -73.5674 },
        City { name: "Ottawa", lat: 45.4215, lon: -75.6972 },
        City { name: "Chelsea, QC", lat: 45.5037, lon: -75.7794 },
        City { name: "Amman", lat: 31.9544, lon: 35.9106 },
        City { name: "San Francisco", lat: 37.7749, lon: -122.4194 },
        City { name: "London", lat: 51.5074, lon: -0.1278 },
        City { name: "Jakarta", lat: -6.2088, lon: 106.8456 },
    ];

    for c in &cities {
        println!("\n######## {} ({:.4}, {:.4}) ########", c.name, c.lat, c.lon);

        // The gazetteer label point nearest the city centre.
        let pad = 0.05;
        let pt_box = BoundingBox::new(c.lon - pad, c.lat - pad, c.lon + pad, c.lat + pad);
        if let Ok(pts) = geodesk.query("n[place=city,town,municipality]", pt_box) {
            if let Ok(v) = pts.to_vec() {
                if let Some(p) = v
                    .iter()
                    .min_by(|a, b| {
                        let da = (a.lat - c.lat).powi(2) + (a.lon - c.lon).powi(2);
                        let db = (b.lat - c.lat).powi(2) + (b.lon - c.lon).powi(2);
                        da.partial_cmp(&db).unwrap()
                    })
                {
                    println!(
                        "  label node: '{}' place={:?} pop={:?} wikidata={:?}",
                        p.name,
                        p.tag("place"),
                        p.tag("population"),
                        p.tag("wikidata"),
                    );
                }
            }
        }

        // The admin-boundary stack: every boundary whose bbox contains the
        // point, smallest first. Search a generous region so large units are
        // seen, then rank by extent.
        let r = 1.5;
        let region = BoundingBox::new(c.lon - r, c.lat - r, c.lon + r, c.lat + r);
        let feats = geodesk.query("a[boundary=administrative]", region)?.to_vec()?;

        let mut hits: Vec<_> = feats
            .iter()
            .filter(|f| {
                let b = &f.bounds;
                c.lon >= b.west && c.lon <= b.east && c.lat >= b.south && c.lat <= b.north
            })
            .map(|f| {
                let b = &f.bounds;
                let w = km_lon(b.east - b.west, c.lat);
                let h = km_lat(b.north - b.south);
                (w * h, w, h, f)
            })
            .collect();
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        println!("  admin stack containing the point (smallest → largest):");
        for (_area, w, h, f) in hits.iter().take(8) {
            // Skip obvious bbox false-positives (country/continent-scale or
            // antimeridian-wrapped relations) for the "city" discussion.
            if *w > 4000.0 {
                continue;
            }
            println!(
                "    admin_level={:<4} {:<30} {:>7.1} x {:<7.1} km  place={:<10?} wikidata={:?}",
                f.tag("admin_level").unwrap_or("-"),
                if f.name.is_empty() { "(unnamed)" } else { &f.name },
                w,
                h,
                f.tag("place").unwrap_or("-"),
                f.tag("wikidata").unwrap_or("-"),
            );
        }
    }

    Ok(())
}
