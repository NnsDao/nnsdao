use crate::canister::ledger;
use crate::canister::standard_ext::TokenIdentifier;
use crate::tools;
use candid::{CandidType, Principal};
use ic_ledger_types::{
    AccountIdentifier as LedgerAccountIdentifier, Memo, Subaccount, DEFAULT_SUBACCOUNT,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{convert::TryFrom, vec};

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub enum Amount {
    NDP(u64),
    ICP(u64),
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            Self::NDP(a) => format!("{}:{}", "NDP", a),
            Self::ICP(a) => format!("{}:{}", "ICP", a),
        };

        write!(f, "{}", s)
    }
}

// Disbursement
#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct Disbursement {
    pub canister: String,
    pub token_idf: TokenIdentifier,
    pub from_subaccount: Option<Subaccount>,
    pub to: Principal,
    pub to_subaccount: Option<Subaccount>,
    pub amount: Amount,
    pub try_num: u8,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Default, Debug)]
pub struct DisburseService {
    #[serde(default)]
    pub subaccount_num: u128,

    #[serde(default)]
    pub disbursements_queue: Vec<Disbursement>,

    #[serde(default)]
    pub failed_disbursements: Vec<Disbursement>,

    #[serde(default)]
    #[serde(skip_serializing)]
    pub disbursements_process_lock: bool,
}

impl DisburseService {
    pub fn get_new_subaccount_num(&mut self) -> u128 {
        if self.subaccount_num == u128::MAX {
            self.subaccount_num = 1;
            return 1;
        }
        self.subaccount_num += 1;
        self.subaccount_num
    }

    pub fn get_transaction_subaccount(&mut self) -> Subaccount {
        let num = self.get_new_subaccount_num();

        let mut default_subaccount = DEFAULT_SUBACCOUNT;
        let num_to_vec = num.to_le_bytes();
        for (index, item) in num_to_vec.iter().enumerate() {
            default_subaccount.0[32 - index - 1] = item.clone()
        }
        default_subaccount
    }

    pub fn add_disbursement(&mut self, disbursement: Disbursement) -> () {
        self.disbursements_queue.push(disbursement);
    }

    pub async fn handle_faild_disbursements(
        &mut self,
    ) -> (Option<Disbursement>, Result<String, String>) {
        if self.failed_disbursements.is_empty() {
            return (None, Ok(String::from("")));
        }

        let disbursement = self.failed_disbursements.pop().unwrap();
        let r = self.handle_disbursement(disbursement.clone()).await;

        (Some(disbursement), r)
    }

    pub async fn handle_pendding_disbursements(
        &mut self,
    ) -> (Option<Disbursement>, Result<String, String>) {
        if self.disbursements_queue.is_empty() {
            return (None, Ok(String::from("")));
        }

        if self.disbursements_process_lock == true {
            return (None, Ok(String::from("")));
        };

        self.disbursements_process_lock = true;

        let disbursement = self.disbursements_queue.pop().unwrap();

        let r = self.handle_disbursement(disbursement.clone()).await;

        if r.is_err() {
            self.failed_disbursements.push(disbursement.clone());
        }

        self.disbursements_process_lock = false;

        (Some(disbursement), r)
    }

    pub async fn handle_disbursement(
        &mut self,
        disbursement: Disbursement,
    ) -> Result<String, String> {
        let result = match disbursement.amount {
            Amount::ICP(amount) => {
                let (_, idx) = tools::decode_token(disbursement.token_idf.clone())
                    .unwrap_or((Principal::anonymous(), 0));
                ledger::icp_transfer(
                    disbursement.from_subaccount,
                    disbursement.to,
                    disbursement.to_subaccount,
                    amount,
                    Memo(u64::try_from(idx).unwrap()),
                )
                .await
            }
            Amount::NDP(amount) => {
                ledger::ndp_transfer(
                    ic_cdk::api::id(),
                    disbursement.from_subaccount,
                    disbursement.to,
                    disbursement.to_subaccount,
                    amount.into(),
                    disbursement.token_idf.as_bytes().to_vec(),
                )
                .await
            }
        };

        match result {
            Ok(_) => {
                tools::log_message(
                    disbursement.canister.clone(),
                    ic_cdk::api::id(),
                    String::from("disburse_ok"),
                    vec![
                        (String::from("disbursement"), format!("{:?}", disbursement)),
                        (
                            "from".to_string(),
                            LedgerAccountIdentifier::new(
                                &ic_cdk::api::id(),
                                &disbursement.from_subaccount.unwrap_or(DEFAULT_SUBACCOUNT),
                            )
                            .to_string(),
                        ),
                        (
                            "to".to_string(),
                            LedgerAccountIdentifier::new(
                                &disbursement.to,
                                &disbursement.to_subaccount.unwrap_or(DEFAULT_SUBACCOUNT),
                            )
                            .to_string(),
                        ),
                    ],
                );

                Ok(String::from("ok"))
            }
            Err(err) => {
                tools::log_message(
                    disbursement.canister.clone(),
                    ic_cdk::api::id(),
                    String::from("disburse_err"),
                    vec![
                        (String::from("disbursement"), format!("{:?}", disbursement)),
                        (
                            "from".to_string(),
                            LedgerAccountIdentifier::new(
                                &ic_cdk::api::id(),
                                &disbursement.from_subaccount.unwrap_or(DEFAULT_SUBACCOUNT),
                            )
                            .to_string(),
                        ),
                        (
                            "to".to_string(),
                            LedgerAccountIdentifier::new(
                                &disbursement.to,
                                &disbursement.to_subaccount.unwrap_or(DEFAULT_SUBACCOUNT),
                            )
                            .to_string(),
                        ),
                        ("err".to_string(), format!("{:#?}", err)),
                    ],
                );

                Err(err)
            }
        }
    }
}
