// Integration tests for `GeoDesk::query_ways_compact` and for sharing one
// store across threads.
//
// They need a GOL file: set `OSM_GOL_PATH` to a planet (or Canada) GOL. When
// it is missing the tests skip rather than fail.

use geodesk_rs::{BoundingBox, Feature, GeoDesk};
use std::collections::{HashMap, HashSet};

fn gol_path() -> Option<String> {
    let path = std::env::var("OSM_GOL_PATH").ok()?;
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        eprintln!("OSM_GOL_PATH {path} not found; skipping");
        None
    }
}

// Ottawa core: ~140k highway ways, mixed urban/suburban tagging.
const OTTAWA: BoundingBox = BoundingBox {
    west: -75.80,
    south: 45.35,
    east: -75.60,
    north: 45.48,
};

// Every highway way, so the comparison sees all tag shapes, not only the
// road classes a router keeps.
const QUERY: &str = "w[highway]";

// The five keys a road-graph builder reads, plus `layer` (often "0" or a
// negative number, i.e. stored as a numeric tag value).
const KEYS: [&str; 6] = ["highway", "maxspeed", "surface", "oneway", "access", "layer"];

fn reference(gd: &GeoDesk, bbox: BoundingBox) -> Vec<Feature> {
    gd.query(QUERY, bbox).unwrap().to_vec().unwrap()
}

#[test]
fn compact_matches_feature_api_field_by_field() {
    let Some(path) = gol_path() else { return };
    let gd = GeoDesk::open(&path).unwrap();

    let features = reference(&gd, OTTAWA);
    let ways: HashMap<i64, &Feature> = features
        .iter()
        .filter(|f| f.is_way())
        .map(|f| (f.id, f))
        .collect();
    assert!(ways.len() > 50_000, "bbox too small: {} ways", ways.len());

    let compact = gd.query_ways_compact(QUERY, OTTAWA, &KEYS).unwrap();
    let v = compact.view();
    assert_eq!(v.len(), compact.len());
    assert_eq!(v.len(), ways.len(), "same number of ways");

    let mut seen = HashSet::new();
    let mut counts: HashMap<(&str, &str), usize> = HashMap::new();
    let mut total_nodes = 0;
    for w in 0..v.len() {
        let id = v.way_id(w);
        assert!(seen.insert(id), "way {id} returned twice");
        let f = ways.get(&id).unwrap_or_else(|| panic!("way {id} not in query()"));

        // Coordinates: same nodes, same order, same bits.
        let (lons, lats) = (v.lons(w), v.lats(w));
        assert_eq!(lons.len(), f.nodes.len(), "way {id} node count");
        assert_eq!(v.node_count(w), f.nodes.len());
        for (i, n) in f.nodes.iter().enumerate() {
            assert_eq!(lons[i].to_bits(), n.lon.to_bits(), "way {id} node {i} lon");
            assert_eq!(lats[i].to_bits(), n.lat.to_bits(), "way {id} node {i} lat");
        }
        total_nodes += lons.len();

        // Tags: exactly what Feature::tag says, presence included.
        for (k, key) in KEYS.iter().enumerate() {
            assert_eq!(v.tag(w, k), f.tag(key), "way {id} tag {key}");
            if let Some(val) = v.tag(w, k) {
                *counts.entry((key, val)).or_default() += 1;
            }
        }
    }
    assert_eq!(v.total_nodes(), total_nodes);

    // Values a truthiness test would drop ("no") and numeric values must be
    // present here (and were compared above). Numeric 0 is covered by
    // `compact_tag_semantics_for_falsy_values`: Ottawa has no `layer=0` road.
    for (key, val) in [("access", "no"), ("oneway", "no"), ("maxspeed", "50")] {
        let n = counts.get(&(key, val)).copied().unwrap_or(0);
        eprintln!("{key}={val}: {n} ways");
        assert!(n > 0, "expected some {key}={val} ways in the bbox");
    }
}

#[test]
fn compact_tag_semantics_for_falsy_values() {
    // libgeodesk's TagValue converts to `false` for "no" and for numeric 0.
    // Ways carrying such a value must read back as Some(value), never None,
    // and agree with Feature::tag. `access=0` / `oneway=0` / `maxspeed=0` do
    // not occur on roads in these cities (checked: Ottawa region, Montreal,
    // New York, Paris, Berlin, Tokyo), so numeric 0 is exercised through
    // `layer=0`, stored as a number in the GOL like any other key's 0 would be.
    let Some(path) = gol_path() else { return };
    let gd = GeoDesk::open(&path).unwrap();
    let ottawa_region = BoundingBox::new(-76.5, 45.0, -75.0, 46.0);
    let paris = BoundingBox::new(1.7, 48.5, 2.7, 49.1);
    let keys = ["highway", "maxspeed", "surface", "oneway", "access", "layer"];
    for (query, bbox, key, value) in [
        ("w[highway][access=no]", ottawa_region, 4, "no"),
        ("w[highway][oneway=no]", ottawa_region, 3, "no"),
        ("w[highway][layer=0]", paris, 5, "0"),
    ] {
        let features = gd.query(query, bbox).unwrap().to_vec().unwrap();
        let compact = gd.query_ways_compact(query, bbox, &keys).unwrap();
        let v = compact.view();
        assert!(v.len() > 100, "{query}: only {} ways", v.len());
        let by_id: HashMap<i64, usize> = (0..v.len()).map(|w| (v.way_id(w), w)).collect();
        assert_eq!(by_id.len(), features.iter().filter(|f| f.is_way()).count());
        let mut exact = 0;
        for f in features.iter().filter(|f| f.is_way()) {
            let w = by_id[&f.id];
            // GOQL also matches list values such as "0;1", so only count the
            // exact ones; every way must still carry the key.
            assert!(v.tag(w, key).is_some(), "{query}: way {} lost its value", f.id);
            exact += usize::from(v.tag(w, key) == Some(value));
            for (k, name) in keys.iter().enumerate() {
                assert_eq!(v.tag(w, k), f.tag(name), "{query}: way {} {name}", f.id);
            }
        }
        eprintln!("{query}: {} ways, {exact} exactly {value:?}", v.len());
        assert!(exact > 100, "{query}: only {exact} ways with exactly {value:?}");
    }
}

#[test]
fn absent_keys_and_empty_results() {
    let Some(path) = gol_path() else { return };
    let gd = GeoDesk::open(&path).unwrap();
    // A key no road carries, and a key list that is empty.
    let compact = gd
        .query_ways_compact("w[highway=residential]", OTTAWA, &["no:such:key"])
        .unwrap();
    let v = compact.view();
    assert!(v.len() > 1000);
    assert!((0..v.len()).all(|w| v.tag(w, 0).is_none()));
    let compact = gd
        .query_ways_compact("w[highway=residential]", OTTAWA, &[])
        .unwrap();
    assert_eq!(compact.view().len(), v.len());
    // Nodes only (not ways): an empty result, not an error.
    let compact = gd.query_ways_compact("n[amenity=cafe]", OTTAWA, &KEYS).unwrap();
    assert!(compact.is_empty());
    assert_eq!(compact.view().total_nodes(), 0);
}

#[test]
fn one_store_shared_across_threads() {
    // k×k sub-boxes queried concurrently on ONE store give the same ways as a
    // single query of the whole box (a way crossing a sub-box edge comes back
    // from each box it touches; de-duplicate by id).
    let Some(path) = gol_path() else { return };
    let gd = GeoDesk::open(&path).unwrap();
    let whole = gd.query_ways_compact(QUERY, OTTAWA, &KEYS).unwrap();
    let wv = whole.view();
    let whole_ids: HashMap<i64, usize> = (0..wv.len()).map(|w| (wv.way_id(w), w)).collect();

    let k = 3;
    let (dw, dh) = (
        (OTTAWA.east - OTTAWA.west) / k as f64,
        (OTTAWA.north - OTTAWA.south) / k as f64,
    );
    let mut subs = Vec::new();
    for i in 0..k {
        for j in 0..k {
            subs.push(BoundingBox::new(
                OTTAWA.west + dw * i as f64,
                OTTAWA.south + dh * j as f64,
                if i + 1 == k { OTTAWA.east } else { OTTAWA.west + dw * (i + 1) as f64 },
                if j + 1 == k { OTTAWA.north } else { OTTAWA.south + dh * (j + 1) as f64 },
            ));
        }
    }
    // Run the sub-queries twice over, so more than one query is in flight on
    // the store most of the time.
    let parts: Vec<_> = std::thread::scope(|s| {
        let gd = &gd;
        let handles: Vec<_> = subs
            .iter()
            .chain(subs.iter())
            .map(|b| s.spawn(move || gd.query_ways_compact(QUERY, *b, &KEYS).unwrap()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut union = HashSet::new();
    for part in &parts {
        let pv = part.view();
        for w in 0..pv.len() {
            let id = pv.way_id(w);
            let ww = *whole_ids
                .get(&id)
                .unwrap_or_else(|| panic!("way {id} from a sub-box is not in the whole box"));
            assert_eq!(pv.lons(w), wv.lons(ww));
            assert_eq!(pv.lats(w), wv.lats(ww));
            for key in 0..KEYS.len() {
                assert_eq!(pv.tag(w, key), wv.tag(ww, key));
            }
            union.insert(id);
        }
    }
    assert_eq!(union.len(), whole_ids.len(), "sub-boxes cover the whole box");

    // Results and the store itself can be moved to, and dropped on, another
    // thread.
    let moved = std::thread::spawn(move || {
        let n = gd.query_ways_compact(QUERY, subs[0], &KEYS).unwrap().len();
        drop(gd);
        (n, parts)
    })
    .join()
    .unwrap();
    assert!(moved.0 > 0);
}
