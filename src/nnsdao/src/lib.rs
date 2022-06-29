mod dao;
mod init;
mod logger;
mod owner;
mod tools;

use crate::logger::*;
use crate::owner::*;
use dao::UserVoteArgs;
use dao::{DaoService, MemberItems};
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_cdk_macros::*;
use ic_kit::ic;
use nnsdao_sdk_basic::Proposal;
use nnsdao_sdk_basic::ProposalArg;
use nnsdao_sdk_basic::VotesArg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::vec::Vec;

// #[derive(Default, Clone)]
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Data {
    #[serde(default)]
    pub owners: OwnerService,
    #[serde(default)]
    pub logger: LoggerService,
    #[serde(default)]
    pub dao: DaoService,

    #[serde(default)]
    pub run_heartbeat: bool,
    #[serde(default)]
    pub heartbeat_last_beat: u64,
    #[serde(default)]
    pub heartbeat_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DataV0 {
    #[serde(default)]
    pub owners: OwnerService,

    #[serde(default)]
    pub dao: DaoService,

    #[serde(default)]
    pub logger: LoggerService,
}

#[update]
#[candid::candid_method]
fn join(user_info: MemberItems) -> Result<MemberItems, String> {
    let data = ic::get_mut::<Data>();
    let caller = ic_cdk::caller();
    data.dao.join(caller, user_info)
}

#[query]
#[candid::candid_method]
fn member_list() -> Result<Vec<MemberItems>, String> {
    let data = ic::get::<Data>();
    data.dao.member_list()
}

#[update]
#[candid::candid_method]
fn quit() -> Result<bool, String> {
    let data = ic::get_mut::<Data>();
    let caller = ic_cdk::caller();
    data.dao.quit(caller)
}

#[query]
#[candid::candid_method(query)]
fn proposal_list() -> Result<HashMap<u64, Proposal>, String> {
    let data = ic::get::<Data>();
    Ok(data.dao.basic.proposal_list())
}

#[update]
#[candid::candid_method]
async fn proposal(arg: ProposalArg) -> Result<(), String> {
    let data = ic::get_mut::<Data>();
    data.dao.basic.proposal(arg).await
}

#[query]
#[candid::candid_method(query)]
fn get_proposal(id: u64) -> Result<Proposal, String> {
    let data = ic::get::<Data>();
    data.dao.basic.get_proposal(id)
}

#[update]
#[candid::candid_method(update)]
async fn votes(arg: UserVoteArgs) -> Result<(), String> {
    let caller = ic::caller();
    let data = ic::get_mut::<Data>();
    let vote_arg = VotesArg {
        caller,
        id: arg.id,
        vote: arg.vote,
    };
    data.dao.basic.vote(vote_arg).await
}

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();

    let writer = StableWriter::default();
    serde_cbor::to_writer(
        writer,
        &DataV0 {
            owners: data.owners.clone(),
            logger: data.logger.clone(),
            dao: data.dao.clone(),
        },
    )
    .expect("Failed to serialize data.");
}

#[post_upgrade]
fn post_upgrade() {
    let reader = StableReader::default();

    let data: DataV0 = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };

    ic::store(Data {
        owners: data.owners,
        logger: data.logger,
        dao: data.dao,

        run_heartbeat: false,
        heartbeat_last_beat: 0,
        heartbeat_interval_seconds: 5,
    });
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
