use crate::Data;
use ic_cdk_macros::init;
use ic_kit::ic;

#[init]

fn init(owner: Principal) {
    ic_cdk::setup();
    let data = ic::get_mut::<Data>();
    data.owners.add_owner(owner);
    // data.dao.update_dao_info(dao_info).unwrap();
}
