use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_users: i64,
    pub total_timezones: i64,
    pub timezone_distribution: HashMap<String, i64>,
    pub top_timezones: Vec<TimezoneCount>,
    pub unique_timezones: i64,
    pub top_timezone: String,
    pub recent_registrations: i64,
}

#[derive(Serialize)]
pub struct TimezoneCount {
    pub timezone: String,
    pub count: i64,
}
