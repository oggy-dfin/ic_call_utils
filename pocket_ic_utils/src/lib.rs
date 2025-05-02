use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Gets the workspace root directory that the current package is part of.
pub fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir
        .ancestors()
        .skip(1)
        .find(|p| {
            p.join("Cargo.toml").exists()
                && std::fs::read_to_string(p.join("Cargo.toml"))
                    .map(|s| s.contains("[workspace]"))
                    .unwrap_or(false)
        })
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest_dir.clone()); // Use cloned manifest_dir if workspace root not found
    println!("Workspace dir: {:?}", workspace_dir);
    workspace_dir
}

/// Builds a specific Rust crate to Wasm.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory.
/// * `crate_name` - The name of the crate package to build.
/// * `target_arch` - The wasm target triple (e.g., "wasm32-unknown-unknown").
/// * `profile` - The build profile (e.g., "release", "dev").
/// * `features` - Optional slice of features to enable.
/// * `no_default_features` - Whether to pass `--no-default-features`.
/// * `output_dir` - The directory to copy the final Wasm artifact into.
/// * `output_filename` - The desired filename for the final Wasm artifact.
///
/// # Returns
///
/// `Ok(PathBuf)` with the path to the final Wasm artifact in `output_dir`, or `Err(String)` on failure.
pub fn build_wasm(
    workspace_root: &Path,
    crate_name: &str,
    target_arch: &str,
    profile: &str,
    features: &[&str],
    no_default_features: bool,
    output_dir: &Path,
    output_filename: &str,
) -> Result<PathBuf, String> {
    let profile_arg = format!("--profile={}", profile);
    let mut args = vec![
        "build",
        "--package",
        crate_name,
        "--target",
        target_arch,
        &profile_arg,
    ];

    if no_default_features {
        args.push("--no-default-features");
    }

    let feature_string; // Keep the string alive long enough
    if !features.is_empty() {
        feature_string = features.join(",");
        args.push("--features");
        args.push(&feature_string);
    }

    println!(
        "Building Wasm: crate='{}', target='{}', profile='{}', features={:?}, no_default={}",
        crate_name, target_arch, profile, features, no_default_features
    );
    println!("Running cargo with args: {:?}", args);

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir).map_err(|e| {
        format!(
            "Failed to create Wasm output directory {:?}: {}",
            output_dir, e
        )
    })?;

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(workspace_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            format!(
                "Failed to execute cargo build command for {}: {}",
                crate_name, e
            )
        })?;

    if !status.success() {
        return Err(format!(
            "Cargo build failed for {} with status: {} (features: {:?})",
            crate_name, status, features
        ));
    }

    // Cargo changes the crate name to use underscores instead of dashes in the output file name.
    let crate_output_name = crate_name.replace('-', "_");

    // Calculate paths after successful build
    let source_wasm_path = workspace_root
        .join("target")
        .join(target_arch)
        .join(profile)
        .join(format!("{}.wasm", crate_output_name)); // Cargo naming convention

    let final_wasm_path = output_dir.join(output_filename);

    // Check if source file exists
    if !source_wasm_path.exists() {
        return Err(format!(
            "Cargo build succeeded but output Wasm file not found at expected location: {:?}",
            source_wasm_path
        ));
    }

    // Copy the artifact
    println!(
        "Copying Wasm from {:?} to {:?}",
        source_wasm_path, final_wasm_path
    );
    std::fs::copy(&source_wasm_path, &final_wasm_path).map_err(|e| {
        format!(
            "Failed to copy Wasm from {:?} to {:?}: {}",
            source_wasm_path, final_wasm_path, e
        )
    })?;

    Ok(final_wasm_path)
}
