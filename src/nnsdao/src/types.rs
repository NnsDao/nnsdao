use std::collections::HashMap;

use candid::CandidType;
use ic_kit::interfaces::management::CanisterStatusResponse;

use crate::dao::{DaoInfo, MemberItems};

// daoinfo & canister status  & memberList &

#[derive(CandidType, Clone, Debug)]
pub struct DaoData {
    pub info: DaoInfo,
    pub status: CanisterStatusResponse,
    pub owners: Vec<String>,
    pub member_list: Vec<MemberItems>,
}
