use crate::Data;
use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_kit::ic;
use nnsdao_sdk_basic::DaoCustomFn;
use serde::Serialize;
use std::collections::HashMap;

struct CustomDao {}

#[async_trait]
impl DaoCustomFn for CustomDao {
    async fn is_member(&self, member: Principal) -> Result<bool, String> {
        let data = ic::get_mut::<Data>();
        data.dao.is_member(member)
    }

    async fn handle_proposal(&self) -> Result<(), String> {
        todo!()
    }
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub enum MemberStatusCode {
    Quit(i8),    // -1
    Default(i8), // 0
    Joined(i8),  // 1
}

impl Default for MemberStatusCode {
    fn default() -> Self {
        MemberStatusCode::Default(0)
    }
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
pub struct MemberItems {
    nickname: String,
    member_status_code: MemberStatusCode,
    avatar: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
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
        self.member_list
            .get(&member)
            .ok_or(String::from("Users have not yet joined current DAO!"))?;

        Ok(true)
    }

    pub fn join(&mut self, principal: Principal) -> Result<MemberItems, String> {
        let member = self.member_list.get(&principal).map_or(
            Ok(MemberItems {
                nickname: String::from("Anonymous"),
                member_status_code: MemberStatusCode::Joined(1),
                avatar: String::from(""),
            }),
            |_| Err(String::from("You are alreay a member of this group!")),
        )?;

        self.member_list.insert(principal, member.clone());

        Ok(member)
    }

    pub fn quit(&mut self, principal: Principal) -> Result<bool, String> {
        let mut member = self
            .member_list
            .get_mut(&principal)
            .ok_or(String::from("You are not yet a member of this group!"))?;

        member.member_status_code = MemberStatusCode::Quit(-1);

        Ok(true)
    }
}
