use chrono::NaiveDateTime;
use serde::Serialize;

pub fn fmt_ts(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn fmt_ts_opt(dt: Option<NaiveDateTime>) -> Option<String> {
    dt.map(fmt_ts)
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}
