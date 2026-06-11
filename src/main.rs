use std::fmt::format;

use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Session {
    session_key: i64,
    session_name: String,
    date_start: String,
    circuit_short_name: String,
    country_name: String,
}

#[derive(Deserialize, Debug)]
struct LapData {
    driver_number: i32,
    lap_number: i32,
    lap_duration: Option<f64>,
    duration_sector_1: Option<f64>,
    duration_sector_2: Option<f64>,
    duration_sector_3: Option<f64>,
    is_pit_out_lap: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Driver {
    driver_number: i32,
    full_name: Option<String>,
    team_name: Option<String>,
    name_acronym: Option<String>,
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    let sessions = client
        .get("https://api.openf1.org/v1/sessions?session_key=latest")
        .send()
        .await
        .unwrap()
        .json::<Vec<Session>>()
        .await
        .unwrap();

    let session = &sessions[0];
    let session_key = session.session_key;

    println!("{} - {}", session.circuit_short_name, session.session_name);

    // drivers of this session
    let drivers = client
        .get(format!(
            "https://api.openf1.org/v1/drivers?session_key={}",
            session_key
        ))
        .send()
        .await
        .unwrap()
        .json::<Vec<Driver>>()
        .await
        .unwrap();

    println!("\n Drivers ({}):", drivers.len());

    for d in &drivers {
        println!(
            " #{} {} - {}",
            d.driver_number,
            d.name_acronym.as_deref().unwrap_or("???"),
            d.team_name.as_deref().unwrap_or("Unknown")
        );
    }

    // time laps for this session
    let laps = client
        .get(format!(
            "https://api.openf1.org/v1/laps?session_key={}",
            session_key
        ))
        .send()
        .await
        .unwrap()
        .json::<Vec<LapData>>()
        .await
        .unwrap();

    println!("\n Total laps recorded {}", laps.len());

    // fastest lap
    let fastest = laps
        .iter()
        .filter(|l| l.lap_duration.is_some() && !l.is_pit_out_lap.unwrap_or(false))
        .min_by(|a, b| {
            a.lap_duration
                .unwrap()
                .partial_cmp(&b.lap_duration.unwrap())
                .unwrap()
        });

    if let Some(lap) = fastest {
        let driver = drivers
            .iter()
            .find(|d| d.driver_number == lap.driver_number);

        println!("\n🏆 Fastest Lap: {:.3}s", lap.lap_duration.unwrap());

        println!(
            "Driver: #{} {}",
            lap.driver_number,
            driver
                .and_then(|d| d.name_acronym.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("Unknown")
        );

        println!(" Lap: {}", lap.lap_number);

        println!(
            "Sectors: S1={:.3} | S2={:.3} | S3={:.3}",
            lap.duration_sector_1.unwrap_or(0.0),
            lap.duration_sector_2.unwrap_or(0.0),
            lap.duration_sector_3.unwrap_or(0.0),
        );
    }

    for s in sessions {
        println!(
            "Session: {} | Circuit: {} | country: {}",
            s.session_name, s.circuit_short_name, s.country_name,
        )
    }
}
