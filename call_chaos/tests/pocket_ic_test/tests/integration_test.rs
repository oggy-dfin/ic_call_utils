use candid::{decode_args, encode_one, Principal};
use once_cell::sync::Lazy;
use pocket_ic::PocketIc;
use pocket_ic_utils::{build_wasm, get_workspace_root};
use std::path::PathBuf;

// --- Constants ---
const CRATE_NAME: &str = "call-chaos-test-canister";
const FEATURE_NAME: &str = "use_call_chaos";
const TARGET_ARCH: &str = "wasm32-unknown-unknown";

static WORKSPACE_ROOT: Lazy<PathBuf> = Lazy::new(|| get_workspace_root());

// Define output paths within the target directory
static WASM_OUTPUT_DIR: Lazy<PathBuf> =
    Lazy::new(|| WORKSPACE_ROOT.join("target/test-wasm-artifacts"));
static WASM_NO_FEATURE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    build_wasm(
        &WORKSPACE_ROOT,
        CRATE_NAME,
        TARGET_ARCH,
        "release",
        &[],
        false,
        &WASM_OUTPUT_DIR,
        &format!("{}_no_feature.wasm", CRATE_NAME),
    )
    .expect("Failed to build Wasm artifact with no feature")
});
static WASM_WITH_FEATURE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    build_wasm(
        &WORKSPACE_ROOT,
        CRATE_NAME,
        TARGET_ARCH,
        "release",
        &[FEATURE_NAME],
        false,
        &WASM_OUTPUT_DIR,
        &format!("{}_with_feature.wasm", CRATE_NAME),
    )
    .expect("Failed to build Wasm artifact with feature")
});

fn call_ping(
    pic: &PocketIc,
    canister_id: Principal,
    times: u32,
) -> Result<(u32, u32, u32), String> {
    let response = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "call_ping",
            encode_one(&times).expect("Couldn't encode times"),
        )
        .expect("Failed to call counter canister");

    decode_args(&response).map_err(|e| format!("Failed to decode response: {}", e))
}

#[test]
fn test_without_call_chaos() -> Result<(), String> {
    println!("Uploading wasm with path: {:?}", *WASM_NO_FEATURE_PATH);
    let wasm_bytes_no_feature = std::fs::read(&*WASM_NO_FEATURE_PATH).map_err(|e| {
        format!(
            "Failed to read Wasm (no feature) {:?}: {}",
            *WASM_NO_FEATURE_PATH, e
        )
    })?;

    let pic = PocketIc::new();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm_bytes_no_feature, vec![], None);

    let times = 10_u32;

    let (succeeded, failed, nr_pings) = call_ping(&pic, canister_id, times)?;

    assert_eq!(succeeded, times);
    assert_eq!(nr_pings, times);
    assert_eq!(failed, 0);

    Ok(())
}

#[test]
fn test_with_call_chaos() -> Result<(), String> {
    println!("Uploading wasm with path: {:?}", *WASM_WITH_FEATURE_PATH);
    let wasm_bytes_with_feature = std::fs::read(&*WASM_WITH_FEATURE_PATH).map_err(|e| {
        format!(
            "Failed to read Wasm (with feature) {:?}: {}",
            *WASM_WITH_FEATURE_PATH, e
        )
    })?;

    let pic = PocketIc::new();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm_bytes_with_feature, vec![], None);

    let times = 10_u32;

    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "set_policy",
        encode_one("AllowAll").expect("Couldn't encode policy"),
    )
    .expect("Failed to set policy");

    let (succeeded, failed, nr_pings): (u32, u32, u32) = call_ping(&pic, canister_id, times)?;

    assert_eq!(succeeded + failed, times);
    assert_eq!(succeeded, times);
    assert_eq!(failed, 0);
    assert_eq!(nr_pings, times);

    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "set_policy",
        encode_one("AllowEveryOther").expect("Couldn't encode policy"),
    )
    .expect("Failed to set policy");

    let (succeeded, failed, nr_pings): (u32, u32, u32) = call_ping(&pic, canister_id, times)?;
    assert_eq!(succeeded + failed, times);
    assert_eq!(succeeded, times / 2);
    assert_eq!(failed, times / 2);
    assert_eq!(nr_pings, times / 2);

    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "set_policy",
        encode_one("DenyAll").expect("Couldn't encode policy"),
    )
    .expect("Failed to set policy");

    let (succeeded, failed, nr_pings): (u32, u32, u32) = call_ping(&pic, canister_id, times)?;
    assert_eq!(succeeded + failed, times);
    assert_eq!(succeeded, 0);
    assert_eq!(failed, times);
    assert_eq!(nr_pings, 0);

    pic.update_call(
        canister_id,
        Principal::anonymous(),
        "set_policy",
        encode_one("WithProbability").expect("Couldn't encode policy"),
    )
    .expect("Failed to set policy");

    let (succeeded, failed, nr_pings): (u32, u32, u32) = call_ping(&pic, canister_id, times)?;
    // Can't assert the exact number of succeeded and failed calls, but we can assert that
    // the sum of succeeded and failed is equal to times
    assert_eq!(succeeded + failed, times);
    assert!(nr_pings <= times);

    Ok(())
}
