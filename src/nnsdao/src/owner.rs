use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Deserialize, Serialize, Default)]
pub struct OwnerService {
    #[serde(default)]
    pub owners: Vec<Principal>,
}

impl OwnerService {
    pub fn add_owner(&mut self, principal: Principal) -> () {
        self.owners.push(principal)
    }

    pub fn get_owners(&self) -> Vec<Principal> {
        self.owners.clone()
    }

    pub fn is_owner(&self, caller: Principal) -> Result<(), String> {
        for owner in self.owners.clone() {
            if owner == caller {
                return Ok(());
            }
        }

        Err("no auth".to_owned())
    }
}
