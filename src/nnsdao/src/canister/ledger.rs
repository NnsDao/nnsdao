use crate::canister::ext_client::CanisterExtClient;
use crate::canister::standard_ext::{self as ext, TransferRequest, TransferResponse, User};
use ic_cdk::export::candid::Principal;

use ic_ledger_types::{
    AccountBalanceArgs, AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, TransferArgs,
    DEFAULT_FEE, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID,
};
use std::convert::TryFrom;

pub async fn icp_balance(
    user: Principal,
    user_subaccount: Option<Subaccount>,
) -> Result<u128, String> {
    let arg = AccountBalanceArgs {
        account: AccountIdentifier::new(&user, &user_subaccount.unwrap_or(DEFAULT_SUBACCOUNT)),
    };

    let tokens = ic_ledger_types::account_balance(MAINNET_LEDGER_CANISTER_ID, arg)
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e));

    match tokens {
        Ok(t) => Ok(t.e8s() as u128),
        Err(err) => Err(err),
    }

    // return Ok(100000000);
}

pub async fn icp_transfer(
    from_sub_account: Option<Subaccount>,
    to: Principal,
    to_sub_account: Option<Subaccount>,
    amount: u64,
    memo: Memo,
) -> Result<BlockIndex, String> {
    let arg = TransferArgs {
        memo,
        amount: Tokens::from_e8s(amount),
        fee: DEFAULT_FEE,
        from_subaccount: from_sub_account,
        to: AccountIdentifier::new(&to, &to_sub_account.unwrap_or(DEFAULT_SUBACCOUNT)),
        created_at_time: None,
    };

    ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, arg)
        .await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .map_err(|e| format!("ledger transfer error {:?}", e))
}

pub async fn ndp_balance(
    user: Principal,
    user_subaccount: Option<Subaccount>,
) -> Result<u128, String> {
    let ledger = CanisterExtClient::new(String::from("vgqnj-miaaa-aaaal-qaapa-cai"));

    let aid = AccountIdentifier::new(&user, &user_subaccount.unwrap_or(DEFAULT_SUBACCOUNT));

    let arg = ext::BalanceRequest {
        token: String::from(""),
        user: User::address(aid.to_string().to_lowercase()),
    };

    match ledger.balance(arg).await.unwrap().0 {
        ext::BalanceResponse::ok(balance) => Ok(balance),
        ext::BalanceResponse::err(err) => Err(format!("{:#?}", &err)),
    }
}

// Transfer funds on nns ledger
pub async fn ndp_transfer(
    from: Principal,
    from_subaccount: Option<Subaccount>,
    to: Principal,
    to_subaccount: Option<Subaccount>,
    amount: u128,
    memo: Vec<u8>,
) -> Result<BlockIndex, String> {
    let ledger = CanisterExtClient::new(String::from("vgqnj-miaaa-aaaal-qaapa-cai"));

    let from = AccountIdentifier::new(&from, &from_subaccount.unwrap_or(DEFAULT_SUBACCOUNT));
    let to = AccountIdentifier::new(&to, &to_subaccount.unwrap_or(DEFAULT_SUBACCOUNT));

    let arg = TransferRequest {
        from: User::address(from.to_string().to_lowercase()),
        subaccount: Some(from_subaccount.unwrap_or(DEFAULT_SUBACCOUNT).0.to_vec()),
        to: User::address(to.to_string().to_lowercase()),
        token: String::from("vgqnj-miaaa-aaaal-qaapa-cai"),
        amount,
        notify: false,
        memo,
    };

    match ledger.transfer(arg).await.unwrap().0 {
        TransferResponse::ok(block) => {
            return Ok(u64::try_from(block).unwrap());
        }
        TransferResponse::err(err) => {
            return Err(format!("ledger transfer error {:?}", err));
        }
    }
}
