fn main() {
    println!(
        "cargo:rustc-env=BUILD_TIME={}",
        chrono::Utc::now().to_rfc3339()
    );

    tauri_build::build()
}
