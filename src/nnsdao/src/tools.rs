use crate::Data;
use ic_cdk::export::Principal;
use ic_kit::ic;

pub fn is_owner() -> Result<(), String> {
    let data = ic::get::<Data>();
    let caller = ic_cdk::caller();

    data.owners.is_owner(caller)
}

pub fn log_message(canister: String, caller: Principal, method: String, kv: Vec<(String, String)>) {
    let data = ic::get_mut::<Data>();
    data.logger.log_format_message(canister, caller, method, kv)
}

pub fn encode_token(canister_id: Principal, token: u32) -> String {
    let prefix: Vec<u8> = vec![10, 116, 105, 100];
    let mut token_u8 = vec![];
    let mut token_byte = vec![];

    token_byte.push((token >> 24) as u8);
    token_byte.push((token >> 16) as u8);
    token_byte.push((token >> 8) as u8);
    token_byte.push(token as u8);

    token_u8.extend_from_slice(&prefix);
    token_u8.extend_from_slice(canister_id.as_ref());
    token_u8.extend_from_slice(&token_byte);

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&token_u8);
    let checksum = hasher.finalize();

    let mut bytes = vec![];
    bytes.extend_from_slice(&checksum.to_be_bytes());
    bytes.extend_from_slice(&token_u8);
    let token_raw = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes)
        .as_str()
        .to_ascii_lowercase();
    let mut token_str = String::new();
    for (index, item) in token_raw.chars().enumerate() {
        if index % 5 == 0 && index != 0 {
            token_str.push('-');
        }
        token_str.push(item);
    }
    token_str
}

pub fn decode_token(mut token: String) -> Result<(Principal, u32), String> {
    token.retain(|c| c != '-');

    let prefix: Vec<u8> = vec![10, 116, 105, 100];
    match base32::decode(base32::Alphabet::RFC4648 { padding: false }, &token) {
        Some(mut bytes) => {
            if bytes.len() < 4 {
                return Err("too small string".to_owned());
            }
            let bytes = bytes.split_off(4);
            let (left, right) = bytes.split_at(4);
            if left != prefix {
                return Err("Token format error".to_owned());
            }
            let canister = Principal::from_slice(&right[..right.len() - 4]);
            let index_vec = right[right.len() - 4..].to_vec();
            let index = index_vec[3] as u32
                | (index_vec[2] as u32) << 8
                | (index_vec[1] as u32) << 16
                | (index_vec[0] as u32) << 24;
            Ok((canister, index))
        }
        None => Err("Token format error".to_owned()),
    }
}
