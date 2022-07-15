use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_kit::ic::{self};
use ic_ledger_types::Subaccount;
use nnsdao_sdk_basic::{DaoBasic, DaoCustomFn, Proposal, ProposalArg, Votes, VotesArg};
use serde::Serialize;
use std::collections::HashMap;

use crate::{canister::ledger, Data};

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct CustomDao {}
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct UserVoteArgs {
    pub principal: Option<Principal>,
    pub id: u64,
    pub vote: Votes,
    pub sub_account: Subaccount,
}
#[async_trait]
impl DaoCustomFn for CustomDao {
    async fn is_member(&self, member: Principal) -> Result<bool, String> {
        let data = ic::get_mut::<Data>();
        data.dao.is_member(member)
    }

    async fn handle_proposal(&self) -> Result<(), String> {
        // heartbeat
        Ok(())
    }
}

// / error ic post_upgrade not support
// #[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
// pub enum MemberStatusCode {
//     Quit(i8),    // -1
//     Default(i8), // 0
//     Joined(i8),  // 1
// }

// impl Default for MemberStatusCode {
//     fn default() -> Self {
//         MemberStatusCode::Default(0)
//     }
// }

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct Social {
    key: String,
    link: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct MemberItems {
    nickname: String,
    status_code: i8, // -1 quit | 0 default | 1 joined |
    avatar: String,
    intro: String,
    social: Vec<Social>,
}
#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct JoinDaoParams {
    nickname: String,
    avatar: String,
    intro: String,
    social: Vec<Social>,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct DaoInfo {
    name: String,      // dao name
    poster: String,    //  dao poster
    avatar: String,    // dao avatar
    tags: Vec<String>, // dao tags
    intro: String,     // dao intro
    social: Social,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct VoteArg {
    pub id: u64,
    pub ndp_count: u64,
}

#[derive(CandidType, Serialize, Deserialize, Default, Clone, Debug)]
pub struct DaoService {
    member_list: HashMap<Principal, MemberItems>,
    proposer_list: Vec<ProposerListItem>,
    votes_list: Vec<UserVoteArgs>,
    info: DaoInfo,
    pub basic: DaoBasic<CustomDao>,
}

impl DaoService {
    pub fn is_member(&self, member: Principal) -> Result<bool, String> {
        self.member_list
            .get(&member)
            .ok_or_else(|| String::from("Users have not yet joined current DAO!"))?;

        Ok(true)
    }
    pub async fn initiate_proposal(&mut self, arg: ProposalBody) -> Result<Proposal, String> {
        // validate balance of subAccount
        let balance = ledger::ndp_balance(arg.proposer, Some(arg.sub_account)).await;

        // 1000 ndp
        let amount: u128 = 1000_0000_0000;
        if balance < Ok(amount as u128) {
            Err(String::from("Insufficient funds sent."))
        } else {
            let proposal_info = self
                .basic
                .proposal(ProposalArg {
                    proposer: arg.proposer,
                    title: arg.title,
                    content: arg.content,
                    end_time: arg.end_time,
                })
                .await?;
            self.proposer_list.push(ProposerListItem {
                proposer: arg.proposer,
                sub_account: arg.sub_account,
                id: proposal_info.id,
            });
            Ok(proposal_info)
        }
    }
    async fn validate_before_vote(&mut self, vote_arg: UserVoteArgs) -> Result<bool, String> {
        let balance = ledger::ndp_balance(ic_cdk::caller(), Some(vote_arg.sub_account))
            .await
            .unwrap();

        let equal = match vote_arg.vote {
            Votes::Yes(count) | Votes::No(count) => balance == count as u128,
        };
        Ok(equal)
    }
    pub fn proposal_list(
        &self,
    ) -> std::collections::hash_map::IntoIter<u64, nnsdao_sdk_basic::Proposal> {
        self.basic.proposal_list().into_iter()
    }
    pub async fn vote(&mut self, mut arg: UserVoteArgs) -> Result<(), String> {
        let valid = self.validate_before_vote(arg.clone()).await?;
        if !valid {
            return Err(String::from("Validate fail,vote failed"));
        }
        arg.principal = Some(ic_cdk::caller());
        self.basic
            .vote(VotesArg {
                id: arg.id,
                caller: ic_cdk::caller(),
                vote: arg.vote.clone(),
            })
            .await?;
        self.votes_list.push(arg);
        Ok(())
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
        user_info: JoinDaoParams,
    ) -> Result<MemberItems, String> {
        let member = self.member_list.get(&principal).map_or(
            Ok(MemberItems {
                nickname: user_info.nickname,
                status_code: 1,
                avatar: user_info.avatar,
                intro: user_info.intro,
                social: user_info.social,
            }),
            |_| Err(String::from("You are alreay a member of this group!")),
        )?;

        self.member_list.insert(principal, member.clone());

        Ok(member)
    }
    pub fn user_info(&self, principal: Principal) -> Result<MemberItems, String> {
        self.member_list
            .get(&principal)
            .cloned()
            .ok_or_else(|| "You are not yet a member of this group!".to_string())
    }
    pub fn quit(&mut self, principal: Principal) -> Result<bool, String> {
        let mut member = self
            .member_list
            .get_mut(&principal)
            .ok_or_else(|| String::from("You are not yet a member of this group!"))?;

        member.status_code = -1;

        Ok(true)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProposalContent {
    pub title: String,
    pub content: String,
    pub end_time: u64,
    pub sub_account: Subaccount,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProposalBody {
    pub proposer: Principal,
    pub title: String,
    pub content: String,
    pub end_time: u64,
    pub sub_account: Subaccount,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ProposerListItem {
    proposer: Principal,
    sub_account: Subaccount,
    id: u64,
}
