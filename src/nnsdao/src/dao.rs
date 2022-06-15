use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use serde::Serialize;

/// You need to use the basic methods implemented by the party
pub trait DaoCustomFn {
    // It is used to determine whether you are DAO member of Organization A
    fn is_member(&self, member: Principal) -> Result<bool, String>;

    // Implement specific voting methods
    fn get_equities(member: Principal) -> Result<u64, String>;

    // Implement process completed proposals
    fn handle_proposal();
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct DaoBasic<T: DaoCustomFn> {
    custom_fn: T,
}

#[derive(CandidType, Clone, Deserialize, Serialize)]

enum StatusCode {
    Quit,
    Default,
    Joined,
}

struct MemberItems {
    nickname: String,
    status_code: StatusCode,
    avatar: String,
}
pub struct DapService {
    member_list: HashMap<Principal, MemberItems>,
}

impl DaoCustomFn for DapService {
    fn is_member(&self, member: Principal) -> Result<bool, String> {
        if self.member_list.contains_key(&member) {
            Ok(true)
        } else {
            Err(String::from("Users have not yet joined current DAO!"))
        }
    }

    fn get_equities(member: Principal) -> Result<u64, String> {
        todo!()
    }

    fn handle_proposal() {
        todo!()
    }
}

impl DapService {
    pub fn join(&mut self, principal: Principal) -> Result<bool, String> {
        match self.is_member(principal) {
            Ok(..) => Err(String::from("Already joined!")),
            Err(..) => {
                self.member_list.insert(
                    principal,
                    MemberItems {
                        nickname: String::from("Anonymous"),
                        status_code: StatusCode::Joined,
                        avatar: String::from(""),
                    },
                );
                Ok(true)
            }
        }
    }
    pub fn quit(&mut self, principal: Principal) -> Result<bool, String> {
        match self.is_member(principal) {
            Ok(..) => match self.member_list.remove(&principal) {
                Some(_) => Ok(true),
                None => Err(String::from("Failed to quit!")),
            },
            Err(..) => Err(String::from("You are not yet a member of this group!")),
        }
    }
}
