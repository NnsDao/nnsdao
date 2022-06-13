use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use serde::Serialize;

/// You need to use the basic methods implemented by the party
pub trait DaoCustomFn {
    // It is used to determine whether you are DAO member of Organization A
    fn is_member(member: Principal) -> Result<bool, String>;

    // Implement specific voting methods
    fn get_equities(member: Principal) -> Result<u64, String>;

    // Implement process completed proposals
    fn handle_prposal();
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct DaoBasic<T:DaoCustomFn> {

}

#[derive(CandidType, Clone, Deserialize, Serialize, Default)]
pub struct DapService {
}

impl DaoCustomFn for DapService {
    fn is_member(member: Principal) -> Result<bool, String> {
        Ok(true)
    }

    fn get_equities(member: Principal) -> Result<u64, String> {
        todo!()
    }

    fn handle_prposal() {
        todo!()
    }
}
