use super::standard_ext as ext;
use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::Principal;

pub struct CanisterExtClient {
    id: String,
}

impl CanisterExtClient {
    pub fn new(canister_id: String) -> Self {
        CanisterExtClient { id: canister_id }
    }

    pub async fn transfer(
        &self,
        arg: ext::TransferRequest,
    ) -> CallResult<(ext::TransferResponse,)> {
        ext::transfer(Principal::from_text(self.id.as_str()).unwrap(), arg).await
    }

    pub async fn balance(&self, arg: ext::BalanceRequest) -> CallResult<(ext::BalanceResponse,)> {
        ext::balance(Principal::from_text(self.id.as_str()).unwrap(), arg).await
    }

    pub async fn bearer(&self, arg: ext::TokenIdentifier) -> CallResult<(ext::BearerResponse,)> {
        ext::bearer(Principal::from_text(self.id.as_str()).unwrap(), arg).await
    }

    pub async fn tokens(&self, arg: ext::TokenIdentifier) -> CallResult<(ext::TokensResponse,)> {
        ext::tokens(Principal::from_text(self.id.as_str()).unwrap(), arg).await
    }
}
