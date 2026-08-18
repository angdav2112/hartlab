fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let memory = manifest.join("../../shared/memory.x");
    println!("cargo:rerun-if-changed={}", memory.display());
    println!("cargo:rustc-link-search={}", manifest.join("../../shared").display());
    println!("cargo:rustc-link-arg=-Tmemory.x");
}
