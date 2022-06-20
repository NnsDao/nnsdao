use crate::DAO_SERVICE_STABLE;

pub fn is_owner() -> Result<(), String> {
    DAO_SERVICE_STABLE.with(|dao_service| dao_service.borrow().is_owner())
}
