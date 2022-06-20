mod dao;
mod init;
mod logger;
mod owner;
mod tools;

use candid::Principal;
use dao::DaoService;
use ic_cdk_macros::update;
use std::cell::RefCell;
// use crate::logger::*;
// use ic_cdk::api::stable::{StableReader, StableWriter};
// // use ic_cdk_macros::*;
// use ic_kit::ic;
// use serde::{Deserialize, Serialize};
// use std::io::Read;
// use std::vec::Vec;

thread_local! {
    static DAO_SERVICE_STABLE:RefCell<DaoService> = RefCell::default();
}

// #[query]
// #[candid::candid_method(query)]
// fn is_member(member: Principal) -> Result<bool, String> {
//     DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow().is_member(member))
// }

#[update]
#[candid::candid_method]
fn join(principal: Principal) -> Result<bool, String> {
    DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow_mut().join(principal))
}

#[update]
#[candid::candid_method]
fn quit(principal: Principal) -> Result<bool, String> {
    DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow_mut().quit(principal))
}

// #[derive(Default)]
// pub struct Data {
//     pub owners: OwnerService,
//     pub logger: LoggerService,

//     pub run_heartbeat: bool,
//     pub heartbeat_last_beat: u64,
//     pub heartbeat_interval_seconds: u64,
// }

// #[derive(Serialize, Deserialize, Default)]
// pub struct DataV0 {
//     #[serde(default)]
//     pub owners: OwnerService,

//     #[serde(default)]
//     pub logger: LoggerService,
// }

// #[pre_upgrade]
// fn pre_upgrade() {
//     let data = ic::get::<Data>();

//     let writer = StableWriter::default();
//     serde_cbor::to_writer(
//         writer,
//         &DataV0 {
//             owners: data.owners.clone(),
//             logger: data.logger.clone(),
//         },
//     )
//     .expect("Failed to serialize data.");
// }

// #[post_upgrade]
// fn post_upgrade() {
//     let reader = StableReader::default();

//     let data: DataV0 = match serde_cbor::from_reader(reader) {
//         Ok(t) => t,
//         Err(err) => {
//             let limit = err.offset() - 1;
//             let reader = StableReader::default().take(limit);
//             serde_cbor::from_reader(reader).expect("Failed to deserialize.")
//         }
//     };

//     ic::store(Data {
//         owners: data.owners,
//         logger: data.logger,

//         run_heartbeat: false,
//         heartbeat_last_beat: 0,
//         heartbeat_interval_seconds: 5,
//     });
// }

// candid::export_service!();

// #[query(name = "__get_candid_interface_tmp_hack")]
// fn export_candid() -> String {
//     __export_service()
// }
