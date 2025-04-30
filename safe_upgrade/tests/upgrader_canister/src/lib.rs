use candid::Principal;
use ic_cdk::call::{CallFailed, CallRejected, OnewayError};
use ic_call_chaos::{set_policy as cc_set_policy, Call, Policy};
use ic_call_retry::{unless_out_of_time_or_stopping, Deadline};
use ic_cdk::update;
use ic_safe_upgrades::{upgrade_canister, WasmModule, UpgradeStage};

#[update]
pub async fn try_upgrading_target(
    target_canister: Principal,
    new_wasm: Vec<u8>,
    deadline: u64,
) -> Result<(), String> {
    upgrade_canister(
        target_canister,
        WasmModule::Bytes(new_wasm),
        vec![],
        &mut unless_out_of_time_or_stopping(&Deadline::TimeOrStopping(deadline)),
    )
    .await
    .map_err(|e| format!("Failed to upgrade canister: {:?}", e))
}

struct FailAtStagePolicy {
    stage: UpgradeStage
}

impl FailAtStagePolicy {
    fn new(step: u32) -> Self {
        Self {
            stage: match step {
                0 => UpgradeStage::Stopping,
                1 => UpgradeStage::ObtainingInfo,
                2 => UpgradeStage::Installing,
                3 => UpgradeStage::Starting,
                _ => panic!("Invalid step {}", step),
            }
        }
    }
}

impl Policy for FailAtStagePolicy {
    fn allow(&mut self, call: &Call) -> Result<(), CallFailed> {
        let call_stage  = match call.method {
            "stop_canister" => UpgradeStage::Stopping,
            "canister_info" => UpgradeStage::ObtainingInfo,
            "install_code" => UpgradeStage::Installing,
            "start_canister" => UpgradeStage::Starting,
            _ => panic!("Unknown method: {}", call.method),
        };
        if call_stage == self.stage {
            Err(CallFailed::CallRejected(CallRejected::with_rejection(2, "Simulate a transient failure".to_string())))
        } else {
            Ok(())
        }

    }

    fn allow_oneway(&mut self, _call: &Call) -> Result<(), Option<OnewayError>> {
        todo!()
    }
}


#[update]
pub async fn set_call_chaos_policy(policy: String) {
    match policy.as_str() {
        "AllowAll" => cc_set_policy(ic_call_chaos::AllowAll::default()),
        "AllowEveryOther" => cc_set_policy(ic_call_chaos::AllowEveryOther::default()),
        "DenyAll" => cc_set_policy(ic_call_chaos::DenyAll::default()),
        "WithProbability" => cc_set_policy(ic_call_chaos::WithProbability::new(0.1, 1337, true)),
        _ => panic!("Unknown policy: {}", policy),
    }
}

#[update]
pub async fn set_fail_at_stage_policy(step: u32) {
    cc_set_policy(FailAtStagePolicy::new(step));
}
