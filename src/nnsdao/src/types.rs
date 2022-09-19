use candid::CandidType;

#[derive(CandidType)]
pub struct ProposalLog {
    pub pending: Vec<u64>,
    pub finished: Vec<(u64, Result<String, String>)>,
}
