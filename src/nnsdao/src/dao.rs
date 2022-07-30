use crate::{canister::dip20, Data};
use async_trait::async_trait;
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_kit::ic::{self};
use nnsdao_sdk_basic::{
    ChangeProposalStateArg, DaoBasic, DaoCustomFn, Proposal, ProposalArg, ProposalState, Votes,
    VotesArg,
};
use num_bigint::ToBigUint;
use serde::Serialize;
use std::{collections::HashMap, option::Option::Some};

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
    // votes_list: Vec<UserVoteArgs>,
    info: DaoInfo,
    pub basic: DaoBasic<CustomDao>,
    pub handled_list: Vec<(u64, Result<String, String>)>,
    checking: bool, //
}

impl DaoService {
    pub fn is_member(&self, member: Principal) -> Result<bool, String> {
        self.member_list
            .get(&member)
            .ok_or_else(|| String::from("Users have not yet joined current DAO!"))?;

        Ok(true)
    }
    pub async fn propose(&mut self, arg: ProposalBody) -> Result<Proposal, String> {
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
    async fn validate_before_vote(&mut self, vote_arg: UserVoteArgs) -> Result<bool, String> {
        // owner can not vote for self;
        let proposal_info = if let Ok(proposal) = self.basic.get_proposal(vote_arg.id) {
            proposal
        } else {
            return Err("Failed to get proposal information".to_string());
        };
        match vote_arg.principal {
            Some(principal) => {
                if principal == proposal_info.proposer {
                    return Err("You can't vote for yourself!".to_string());
                }
            }
            None => (),
        }
        let caller = ic_cdk::caller();
        // check balance
        let dip_client =
            dip20::Service::new(Principal::from_text("vgqnj-miaaa-aaaal-qaapa-cai").unwrap());
        let dao_principal = Principal::from_text("67bzx-5iaaa-aaaam-aah5a-cai").unwrap();
        let balance = dip_client
            .balanceOf(vote_arg.principal.unwrap())
            .await
            .unwrap();
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
    pub async fn check_proposal(&mut self) {
        if self.checking {
            return;
        }
        self.checking = true;
        let now = ic_cdk::api::time();
        let dip_client =
            dip20::Service::new(Principal::from_text("vgqnj-miaaa-aaaal-qaapa-cai").unwrap());

        for (id, proposal) in self.basic.proposal_list.clone().into_iter() {
            // expired or not been handled
            let mut already_handed = false;
            for item in &self.handled_list {
                if item.0 == id {
                    already_handed = true;
                    break;
                }
            }
            if proposal.end_time <= now || already_handed {
                continue;
            }
            // let result = (
            //     id,
            //     Ok(format!("now is {} expired at{}", now, proposal.end_time)),
            // );
            // self.handled_list.push(result);
            if proposal.proposal_state == ProposalState::Open {
                // caculate weight
                let mut yes = 0;
                let mut yes_count = 0;
                let mut no = 0;
                let mut no_count = 0;
                for vote in &proposal.vote_data {
                    match vote.1 {
                        Votes::Yes(count) => {
                            yes += count;
                            yes_count += 1
                        }
                        Votes::No(count) => {
                            no += count;
                            no_count += 1;
                        }
                    }
                }
                if yes == 0 && no == 0 {
                    if let Err(err) = self.basic.change_proposal_state(ChangeProposalStateArg {
                        id,
                        state: ProposalState::Rejected,
                    }) {
                        let result = (id, Err(err));
                        self.handled_list.push(result);
                        continue;
                    }
                    let result = (id, Ok(format!("done yes:{} no:{}", yes, no)));
                    self.handled_list.push(result);
                    continue;
                }
                // reward yes
                if yes > no {
                    // return proposer ndp;
                    let proposal_amount = 1;
                    // Divide equally left ndp
                    let per_count = no / (no_count + 1);
                    if (dip_client
                        .transfer_token(
                            proposal.proposer,
                            candid::Nat((proposal_amount + per_count).to_biguint().unwrap()),
                        )
                        .await)
                        .is_err()
                    {
                        let result = (
                            id,
                            Err(format!(
                                "{} failed transfer {}",
                                proposal.proposer.clone().to_text(),
                                proposal_amount + per_count
                            )),
                        );
                        self.handled_list.push(result);
                        continue;
                    }

                    for vote in &proposal.vote_data {
                        match vote.1 {
                            Votes::Yes(count) => {
                                if (dip_client
                                    .transfer_token(
                                        vote.0,
                                        candid::Nat((per_count + count).to_biguint().unwrap()),
                                    )
                                    .await)
                                    .is_err()
                                {
                                    let result =
                                        (id, Err(format!("{} failed transfer {}", vote.0, count)));
                                    self.handled_list.push(result);
                                    continue;
                                }
                            }
                            Votes::No(_count) => (),
                        }
                    }
                } else {
                    // give back no
                    // let per_count = yes / yes_count;
                    for vote in &proposal.vote_data {
                        match vote.1 {
                            Votes::Yes(_count) => {}
                            Votes::No(count) => {
                                if (dip_client
                                    .transfer_token(
                                        vote.0,
                                        candid::Nat((&count).to_biguint().unwrap()),
                                    )
                                    .await)
                                    .is_err()
                                {
                                    let result =
                                        (id, Err(format!("{} failed transfer {}", vote.0, count)));
                                    self.handled_list.push(result);
                                    continue;
                                }
                            }
                        }
                    }
                }
                if let Err(err) = self.basic.change_proposal_state(ChangeProposalStateArg {
                    id,
                    state: if yes > no {
                        ProposalState::Accepted
                    } else {
                        ProposalState::Rejected
                    },
                }) {
                    let result = (id, Err(err));
                    self.handled_list.push(result);
                    continue;
                }
                let result = (id, Ok(format!("done yes:{} no:{}", yes, no)));
                self.handled_list.push(result)
            }
        }
        self.checking = false;
    }
    pub async fn vote(&mut self, mut arg: UserVoteArgs) -> Result<(), String> {
        let caller = ic_cdk::caller();
        arg.principal = Some(caller);
        let valid = self.validate_before_vote(arg.clone()).await?;
        if !valid {
            return Err(String::from("vote failed"));
        }
        self.basic
            .vote(VotesArg {
                id: arg.id,
                caller,
                vote: arg.vote,
            })
            .await?;
        // self.votes_list.push(arg);
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
    pub fn get_handled_proposal(&self) -> Vec<(u64, Result<String, String>)> {
        self.handled_list.clone()
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
