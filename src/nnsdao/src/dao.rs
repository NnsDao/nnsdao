use std::{collections::HashMap, string};

use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use nnsdao_sdk_basic::DaoCustomFn;
use serde::Serialize;

use crate::DAO_SERVICE_STABLE;

struct CustomDao {}

#[async_trait]
impl DaoCustomFn for CustomDao {
    async fn is_member(&self, member: Principal) -> Result<bool, String> {
        DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow().is_member(member))
    }

    async fn handle_proposal(&self) -> Result<(), String> {
        todo!()
    }
}

#[derive(CandidType, Clone, Deserialize)]
enum MemberStatusCode {
    Quit(i8),    // -1
    Default(i8), // 0
    Joined(i8),  // 1
}
impl Default for MemberStatusCode {
    fn default() -> Self {
        MemberStatusCode::Default(0)
    }
}

#[derive(CandidType, Clone, Deserialize, Default)]
struct MemberItems {
    nickname: String,
    member_status_code: MemberStatusCode,
    avatar: String,
}

#[derive(CandidType, Clone, Deserialize, Default)]
pub struct DaoService {
    owner: Option<Principal>,
    member_list: HashMap<Principal, MemberItems>,
    id: String,        // radom unique id
    name: String,      // dao name
    poster: String,    // optional dao poster
    avatar: String,    // dao avatar
    tags: Vec<String>, // dao tags
    intro: String,     // dao intro
}

impl DaoService {
    pub fn set_owner(&mut self, principal: Principal) {
        self.owner = Some(principal);
    }
    pub fn get_owner(&self) -> Option<Principal> {
        self.owner
    }

    pub fn is_owner(&self) -> Result<(), String> {
        if self.owner.unwrap() != ic_cdk::caller() {
            return Err("no auth".to_owned());
        }
        Ok(())
    }
    pub fn is_member(&self, member: Principal) -> Result<bool, String> {
        if self.member_list.contains_key(&member) {
            Ok(true)
        } else {
            Err(String::from("Users have not yet joined current DAO!"))
        }
    }
    pub fn join(&mut self, principal: Principal) -> Result<bool, String> {
        match self.is_member(principal) {
            Ok(..) => Err(String::from("Already joined!")),
            Err(..) => {
                self.member_list.insert(
                    principal,
                    MemberItems {
                        nickname: String::from("Anonymous"),
                        member_status_code: MemberStatusCode::Joined(1),
                        avatar: String::from(""),
                    },
                );
                Ok(true)
            }
        }
    }
    pub fn quit(&mut self, principal: Principal) -> Result<bool, String> {
        match self.is_member(principal) {
            Ok(..) => match self.member_list.get_mut(&principal) {
                Some(item) => {
                    item.member_status_code = MemberStatusCode::Quit(-1);
                    Ok(true)
                }
                None => Err(String::from("Failed to quit!")),
            },
            Err(..) => Err(String::from("You are not yet a member of this group!")),
        }
    }
}
