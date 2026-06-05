use serde::Deserialize;

use crate::config::WeatherConfig;
use crate::notifier::{Notification, Notifier};
use crate::storage::Storage;

const STORAGE_KEY: &str = "weather_alerted_dates";

pub struct Weather {
    config: WeatherConfig,
}

impl Weather {
    pub fn new(config: WeatherConfig) -> Self {
        Self { config }
    }
}

#[derive(Deserialize)]
struct DailyData {
    time: Vec<String>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    weathercode: Vec<i64>,
    wind_speed_10m_max: Vec<f64>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    daily: DailyData,
}

impl Notifier for Weather {
    fn name(&self) -> &str {
        "weather"
    }

    fn check(&self, storage: &mut Storage) -> Option<Notification> {
        let forecast = match fetch_forecast(&self.config) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[weather] fetch error: {e}");
                return None;
            }
        };

        let alerted: Vec<String> = storage
            .get(STORAGE_KEY)
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let mut day_alerts: Vec<String> = Vec::new();
        let mut newly_alerted: Vec<String> = Vec::new();

        for i in 0..forecast.daily.time.len() {
            let date = &forecast.daily.time[i];
            if alerted.contains(date) || newly_alerted.contains(date) {
                continue;
            }

            let min_temp = forecast.daily.temperature_2m_min[i];
            let max_temp = forecast.daily.temperature_2m_max[i];
            let wcode = forecast.daily.weathercode[i];
            let wind = forecast.daily.wind_speed_10m_max[i];

            let mut reasons: Vec<String> = Vec::new();

            if let Some(threshold) = self.config.min_night_temp
                && min_temp < threshold
            {
                reasons.push(format!(
                    "Night low {min_temp:.0}°C is below {threshold:.0}°C"
                ));
            }

            if let Some(threshold) = self.config.max_day_temp
                && max_temp > threshold
            {
                reasons.push(format!("Day high {max_temp:.0}°C exceeds {threshold:.0}°C"));
            }

            if self.config.extreme_weather {
                match wcode {
                    95..=99 => reasons.push("Thunderstorm forecast".into()),
                    65 | 82 => reasons.push("Heavy rain forecast".into()),
                    75 | 86 => reasons.push("Heavy snow forecast".into()),
                    _ => {}
                }
                if wind > 50.0 {
                    reasons.push(format!("Strong wind ({:.0} km/h)", wind));
                }
            }

            if !reasons.is_empty() {
                day_alerts.push(format!("{date}:\n  {}", reasons.join("\n  ")));
                newly_alerted.push(date.clone());
            }
        }

        if day_alerts.is_empty() {
            return None;
        }

        let mut merged: Vec<String> = alerted;
        merged.extend(newly_alerted);
        storage.set(STORAGE_KEY, &merged.join(","));

        Some(Notification {
            title: format!(
                "Weather alert ({} day{})",
                day_alerts.len(),
                if day_alerts.len() == 1 { "" } else { "s" }
            ),
            body: day_alerts.join("\n\n"),
        })
    }
}

fn fetch_forecast(config: &WeatherConfig) -> Result<ForecastResponse, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,weathercode,wind_speed_10m_max&timezone=auto&forecast_days=3",
        config.latitude, config.longitude
    );

    let resp = ureq::get(&url)
        .header("User-Agent", "simple-notifier/0.1")
        .call()
        .map_err(|e| format!("HTTP error: {e}"))?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| format!("read error: {e}"))?;

    serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {e}"))
}
