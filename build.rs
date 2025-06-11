fn main() {
    // Rebuild if i18n files change
    println!("cargo:rerun-if-changed=i18n");
    println!("cargo:rerun-if-changed=i18n.toml");
}
