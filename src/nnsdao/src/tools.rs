use crate::Data;
use ic_cdk::export::Principal;
use ic_kit::ic;

pub fn is_owner() -> Result<(), String> {
    let data = ic::get::<Data>();
    let caller = ic_cdk::caller();

    data.owners.is_owner(caller)
}

pub fn log_message(
    canister: String,
    caller: Principal,
    method: String,
    kv: Vec<(String, String)>,
) -> () {
    let data = ic::get_mut::<Data>();
    data.logger.log_format_message(canister, caller, method, kv)
}
