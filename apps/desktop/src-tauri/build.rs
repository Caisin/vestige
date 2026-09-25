fn main() {
    println!("cargo:rustc-env=VESTIGE_TARGET={}", std::env::var("TARGET").expect("target"));
    tauri_build::build();
}
