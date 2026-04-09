use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TimezoneResponse {
    pub user: UserInfo,
    pub timezone: String,
}

#[derive(Serialize)]
pub struct MinimalUserInfo {
    pub username: String,
    pub timezone: String,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct GetQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SetQuery {
    pub timezone: String,
}
