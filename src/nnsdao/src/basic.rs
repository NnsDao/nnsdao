//! This SDK provides the basic functionality to build a DAO, which can be imported and used.
//! To use it, you need to implement DaoCustomFn Trait and customize the business logic.
//! Example of implementing DaoCustomFn Trait
//! ```
//! #[derive(Clone, Debug, Default, CandidType, Deserialize)]
//! struct CustomFn{}
//! #[async_trait]
//! impl DaoCustomFn for CustomFn {
//!  async fn is_member(&self, _member: Principal) -> Result<bool, String> {
//!   Ok(true)
//!  }
//!  async fn handle_proposal(&self) -> Result<(), String> {
//!  Ok(())
//!  }
//! }
//! let dao_basic = DaoBasic::new(CustomFn::default());
//! dao_basic.get_proposal(1);
//! ```

use std::collections::HashMap;

use ic_cdk::export::{candid::CandidType, Principal};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use ic_cdk::api;

/// Voting weight
pub type Equities = u64;

/// Votes with weights
#[derive(CandidType, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Votes{
    Yes(Equities),
    No(Equities),
}


/// You need to use the basic methods implemented by the party
#[async_trait]
pub trait DaoCustomFn {
    /// It is used to determine whether you are DAO member of Organization
    async fn is_member(&self, member: Principal) -> Result<bool, String>;

    /// Implement process completed proposals
    async fn handle_proposal(&self) -> Result<(), String>;
}


/// The state of a Proposal
#[derive(CandidType, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ProposalState {
    /// The proposal is open for voting
    Open,

    /// Enough "yes" votes have been cast to accept the proposal, and it will soon be executed
    Accepted,

    /// Enough "no" votes have been cast to reject the proposal, and it will not be executed
    Rejected,

    /// The proposal is currently being executed
    Executing,

    /// The proposal has been successfully executed
    Succeeded,

    /// A failure occurred while executing the proposal
    Failed(String),
}

/// Proposal unit structure
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Proposal {
    id: u64,
    proposer: Principal,
    title: String,
    content: String,
    proposal_state: ProposalState,
    vote_data: Vec<(Principal, Votes)>,
    end_time: u64,
    timestemp: u64,
}


/// Create parameters for the proposal
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProposalArg {
    pub proposer: Principal,
    pub title: String,
    pub content: String,
    pub end_time: u64,
}

/// Voting parameters
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct VotesArg {
    pub id: u64,
    pub caller: Principal,
    pub vote: Votes,
}

/// Change proposal status parameters
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ChangeproposalStateArg {
    pub id: u64,
    pub state: ProposalState
}

/// Basic DAO structure
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct DaoBasic<T:DaoCustomFn> {
    pub proposal_list: HashMap<u64, Proposal>,
    pub next_proposal_id: u64,
    pub custom_fn: T,
}

/// Implements the most basic DAO functionality
impl <T> DaoBasic<T> 
where
    T: DaoCustomFn,
{
    /// Instantiate the underlying DAO
    pub fn new(custom_fn: T) -> Self {
        DaoBasic {
            proposal_list: HashMap::default(),
            next_proposal_id: 1,
            custom_fn: custom_fn,
        }
    }

    /// Submit the proposal
    pub async fn proposal(&mut self, arg: ProposalArg) -> Result<(), String> {
        self.custom_fn.is_member(arg.proposer.clone()).await?;
        let proposal = Proposal {
            id: self.next_proposal_id,
            proposer: arg.proposer,
            title: arg.title,
            content: arg.content,
            proposal_state: ProposalState::Open,
            vote_data: Vec::new(),
            end_time: arg.end_time,
            timestemp: api::time(),
        };
        self.proposal_list.insert(self.next_proposal_id, proposal);
        self.next_proposal_id += 1;
        Ok(())
    }

    pub fn get_proposal(&self, id: u64) -> Result<Proposal, String>{
        self.proposal_list.get(&id).ok_or(String::from("no proposal")).cloned()
    }

    pub fn proposal_list(&self) -> HashMap<u64,Proposal> {
        self.proposal_list.clone()
    }

    pub async fn vote(&mut self, arg: VotesArg) -> Result<(), String> {
        self.custom_fn.is_member(arg.caller.clone()).await?;
        if let Some(proposal) = self.proposal_list.get_mut(&arg.id) {
            for data in proposal.vote_data.iter() {
                if data.0 == arg.caller {
                    return Err(String::from("Users have voted"));
                }
            }
            proposal.vote_data.push((arg.caller.clone(), arg.vote))
        } else {
            return Err(String::from("The proposal does not exist"));
        }
        Ok(())
    }

    pub async fn handle_proposal(&self) -> Result<(), String>{
        self.custom_fn.handle_proposal().await?;
        Ok(())
    }

    pub fn change_proposal_state(&mut self, arg: ChangeproposalStateArg) -> Result<(), String> {
        if let Some(proposal) = self.proposal_list.get_mut(&arg.id) {
            if proposal.end_time <= api::time() {
                return Err(String::from("Proposal time is not over"));
            }
            match proposal.proposal_state {
                ProposalState::Open => {
                    if arg.state != ProposalState::Accepted || arg.state != ProposalState::Rejected {
                        return Err(String::from("Failed to change status, the logic of the status parameter is incorrect"));
                    }
                    proposal.proposal_state = arg.state
                }
                ProposalState::Accepted | ProposalState::Rejected => {
                    if arg.state != ProposalState::Executing {
                        return Err(String::from("Failed to change status, the logic of the status parameter is incorrect"));
                    }
                    proposal.proposal_state = arg.state
                }
                ProposalState::Executing => {
                    match arg.state {
                        ProposalState::Succeeded => proposal.proposal_state = ProposalState::Succeeded,
                        ProposalState::Failed(reason) =>  proposal.proposal_state = ProposalState::Failed(reason),
                        _ => return Err(String::from("Failed to change status, the logic of the status parameter is incorrect")),
                    }
                }
                _ => return Err(String::from("Failed to change status, the logic of the status parameter is incorrect")),
            }
        } else {
            return Err(String::from("no proposal"))
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use ic_cdk::export::{candid::CandidType, Principal};
//     #[derive(Clone, Debug, Default, CandidType, Deserialize)]
//     struct CustomFn{}

//     #[async_trait]
//     impl DaoCustomFn for CustomFn {
//         async fn is_member(&self, _member: Principal) -> Result<bool, String> {
//             Ok(true)
//         }
//         async fn handle_proposal(&self) -> Result<(), String> {
//             Ok(())
//         }
//     }
//     #[actix_rt::test]
//     async fn test_get_proposal_err() {
//         let dao_basic = DaoBasic::new(CustomFn::default());
//         assert_eq!(dao_basic.get_proposal(1).is_err(), true);
//     }

//     #[actix_rt::test]
//     async fn test_get_proposal_ok() {
//         let mut dao_basic = DaoBasic::new(CustomFn::default());
//         let new_proposal = ProposalArg {
//             proposer: Principal::from_text(String::from("")).unwrap(),
//             title: "aaa".to_owned(),
//             content: "aaa".to_owned(),
//             end_time: 11111,
//         };
//         _ = dao_basic.proposal(new_proposal).await;
//         println!("{:?}", dao_basic.get_proposal(1));
//         assert_eq!(dao_basic.get_proposal(1).is_ok(), true);
//     }
// }



