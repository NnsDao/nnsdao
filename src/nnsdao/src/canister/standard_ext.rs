use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::{CandidType, Deserialize};

pub type TokenIdentifier = String;
pub type TokenIndex = u32; // Represents an individual token's index within a given canister.

pub type AccountIdentifier = String;
pub type Subaccount = Vec<u8>;

pub type Memo = Vec<u8>;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum User {
    principal(candid::Principal),
    address(AccountIdentifier),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TransferRequest {
    pub to: User,
    pub token: TokenIdentifier,
    pub notify: bool,
    pub from: User,
    pub memo: Memo,
    pub subaccount: Option<Subaccount>,
    pub amount: u128,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum TransferResponse_err {
    CannotNotify(AccountIdentifier),
    InsufficientBalance,
    InvalidToken(TokenIdentifier),
    Rejected,
    Unauthorized(AccountIdentifier),
    Other(String),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum TransferResponse {
    ok(u128),
    err(TransferResponse_err),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BalanceRequest {
    pub token: TokenIdentifier,
    pub user: User,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CommonError {
    InvalidToken(TokenIdentifier),
    Other(TokenIdentifier),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum BalanceResponse {
    ok(u128),
    err(CommonError),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum BearerResponse {
    ok(AccountIdentifier),
    err(CommonError),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum TokensResponse {
    ok(Vec<u32>),
    err(CommonError),
}

pub fn account_idf_equal(a: &AccountIdentifier, b: &AccountIdentifier) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

pub async fn transfer(
    canister_id: candid::Principal,
    arg: TransferRequest,
) -> CallResult<(TransferResponse,)> {
    ic_cdk::call(canister_id, "transfer", (arg,)).await
}

pub async fn balance(
    canister_id: candid::Principal,
    arg: BalanceRequest,
) -> CallResult<(BalanceResponse,)> {
    ic_cdk::call(canister_id, "balance", (arg,)).await
}

pub async fn bearer(
    canister_id: candid::Principal,
    arg: TokenIdentifier,
) -> CallResult<(BearerResponse,)> {
    ic_cdk::call(canister_id, "bearer", (arg,)).await
}

pub async fn tokens(
    canister_id: candid::Principal,
    arg: TokenIdentifier,
) -> CallResult<(TokensResponse,)> {
    ic_cdk::call(canister_id, "tokens", (arg,)).await
}
