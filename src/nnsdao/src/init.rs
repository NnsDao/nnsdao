use crate::{dao::DaoInfo, Data};
use ic_cdk_macros::init;
use ic_kit::ic;

#[init]
fn init(dao_info: DaoInfo) {
    ic_cdk::setup();
    let data = ic::get_mut::<Data>();
    data.owners.add_owner(ic_cdk::caller());
    data.dao.update_dao_info(dao_info).unwrap();
}
