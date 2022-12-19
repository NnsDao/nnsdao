use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;

use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Deserialize, Serialize, Default, Debug)]
pub struct OwnerService {
    #[serde(default)]
    pub owners: Vec<Principal>,
}

impl OwnerService {
    pub fn add_owner(&mut self, principal: Principal) -> Vec<Principal> {
        self.owners.push(principal);
        self.get_owners()
    }

    pub fn get_owners(&self) -> Vec<Principal> {
        self.owners.clone()
    }

    pub fn is_owner(&self, caller: Principal) -> Result<(), String> {
        for owner in &self.owners {
            if *owner == caller {
                return Ok(());
            }
        }

        Err("no auth".to_owned())
    }
}
