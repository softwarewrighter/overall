// Build script to inject build timestamp and git info at compile time

use std::process::Command;

fn main() {
    // Get build timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    // Get git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT_SHORT={}", git_hash);

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
}
