use candid::{Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::{self, CandidType, Deserialize};

#[derive(CandidType, Deserialize)]
enum TxError {
    InsufficientAllowance,
    InsufficientBalance,
    ErrorOperationStyle,
    Unauthorized,
    LedgerTrap,
    ErrorTo,
    Other(String),
    BlockUsed,
    AmountTooSmall,
}

#[derive(CandidType, Deserialize)]
pub enum Result {
    Ok(candid::Nat),
    Err(TxError),
}

#[derive(CandidType, Deserialize)]
struct Metadata {
    fee: candid::Nat,
    decimals: u8,
    owner: candid::Principal,
    logo: String,
    name: String,
    totalSupply: candid::Nat,
    symbol: String,
}

#[derive(CandidType, Deserialize)]
struct TokenInfo {
    holderNumber: u64,
    deployTime: u64,
    metadata: Metadata,
    historySize: u64,
    cycles: u64,
    feeTo: candid::Principal,
}

pub struct Service(candid::Principal);

impl Service {
    pub fn new(principal: Principal) -> Self {
        Service(principal)
    }
    pub async fn allowance(
        &self,
        arg0: candid::Principal,
        arg1: candid::Principal,
    ) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "allowance", (arg0, arg1)).await
    }
    pub async fn approve(
        &self,
        arg0: candid::Principal,
        arg1: candid::Nat,
    ) -> CallResult<(Result,)> {
        ic_cdk::call(self.0, "approve", (arg0, arg1)).await
    }
    pub async fn balanceOf(&self, arg0: candid::Principal) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "balanceOf", (arg0,)).await
    }
    pub async fn burn(&self, arg0: candid::Nat) -> CallResult<(Result,)> {
        ic_cdk::call(self.0, "burn", (arg0,)).await
    }
    pub async fn decimals(&self) -> CallResult<(u8,)> {
        ic_cdk::call(self.0, "decimals", ()).await
    }
    pub async fn getAllowanceSize(&self) -> CallResult<(u64,)> {
        ic_cdk::call(self.0, "getAllowanceSize", ()).await
    }
    pub async fn getHolders(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> CallResult<(Vec<(candid::Principal, candid::Nat)>,)> {
        ic_cdk::call(self.0, "getHolders", (arg0, arg1)).await
    }
    pub async fn getMetadata(&self) -> CallResult<(Metadata,)> {
        ic_cdk::call(self.0, "getMetadata", ()).await
    }
    pub async fn getTokenInfo(&self) -> CallResult<(TokenInfo,)> {
        ic_cdk::call(self.0, "getTokenInfo", ()).await
    }
    pub async fn getUserApprovals(
        &self,
        arg0: candid::Principal,
    ) -> CallResult<(Vec<(candid::Principal, candid::Nat)>,)> {
        ic_cdk::call(self.0, "getUserApprovals", (arg0,)).await
    }
    pub async fn historySize(&self) -> CallResult<(u64,)> {
        ic_cdk::call(self.0, "historySize", ()).await
    }
    pub async fn logo(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "logo", ()).await
    }
    pub async fn mint(&self, arg0: candid::Principal, arg1: candid::Nat) -> CallResult<(Result,)> {
        ic_cdk::call(self.0, "mint", (arg0, arg1)).await
    }
    pub async fn name(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "name", ()).await
    }
    pub async fn owner(&self) -> CallResult<(candid::Principal,)> {
        ic_cdk::call(self.0, "owner", ()).await
    }
    pub async fn setFee(&self, arg0: candid::Nat) -> CallResult<()> {
        ic_cdk::call(self.0, "setFee", (arg0,)).await
    }
    pub async fn setFeeTo(&self, arg0: candid::Principal) -> CallResult<()> {
        ic_cdk::call(self.0, "setFeeTo", (arg0,)).await
    }
    pub async fn setLogo(&self, arg0: String) -> CallResult<()> {
        ic_cdk::call(self.0, "setLogo", (arg0,)).await
    }
    pub async fn setName(&self, arg0: String) -> CallResult<()> {
        ic_cdk::call(self.0, "setName", (arg0,)).await
    }
    pub async fn setOwner(&self, arg0: candid::Principal) -> CallResult<()> {
        ic_cdk::call(self.0, "setOwner", (arg0,)).await
    }
    pub async fn symbol(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "symbol", ()).await
    }
    pub async fn totalSupply(&self) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "totalSupply", ()).await
    }
    pub async fn transfer(
        &self,
        arg0: candid::Principal,
        arg1: candid::Nat,
    ) -> CallResult<(Result,)> {
        ic_cdk::call(self.0, "transfer", (arg0, arg1)).await
    }
    pub async fn transferFrom(
        &self,
        arg0: candid::Principal,
        arg1: candid::Principal,
        arg2: candid::Nat,
    ) -> CallResult<(Result,)> {
        ic_cdk::call(self.0, "transferFrom", (arg0, arg1, arg2)).await
    }
    pub async fn transfer_token(&self, user: Principal, amount: Nat) -> CallResult<(Result,)> {
        ic_cdk::call(
            Principal::from_text("vgqnj-miaaa-aaaal-qaapa-cai").unwrap(),
            "transfer",
            (user, amount),
        )
        .await
    }
}
