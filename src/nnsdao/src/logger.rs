use ic_cdk::export::Principal;
use ic_cdk::{api, export::candid::CandidType};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::TryInto;

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct LogMessageData {
    pub timeNanos: u64,
    pub message: String,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct LoggerService {
    pub queue: Vec<LogMessageData>,
    pub max_count: usize,
    pub next: usize,
    pub full: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetLogMessagesParameters {
    pub count: u32,
    pub filter: Option<GetLogMessagesFilter>,
    pub fromTimeNanos: Option<u64>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType, Deserialize)]
pub struct GetLogMessagesFilter {
    pub messageContains: Option<String>,
    pub messageRegex: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, CandidType)]
pub struct CanisterLogMessages {
    pub data: Vec<LogMessageData>,
    pub lastAnalyzedMessageTimeNanos: Option<u64>,
}

impl Default for LoggerService {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            max_count: 20000,
            next: 0,
            full: false,
        }
    }
}

impl LoggerService {
    pub fn store_log_message(&mut self, log_message: LogMessageData) {
        if self.full {
            self.queue[self.next] = log_message;
        } else {
            self.queue.push(log_message);
        }

        self.next += 1;

        if self.next == self.max_count {
            self.full = true;
            self.next = 0;
        }
    }

    pub fn log_format_message(
        &mut self,
        canister: String,
        caller: Principal,
        method: String,
        kv: Vec<(String, String)>,
    ) {
        let mut message = String::from("");
        for (k, v) in kv {
            message.push_str(&format!("{}:{},", k, v));
        }

        self.store_log_message(LogMessageData {
            timeNanos: api::time(),
            message: format!(
                "{}||{}||{}||{}",
                canister,
                caller.to_text(),
                method,
                message
            ),
        });
    }

    pub fn get_log_messages(&self, param: GetLogMessagesParameters) -> CanisterLogMessages {
        let mut data: Vec<LogMessageData> = self
            .queue
            .clone()
            .into_iter()
            .filter(|item| {
                let mut r = true;

                if let Some(from_time) = param.fromTimeNanos {
                    r = r && (item.timeNanos >= from_time);
                }

                if let Some(f) = &param.filter {
                    if let Some(str) = f.messageContains.clone() {
                        r = r && item.message.contains(&str);
                    }
                }

                r
            })
            .take(param.count.try_into().unwrap())
            .collect();

        // revc
        data.sort_by(|a, b| {
            if a.timeNanos < b.timeNanos {
                Ordering::Greater
            } else if a.timeNanos == b.timeNanos {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        });

        CanisterLogMessages {
            data,
            lastAnalyzedMessageTimeNanos: Some(api::time()),
        }
    }
}
