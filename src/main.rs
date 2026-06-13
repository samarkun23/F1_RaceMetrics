use core::f64;
use std::{collections::HashMap, fmt::format};

use reqwest::Client;
use serde::Deserialize;

mod app;
use app::{Driver, LapData, Session};

use crate::tyreWear::{analyze_trend, should_pit};
mod tyreWear;

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

    println!(
        "🏎️ Session Circuit name:  {} -  Session name : {}\n",
        session.circuit_short_name, session.session_name
    );

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

    println!("═══════════════════════════════════════════════");
    println!("          DRIVERS LIST             ");
    println!("═══════════════════════════════════════════════");
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

        println!("═══════════════════════════════════════════════");
        println!("          FASTEST LAP             ");
        println!("═══════════════════════════════════════════════");
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

    // for the tyre degradation anayalyis
    // driver lap groups
    let mut driver_laps: HashMap<i32, Vec<f64>> = HashMap::new();

    for lap in &laps {
        if let Some(duration) = lap.lap_duration {
            // ignore the pit out lap
            if lap.is_pit_out_lap.unwrap_or(false) {
                continue;
            }
            // outliners ignore like ( Safety car / very slow laps)
            if duration > 120.0 || duration < 60.0 {
                continue;
            }
            driver_laps
                .entry(lap.driver_number)
                .or_default()
                .push(duration);
        }
    }

    println!("═══════════════════════════════════════════════");
    println!("         TYRE DEGRADATION ANALYSIS             ");
    println!("═══════════════════════════════════════════════\n");

    let mut results: Vec<(String, String, f64, f64, String, Option<String>)> = Vec::new();

    for driver in &drivers {
        let num = driver.driver_number;
        let name = driver.name_acronym.as_deref().unwrap_or("???");
        let team = driver.team_name.as_deref().unwrap_or("Unknown");

        if let Some(laps) = driver_laps.get(&num) {
            if laps.is_empty() {
                continue;
            }

            let best = laps.iter().cloned().fold(f64::INFINITY, f64::min);
            let filtered: Vec<f64> = laps.iter().cloned().filter(|&l| l < best * 1.07).collect();

            let avg = filtered.iter().sum::<f64>() / filtered.len() as f64;
            let trend = analyze_trend(&filtered);
            // pit ??
            let pit_rec = should_pit(&filtered);

            results.push((
                name.to_string(),
                team.to_string(),
                best,
                avg,
                trend,
                pit_rec,
            ));
        }
    }

    // we got result let's print it with the best laps
    results.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    for (i, (name, team, best, avg, trend, pit)) in results.iter().enumerate() {
        println!("P{} {} ({})", i + 1, name, team);
        println!("Best : {:.3} | Avg: {:3}s", best, avg);
        println!("   Trend: {}", trend);
        if let Some(p) = pit {
            println!("   {}", p);
        }
        println!();
    }

    for s in sessions {
        println!(
            "Session: {} | Circuit: {} | country: {}",
            s.session_name, s.circuit_short_name, s.country_name,
        )
    }
}
