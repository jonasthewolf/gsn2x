fn main() {
    // If we set CARGO_PKG_VERSION this way, then it will override the default value, which is
    // taken from the `version` in Cargo.toml.
    if let Ok(val) = std::env::var("GSN2X_VERSION") {
        if !val.is_empty() {
            let version = env!("CARGO_PKG_VERSION");
            println!("cargo:rustc-env=CARGO_PKG_VERSION={}-{}", version, val);    
        }
    }
    println!("cargo:rerun-if-env-changed=GSN2X_VERSION");
}
