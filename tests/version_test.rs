use rcal::{version, PKG_VERSION};

#[test]
fn test_calver_version_format() {
    let ver = version();
    println!("Current version: {} (Cargo: {})", ver, PKG_VERSION);

    // Human-readable format: YYYY.MM.DD.N
    let parts: Vec<&str> = ver.split('.').collect();
    assert_eq!(parts.len(), 4, "Version must have 4 segments: Year.Month.Day.Number");

    let year: u32 = parts[0].parse().expect("Year must be numeric");
    let month: u32 = parts[1].parse().expect("Month must be numeric");
    let day: u32 = parts[2].parse().expect("Day must be numeric");
    let num: u32 = parts[3].parse().expect("Build number must be numeric");

    assert!(year >= 2026, "Year should be >= 2026");
    assert!((1..=12).contains(&month), "Month must be 1..=12");
    assert!((1..=31).contains(&day), "Day must be 1..=31");
    assert!(num >= 1, "Build number should be >= 1");
}
