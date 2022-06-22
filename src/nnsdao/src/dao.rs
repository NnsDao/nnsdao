use crate::Data;
use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_kit::ic;
use nnsdao_sdk_basic::{DaoBasic, DaoCustomFn};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
struct CustomDao {}

#[async_trait]
impl DaoCustomFn for CustomDao {
    async fn is_member(&self, member: Principal) -> Result<bool, String> {
        let data = ic::get_mut::<Data>();
        data.dao.is_member(member)
    }

    async fn handle_proposal(&self) -> Result<(), String> {
        Ok(())
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
pub struct Social {
    telegram: String,
    medium: String,
    discord: String,
    twitter: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
pub struct MemberItems {
    nickname: String,
    status_code: MemberStatusCode,
    avatar: String,
    intro: String,
    social: Social,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default)]
pub struct DaoInfo {
    name: String,           // dao name
    poster: Option<String>, // optional dao poster
    avatar: String,         // dao avatar
    tags: Vec<String>,      // dao tags
    intro: String,          // dao intro
    social: Social,
}

#[derive(CandidType, Serialize, Deserialize, Default)]
pub struct DaoService {
    member_list: HashMap<Principal, MemberItems>,
    info: DaoInfo,
    basic: DaoBasic<CustomDao>,
}

impl DaoService {
    pub fn is_member(&self, member: Principal) -> Result<bool, String> {
        self.member_list
            .get(&member)
            .ok_or_else(|| String::from("Users have not yet joined current DAO!"))?;

        Ok(true)
    }
    pub fn dao_info(&mut self) -> Result<DaoInfo, String> {
        Ok(self.info.clone())
    }
    pub fn update_dao_info(&mut self, dao_info: DaoInfo) -> Result<DaoInfo, String> {
        self.info.name = dao_info.name;
        self.info.poster = dao_info.poster;
        self.info.avatar = dao_info.avatar;
        self.info.tags = dao_info.tags;
        self.info.intro = dao_info.intro;
        self.info.social = dao_info.social;
        self.dao_info()
    }
    pub fn member_list(&self) -> Result<Vec<MemberItems>, String> {
        Ok(self.member_list.values().cloned().collect())
    }

    pub fn join(
        &mut self,
        principal: Principal,
        user_info: MemberItems,
    ) -> Result<MemberItems, String> {
        let member = self.member_list.get(&principal).map_or(
            Ok(MemberItems {
                nickname: user_info.nickname,
                status_code: MemberStatusCode::Joined(1),
                avatar: user_info.avatar,
                intro: user_info.intro,
                social: user_info.social,
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
            .ok_or_else(|| String::from("You are not yet a member of this group!"))?;

        member.status_code = MemberStatusCode::Quit(-1);

        Ok(true)
    }
}
