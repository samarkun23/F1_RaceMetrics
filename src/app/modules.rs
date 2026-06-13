use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Session {
    pub session_key: i64,
    pub session_name: String,
    pub date_start: String,
    pub circuit_short_name: String,
    pub country_name: String,
}

#[derive(Deserialize, Debug)]
pub struct LapData {
    pub driver_number: i32,
    pub lap_number: i32,
    pub lap_duration: Option<f64>,
    pub duration_sector_1: Option<f64>,
    pub duration_sector_2: Option<f64>,
    pub duration_sector_3: Option<f64>,
    pub is_pit_out_lap: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Driver {
    pub driver_number: i32,
    pub full_name: Option<String>,
    pub team_name: Option<String>,
    pub name_acronym: Option<String>,
}
