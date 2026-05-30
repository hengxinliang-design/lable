mod common;

use common::render_helpers;

/// Labelary reference dimensions: 101.625 mm × 203.25 mm at 8 dpmm (labels/ and root).
const LABELARY_WIDTH: u32 = 813;
const LABELARY_HEIGHT: u32 = 1626;

/// Unit test reference dimensions.
/// Labelary-bootstrapped goldens are 812 × 1624 px (101.5mm × 203.0mm at 8 dpmm).
/// Renderer-baseline goldens (produced via LABELIZE_UPDATE_GOLDEN) are 813 × 1626 px
/// (default_options canvas — same as labels/).
const UNIT_WIDTH_LABELARY: u32 = 812;
const UNIT_HEIGHT_LABELARY: u32 = 1624;
const UNIT_WIDTH_RENDERER: u32 = 813;
const UNIT_HEIGHT_RENDERER: u32 = 1626;

/// Verify that every golden PNG has the expected dimensions for its directory.
/// - `testdata/` and `testdata/labels/` → 813 × 1626 px (Labelary reference at 101.625mm × 203.25mm)
/// - `testdata/unit/` → 812 × 1624 px (Labelary) OR 813 × 1626 px (renderer baseline)
#[test]
fn all_golden_pngs_have_standard_dimensions() {
    let dir = render_helpers::testdata_dir();

    let mut checked = 0u32;
    let mut failures: Vec<String> = Vec::new();

    // testdata/ and testdata/labels/ — must be 813×1626 (Labelary at 101.625mm × 203.25mm)
    for scan_dir in &[dir.clone(), dir.join("labels")] {
        for entry in std::fs::read_dir(scan_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>")
                .to_string();
            let bytes =
                std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {}", name, e));
            let img = image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("cannot decode {}: {}", name, e));
            let (w, h) = (img.width(), img.height());
            if w != LABELARY_WIDTH || h != LABELARY_HEIGHT {
                failures.push(format!(
                    "  {} — {}×{} (expected {}×{})",
                    name, w, h, LABELARY_WIDTH, LABELARY_HEIGHT
                ));
            }
            checked += 1;
        }
    }

    // testdata/unit/ — accept either 812×1624 (Labelary bootstrap) or 813×1626 (renderer baseline)
    for entry in std::fs::read_dir(dir.join("unit"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {}", name, e));
        let img = image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!("cannot decode {}: {}", name, e));
        let (w, h) = (img.width(), img.height());
        let ok = (w == UNIT_WIDTH_LABELARY && h == UNIT_HEIGHT_LABELARY)
            || (w == UNIT_WIDTH_RENDERER && h == UNIT_HEIGHT_RENDERER);
        if !ok {
            failures.push(format!(
                "  {} — {}×{} (expected {}×{} or {}×{})",
                name,
                w,
                h,
                UNIT_WIDTH_LABELARY,
                UNIT_HEIGHT_LABELARY,
                UNIT_WIDTH_RENDERER,
                UNIT_HEIGHT_RENDERER,
            ));
        }
        checked += 1;
    }

    assert!(
        checked > 0,
        "no PNG files found in {:?} — check the testdata directory",
        dir
    );

    assert!(
        failures.is_empty(),
        "{} golden PNG(s) have non-standard dimensions:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
