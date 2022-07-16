pub mod canister;
mod dao;
mod disburse;
mod init;
mod logger;
mod owner;
mod tools;

use crate::logger::*;
use crate::owner::*;
use dao::JoinDaoParams;
use dao::ProposalBody;
use dao::ProposalContent;
use dao::UserVoteArgs;
use dao::{DaoService, MemberItems};
use disburse::DisburseService;
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_cdk_macros::*;
use ic_kit::ic;
use ic_ledger_types::AccountIdentifier;
use nnsdao_sdk_basic::Proposal;
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
    #[serde(default)]
    pub disburse: DisburseService,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DataV0 {
    #[serde(default)]
    pub owners: OwnerService,

    #[serde(default)]
    pub dao: DaoService,

    #[serde(default)]
    pub logger: LoggerService,

    #[serde(default)]
    pub disburse: DisburseService,
}

#[update]
#[candid::candid_method]
fn join(user_info: JoinDaoParams) -> Result<MemberItems, String> {
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

#[query]
#[candid::candid_method]
fn user_info() -> Result<MemberItems, String> {
    let data = ic::get::<Data>();
    data.dao.user_info(ic_cdk::caller())
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
fn get_proposal_list() -> Result<HashMap<u64, Proposal>, String> {
    let data = ic::get::<Data>();
    Ok(data.dao.basic.proposal_list())
}

#[query]
#[candid::candid_method(query)]
fn get_pay_address() -> Result<String, String> {
    let data = ic::get_mut::<Data>();
    let transaction_subaccount = data.disburse.get_transaction_subaccount();
    let payment_address = AccountIdentifier::new(&ic_cdk::api::id(), &transaction_subaccount);
    Ok(payment_address.to_string())
}

#[update]
#[candid::candid_method]
async fn initiate_proposal(arg: ProposalContent) -> Result<Proposal, String> {
    let data = ic::get_mut::<Data>();
    let proposal = data
        .dao
        .initiate_proposal(ProposalBody {
            proposer: ic_cdk::caller(),
            title: arg.title,
            content: arg.content,
            end_time: arg.end_time,
            sub_account: arg.sub_account,
        })
        .await?;
    Ok(proposal)
}

#[query]
#[candid::candid_method(query)]
fn get_proposal(id: u64) -> Result<Proposal, String> {
    let data = ic::get::<Data>();
    data.dao.basic.get_proposal(id)
}

#[update]
#[candid::candid_method(update)]
async fn vote(arg: UserVoteArgs) -> Result<(), String> {
    let data = ic::get_mut::<Data>();
    data.dao.vote(arg).await
}

// heartbeat: 5s
// try settle: 15s
// handle disbursements: 5s
#[heartbeat]
async fn heartbeat() {
    let data = ic::get_mut::<Data>();
    if !data.run_heartbeat {
        return;
    }

    // Limit heartbeats
    let now = ic_cdk::api::time();
    if now - data.heartbeat_last_beat < data.heartbeat_interval_seconds * 1_000_000_000 {
        return;
    }
    data.heartbeat_last_beat = now;
    data.dao.check_proposal();
    // check proposal expire time

    // ic_cdk::println!("check proposal expire time : {:?}",'');
}

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    let writer = StableWriter::default();
    serde_cbor::to_writer(
        writer,
        &DataV0 {
            disburse: data.disburse.clone(),
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
        disburse: data.disburse,
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
