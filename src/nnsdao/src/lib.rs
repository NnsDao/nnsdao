mod canister;
mod dao;
mod disburse;
mod init;
mod logger;
mod owner;
pub mod sdk;
mod tools;
pub mod types;

use crate::logger::*;
use crate::owner::*;
use crate::sdk::Comment;
use crate::sdk::Proposal;
use crate::tools::canister_status;
use crate::types::DaoData;

use candid::Principal;
use dao::DaoInfo;
use dao::JoinDaoParams;
use dao::ProposalBody;
use dao::ProposalContent;
use dao::UserVoteArgs;
use dao::{DaoService, MemberItems};
use disburse::DisburseService;
use ic_cdk::api::management_canister::main::{update_settings, UpdateSettingsArgument};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use ic_kit::interfaces::management::CanisterStatusResponse;

use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_cdk_macros::*;
use ic_kit::ic;

use ic_kit::RejectionCode;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::vec::Vec;
use tools::is_owner;

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

#[update]
#[candid::candid_method]
pub async fn comment_proposal(id: u64, comment: Comment) -> Result<Proposal, String> {
    let data = ic::get_mut::<Data>();
    data.dao.basic.comment(id, comment)
}

#[query]
#[candid::candid_method]
fn member_list() -> Result<Vec<MemberItems>, String> {
    let data = ic::get::<Data>();
    data.dao.member_list()
}

#[update]
#[candid::candid_method]
async fn dao_data() -> Result<DaoData, String> {
    let data = ic::get::<Data>();
    let info = data.dao.dao_info()?;
    let member_list = data.dao.member_list()?;
    let owners = data
        .owners
        .get_owners()
        .iter()
        .map(|el| el.to_text())
        .collect();
    let status = canister_status().await?;
    let res = DaoData {
        info,
        owners,
        status,
        member_list,
    };
    Ok(res)
}

#[query]
#[candid::candid_method]
fn dao_info() -> Result<dao::DaoInfo, String> {
    let data = ic::get::<Data>();
    data.dao.dao_info()
}

#[update]
#[candid::candid_method]
async fn dao_status() -> Result<CanisterStatusResponse, String> {
    canister_status().await
}

#[update(guard = "is_owner")]
#[candid::candid_method]
fn update_dao_info(dao_info: DaoInfo) -> Result<DaoInfo, String> {
    let data = ic::get_mut::<Data>();
    data.dao.update_dao_info(dao_info)
}

#[update]
#[candid::candid_method]
fn user_info() -> Result<MemberItems, String> {
    let data = ic::get_mut::<Data>();
    data.dao.user_info()
}

#[update(guard = "is_owner")]
#[candid::candid_method(update)]
async fn update_controller(principal_text: String) -> Result<(), (RejectionCode, String)> {
    let mut owners = ic::get::<Data>().owners.get_owners();
    owners.push(Principal::from_text(principal_text).expect("Could not decode the principal."));
    update_settings(UpdateSettingsArgument {
        canister_id: ic_cdk::id(),
        settings: CanisterSettings {
            controllers: Some(owners),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        },
    })
    .await?;
    Ok(())
}

#[update]
#[candid::candid_method]
fn quit() -> Result<MemberItems, String> {
    let data = ic::get_mut::<Data>();
    let caller = ic_cdk::caller();
    data.dao.quit(caller)
}

#[update(guard = "is_owner")]
#[candid::candid_method]
fn add_owner(principal: Principal) -> Vec<Principal> {
    let data = ic::get_mut::<Data>();
    let caller = ic_cdk::caller();
    data.owners.add_owner(principal)
}

#[query]
#[candid::candid_method(query)]
fn get_owners() -> Vec<Principal> {
    let data = ic::get::<Data>();
    data.owners.get_owners()
}
#[query]
#[candid::candid_method(query)]
fn get_proposal_list() -> Result<HashMap<u64, Proposal>, String> {
    let data = ic::get::<Data>();
    Ok(data.dao.basic.proposal_list())
}

// #[update]
// #[candid::candid_method]
// async fn get_allow() -> Result<
//     (
//         (String),
//         (candid::Nat,),
//         (canister::dip20::Result,),
//         (candid::Nat,),
//         candid::Nat,
//     ),
//     String,
// > {
//     let data = ic::get::<Data>();
//     data.dao.get_allow().await
// }

// #[query]
// #[candid::candid_method(query)]
// fn get_pay_address() -> Result<String, String> {
//     let data = ic::get_mut::<Data>();
//     let transaction_subaccount = data.disburse.get_transaction_subaccount();
//     let payment_address = AccountIdentifier::new(&ic_cdk::api::id(), &transaction_subaccount);
//     Ok(payment_address.to_string())
// }

#[update]
#[candid::candid_method]
async fn propose(arg: ProposalContent) -> Result<Proposal, String> {
    let data = ic::get_mut::<Data>();

    data.dao
        .propose(ProposalBody {
            proposer: ic_cdk::caller(),
            title: arg.title,
            content: arg.content,
            start_time: arg.start_time,
            end_time: arg.end_time,
            property: arg.property,
        })
        .await
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

#[query]
#[candid::candid_method(query)]
pub async fn get_handled_proposal() -> Vec<(u64, Result<String, String>)> {
    let data = ic::get_mut::<Data>();
    data.dao.check_proposal().await;
    data.dao.get_handled_proposal()
}
// // heartbeat: 1s
// #[heartbeat]
// async fn heartbeat() {
//     let data = ic::get_mut::<Data>();
//     // Limit heartbeats
//     let now = ic_cdk::api::time();
//     if now - data.heartbeat_last_beat < 5 * 1_000_000_000 {
//         return;
//     }
//     data.heartbeat_last_beat = now;
//     data.dao.check_proposal().await;
//     // log

//     // check proposal expire time

//     // ic_cdk::println!("check proposal expire time : {:?}",'');
// }

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
        run_heartbeat: true,
        heartbeat_last_beat: 0,
        heartbeat_interval_seconds: 2,
    });
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
