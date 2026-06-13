fn median(v: &[f64]) -> Option<f64> {
    // outlier removes
    if v.is_empty() {
        return None;
    }

    let mut sorted = v.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap()); // this short the array
    // in assending order.

    let n = sorted.len(); // this find the length of the array . 
    if n % 2 == 0 {
        // if it's even than this happened
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0) // this will happened if even case basically
    // shorted[element] + shorted[element] and then devide by 2 that gives us avg
    } else {
        // if it's odd then this happened
        Some(sorted[n / 2]) // if it's odd ex : 3 then ans is 1 shorted[1] is your ans 
    }
}

fn slope(y: &[f64]) -> f64 {
    let n = y.len() as f64;

    let x_mean = (n - 1.0) / 2.0;
    let y_mean = y.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &lap) in y.iter().enumerate() {
        let x = i as f64;

        numerator += (x - x_mean) * (lap - y_mean);
        denominator += (x - x_mean).powi(2);
    }

    numerator / denominator
}

pub fn analyze_trend(laps: &[f64]) -> String {
    if laps.len() < 3 {
        return "Insufficient data".to_string();
    }

    let med = match median(laps) {
        Some(m) => m,
        None => return "No valid laps".to_string(),
    };

    let clean: Vec<f64> = laps // this removes 100 bec array is assending shorted ok and
        // here we are do lap - med so let's say median = 90 then 100 - 90 and we do check here <
        // 3.0 so this remvoes 100
        .iter()
        .copied()
        .filter(|lap| (lap - med).abs() < 3.0) // now this filter
        // +- 3 ke upar sare value remove
        .collect();

    // for the regression we need enough clean data
    if clean.len() < 3 {
        return "Insufficient clean data".to_string();
    }

    let trend = slope(&clean);

    if trend > 0.05 {
        format!("Tyre degradation detected. Pace loss {:3} sec/lap", trend)
    } else if trend < -0.05 {
        format!("Pace improving. Gain {:.3} sec/lap", trend.abs())
    } else {
        format!("Stable pace. Trend {:3} sec/lap", trend)
    }
}

pub fn should_pit(laps: &[f64]) -> Option<String> {
    if laps.len() < 5 {
        return None;
    }

    // Linear regression se actual deg rate nikalo
    let n = laps.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = laps.iter().sum::<f64>() / n;

    let numerator: f64 = laps
        .iter()
        .enumerate()
        .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
        .sum();

    let denominator: f64 = laps
        .iter()
        .enumerate()
        .map(|(i, _)| (i as f64 - x_mean).powi(2))
        .sum();

    if denominator == 0.0 {
        return None;
    }

    // Deg rate = slope (seconds per lap)
    let deg_rate = numerator / denominator;

    if deg_rate > 0.15 {
        Some(format!(
            "⚠️  PIT RECOMMENDED — deg rate: +{:.3}s/lap",
            deg_rate
        ))
    } else if deg_rate > 0.05 {
        Some(format!(
            "🟡 Monitor tyres — deg rate: +{:.3}s/lap",
            deg_rate
        ))
    } else {
        None
    }
}
