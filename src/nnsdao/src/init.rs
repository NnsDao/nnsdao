use crate::DAO_SERVICE_STABLE;
use ic_kit::macros::init;

#[init]
fn init() {
    ic_cdk::setup();
    DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow_mut().set_owner(ic_cdk::caller()))
}
