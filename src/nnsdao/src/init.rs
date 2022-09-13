use crate::{dao::JoinDaoParams, Data};
use candid::Principal;
use ic_cdk_macros::init;
use ic_kit::ic;

#[init]
fn init(owner: Principal) {
    ic_cdk::setup();
    let data = ic::get_mut::<Data>();
    data.owners.add_owner(owner);

    if let Ok(..) = data.dao.join(
        owner,
        JoinDaoParams {
            nickname: "owner".to_string(),
            ..Default::default()
        },
    ) {}
    // data.dao.update_dao_info(dao_info).unwrap();
}
