// Integration tests for area / boundary geometry extraction.
//
// These require a GOL file. Set `OSM_GOL_PATH` to a planet (or Ireland) GOL;
// when it is missing the tests skip rather than fail, so they stay friendly to
// environments without the (large) data file.

use geodesk_rs::{BoundingBox, Feature, GeoDesk};

/// Resolve the GOL path, or `None` if no usable file is present.
fn gol_path() -> Option<String> {
    let path = std::env::var("OSM_GOL_PATH").ok()?;
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// A ring is "closed" when its first and last vertices coincide.
fn ring_is_closed(coords: &[geodesk_rs::Coord]) -> bool {
    match (coords.first(), coords.last()) {
        (Some(a), Some(b)) => (a.lon - b.lon).abs() < 1e-9 && (a.lat - b.lat).abs() < 1e-9,
        _ => false,
    }
}

/// Every area feature should carry at least one outer ring, and every ring
/// should be a closed loop with at least 4 vertices (a triangle + closure).
fn assert_area_invariants(f: &Feature) {
    assert!(f.is_area, "expected area feature, got {}", f.type_name);
    assert!(
        f.outer_rings().count() >= 1,
        "area {} '{}' has no outer ring",
        f.id,
        f.name
    );
    for ring in &f.rings {
        assert!(
            ring.coords.len() >= 4,
            "ring of {} too short: {} pts",
            f.id,
            ring.coords.len()
        );
        assert!(
            ring_is_closed(&ring.coords),
            "ring of {} '{}' is not closed",
            f.id,
            f.name
        );
    }
}

// Central Dublin — has neighbourhoods mapped as both closed ways and a
// multipolygon relation ("North Dock").
const DUBLIN: BoundingBox = BoundingBox {
    west: -6.32,
    south: 53.32,
    east: -6.20,
    north: 53.38,
};

#[test]
fn neighbourhoods_have_boundaries() {
    let Some(path) = gol_path() else {
        eprintln!("skipping: OSM_GOL_PATH not set or file missing");
        return;
    };
    let geodesk = GeoDesk::open(&path).expect("open GOL");

    let features = geodesk
        .query("na[place=neighbourhood,suburb,quarter]", DUBLIN)
        .expect("query")
        .to_vec()
        .expect("to_vec");

    assert!(
        !features.is_empty(),
        "expected some neighbourhoods in central Dublin"
    );

    // Every feature flagged as an area must carry valid ring geometry.
    let areas: Vec<&Feature> = features.iter().filter(|f| f.is_area).collect();
    assert!(
        !areas.is_empty(),
        "expected at least one neighbourhood mapped as an area"
    );
    for f in &areas {
        assert_area_invariants(f);
    }
}

#[test]
fn relation_multipolygon_is_stitched() {
    let Some(path) = gol_path() else {
        eprintln!("skipping: OSM_GOL_PATH not set or file missing");
        return;
    };
    let geodesk = GeoDesk::open(&path).expect("open GOL");

    let features = geodesk
        .query("na[place=neighbourhood,suburb,quarter]", DUBLIN)
        .expect("query")
        .to_vec()
        .expect("to_vec");

    // At least one neighbourhood is mapped as a relation (multipolygon) and
    // must be stitched into closed rings by the Polygonizer.
    let rel_areas: Vec<&Feature> = features
        .iter()
        .filter(|f| f.is_relation() && f.is_area)
        .collect();
    assert!(
        !rel_areas.is_empty(),
        "expected at least one relation-area neighbourhood in Dublin"
    );
    for f in &rel_areas {
        assert_area_invariants(f);
        // A stitched relation ring must have real geometry, not just a bbox.
        let pts: usize = f.rings.iter().map(|r| r.coords.len()).sum();
        assert!(pts >= 8, "relation {} has too few ring points: {}", f.id, pts);
    }
}

#[test]
fn closed_way_area_has_single_outer_ring() {
    let Some(path) = gol_path() else {
        eprintln!("skipping: OSM_GOL_PATH not set or file missing");
        return;
    };
    let geodesk = GeoDesk::open(&path).expect("open GOL");

    // Parks are reliably mapped as closed-way areas.
    let features = geodesk
        .query("a[leisure=park]", DUBLIN)
        .expect("query")
        .to_vec()
        .expect("to_vec");

    let way_areas: Vec<&Feature> = features
        .iter()
        .filter(|f| f.is_way() && f.is_area)
        .collect();
    assert!(!way_areas.is_empty(), "expected closed-way park areas");
    for f in &way_areas {
        assert_eq!(
            f.outer_rings().count(),
            1,
            "closed-way area {} should have exactly one outer ring",
            f.id
        );
        assert_eq!(f.inner_rings().count(), 0);
        assert_area_invariants(f);
    }
}

#[test]
fn multipolygon_with_holes_reports_inner_rings() {
    let Some(path) = gol_path() else {
        eprintln!("skipping: OSM_GOL_PATH not set or file missing");
        return;
    };
    let geodesk = GeoDesk::open(&path).expect("open GOL");

    // Alaska (relation 1116270) is a multipolygon with holes — exercises the
    // inner-ring path. Query a bbox over Alaska.
    let alaska = BoundingBox::new(-170.0, 52.0, -130.0, 72.0);
    let features = geodesk
        .query("a[admin_level=4][name=Alaska]", alaska)
        .expect("query")
        .to_vec()
        .expect("to_vec");

    let Some(alaska_feat) = features.iter().find(|f| f.is_relation() && f.is_area) else {
        eprintln!("skipping: Alaska relation not present in this GOL");
        return;
    };
    assert_area_invariants(alaska_feat);
    assert!(
        alaska_feat.outer_rings().count() > 1,
        "Alaska should have multiple outer rings (islands)"
    );
    assert!(
        alaska_feat.inner_rings().count() >= 1,
        "Alaska should have at least one inner ring (hole)"
    );
}

#[test]
fn point_place_nodes_carry_no_rings() {
    let Some(path) = gol_path() else {
        eprintln!("skipping: OSM_GOL_PATH not set or file missing");
        return;
    };
    let geodesk = GeoDesk::open(&path).expect("open GOL");

    // Nodes are never areas and must not carry rings.
    let features = geodesk
        .query("n[place=neighbourhood,suburb,quarter]", DUBLIN)
        .expect("query")
        .to_vec()
        .expect("to_vec");

    for f in &features {
        assert!(f.is_node());
        assert!(!f.is_area, "node {} wrongly flagged as area", f.id);
        assert!(f.rings.is_empty(), "node {} wrongly carries rings", f.id);
    }
}
