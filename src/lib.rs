pub mod app;
pub mod events;
pub mod i18n;
pub mod model;
pub mod mouse;
pub mod storage;
pub mod sync;
pub mod ui;

/// Cargo package version (SemVer compliant, e.g. "2026.1101.1")
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the human-readable CalVer string in `YYYY.MM.DD.N` format (e.g. `2026.11.01.1`).
pub fn version() -> String {
    let pkg_ver = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = pkg_ver.split('.').collect();
    if parts.len() == 3 && parts[1].len() == 4 {
        let year = parts[0];
        let mm = &parts[1][..2];
        let dd = &parts[1][2..];
        let build = parts[2];
        format!("{}.{}.{}.{}", year, mm, dd, build)
    } else {
        pkg_ver.to_string()
    }
}

