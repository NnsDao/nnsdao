use crate::{
    canister::{self, dip20, ledger},
    Data,
};
use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_kit::ic::{self};
use ic_ledger_types::Subaccount;
use nnsdao_sdk_basic::{DaoBasic, DaoCustomFn, Proposal, ProposalArg, Votes, VotesArg};
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct CustomDao {}
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct UserVoteArgs {
    pub principal: Option<Principal>,
    pub id: u64,
    pub vote: Votes,
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
    // proposer_list: Vec<ProposerListItem>,
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
        // check balance
        // let caller = ic_cdk::caller();
        let dip_client =
            dip20::Service::new(Principal::from_text("vgqnj-miaaa-aaaal-qaapa-cai").unwrap());
        let balance = dip_client.balanceOf(arg.proposer).await.unwrap();
        let dao_principal = Principal::from_text("67bzx-5iaaa-aaaam-aah5a-cai").unwrap();

        // 1 ndp
        let amount: i64 = 1;
        let amount = candid::Nat((amount).to_biguint().unwrap());
        if balance.0 < amount {
            return Err(String::from("Insufficient balance!"));
        }
        // // approve
        // let approved = dip_client.approve(dao_principal, amount.clone()).await;
        // if let Err(_str) = approved {
        //     return Err("Approve failed".to_string());
        // }
        // ic_cdk::println!("approved {:#?}", approved);

        let allow = dip_client
            .allowance(arg.proposer, dao_principal)
            .await
            .unwrap();

        if allow.0 < amount {
            return Err("Approved insufficient NDP count".to_string());
        }
        // transfer
        let transfer = dip_client
            .transferFrom(arg.proposer, dao_principal, amount.clone())
            .await;

        // ic_cdk::println!("transfer {:#?}", transfer);
        if let Err(_str) = transfer {
            return Err("Transfer failed!".to_string());
        }

        let proposal_info = self
            .basic
            .proposal(ProposalArg {
                proposer: arg.proposer,
                title: arg.title,
                content: arg.content,
                end_time: arg.end_time,
            })
            .await?;
        // self.proposer_list.push(ProposerListItem {
        //     proposer: arg.proposer,
        //     id: proposal_info.id,
        // });
        Ok(proposal_info)
    }
    async fn validate_before_vote(&mut self, mut vote_arg: UserVoteArgs) -> Result<bool, String> {
        // check balance
        let caller = ic_cdk::caller();
        vote_arg.principal = Some(caller);
        let dip_client = dip20::Service::new(caller);
        let balance = dip_client.balanceOf(caller).await.unwrap();
        let dao_principal = Principal::from_text("67bzx-5iaaa-aaaam-aah5a-cai").unwrap();
        // 1 ndp
        let amount: i64 = 1_0000_0000;
        let amount = integer_to_nat(amount);

        let has_enough_balance = match vote_arg.vote {
            Votes::Yes(count) | Votes::No(count) => {
                balance.0 >= candid::Nat((count).to_biguint().unwrap())
            }
        };
        if balance.0 < amount || !has_enough_balance {
            return Err(String::from("Insufficient balance"));
        }
        // caculate weight
        let amount = match vote_arg.vote {
            Votes::Yes(num) => integer_to_nat(num as i64),
            Votes::No(num) => integer_to_nat(num as i64),
        };

        let allow = dip_client.allowance(caller, dao_principal).await.unwrap();

        if allow.0 < amount {
            return Err("Approved insufficient NDP count".to_string());
        }
        // transfer
        let transfer = dip_client
            .transferFrom(caller, dao_principal, amount.clone())
            .await;

        // ic_cdk::println!("transfer {:#?}", transfer);
        if let Err(_str) = transfer {
            return Err("Transfer failed!".to_string());
        }

        Ok(true)
    }
    pub fn proposal_list(
        &self,
    ) -> std::collections::hash_map::IntoIter<u64, nnsdao_sdk_basic::Proposal> {
        self.basic.proposal_list().into_iter()
    }
    pub async fn check_proposal(&mut self) -> Result<(), String> {
        let now = ic_cdk::api::time();
        self.basic.proposal_list.iter().for_each(|(_id, proposal)| {
            if now >= proposal.end_time {
                // caculate,then change proposal state
                //
                //
                // if 1 > 0 {
                //     //
                // } else {
                //     //
                // }
                // Ok(())
            }
        });
        Ok(())
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

fn integer_to_nat(amount: i64) -> candid::Nat {
    candid::Nat((amount).to_biguint().unwrap())
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProposalContent {
    pub title: String,
    pub content: String,
    pub end_time: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProposalBody {
    pub proposer: Principal,
    pub title: String,
    pub content: String,
    pub end_time: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ProposerListItem {
    proposer: Principal,
    id: u64,
}
