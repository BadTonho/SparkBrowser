use super::http;
use crate::database::new_tab_repo::City;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Day {
    pub date: String,
    pub code: u16,
    pub min: f64,
    pub max: f64,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Forecast {
    pub temperature: f64,
    pub feels_like: f64,
    pub code: u16,
    pub days: Vec<Day>,
    pub updated: u64,
}
pub fn cities(endpoint: &str, query: &str, language: &str) -> Result<Vec<City>, String> {
    let value: Value = http::json(http::send(http::client()?.get(endpoint).query(&[
        ("name", query),
        ("count", "5"),
        ("language", if language == "pt-BR" { "pt" } else { "en" }),
        ("format", "json"),
    ]))?)?;
    parse_cities(&value)
}
fn parse_cities(value: &Value) -> Result<Vec<City>, String> {
    let Some(rows) = value.get("results") else {
        return Ok(vec![]);
    };
    rows.as_array()
        .ok_or("invalid_response")?
        .iter()
        .take(5)
        .map(|v| {
            let city = City {
                name: v["name"]
                    .as_str()
                    .ok_or("invalid_response")?
                    .chars()
                    .take(200)
                    .collect(),
                region: v["admin1"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(200)
                    .collect(),
                country: v["country"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(200)
                    .collect(),
                latitude: number(&v["latitude"])?,
                longitude: number(&v["longitude"])?,
            };
            crate::database::new_tab_repo::Settings {
                city: Some(city.clone()),
                ..Default::default()
            }
            .validate()?;
            Ok(city)
        })
        .collect()
}
pub fn fetch(endpoint: &str, coordinates: (f64, f64)) -> Result<Forecast, String> {
    let (latitude, longitude) = coordinates;
    let value: Value = http::json(http::send(http::client()?.get(endpoint).query(&[
        ("latitude", latitude.to_string()),
        ("longitude", longitude.to_string()),
        (
            "current",
            "temperature_2m,apparent_temperature,weather_code".into(),
        ),
        (
            "daily",
            "weather_code,temperature_2m_max,temperature_2m_min".into(),
        ),
        ("forecast_days", "3".into()),
        ("timezone", "auto".into()),
    ]))?)?;
    parse_forecast(&value)
}
fn number(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "invalid_response".into())
}
fn code(v: &Value) -> Result<u16, String> {
    v.as_u64()
        .filter(|v| *v <= 99)
        .map(|v| v as u16)
        .ok_or_else(|| "invalid_response".into())
}
fn parse_forecast(v: &Value) -> Result<Forecast, String> {
    let mut days = Vec::new();
    for i in 0..3 {
        days.push(Day {
            date: v["daily"]["time"][i]
                .as_str()
                .filter(|s| valid_date(s))
                .ok_or("invalid_response")?
                .into(),
            code: code(&v["daily"]["weather_code"][i])?,
            min: number(&v["daily"]["temperature_2m_min"][i])?,
            max: number(&v["daily"]["temperature_2m_max"][i])?,
        });
    }
    Ok(Forecast {
        temperature: number(&v["current"]["temperature_2m"])?,
        feels_like: number(&v["current"]["apparent_temperature"])?,
        code: code(&v["current"]["weather_code"])?,
        days,
        updated: super::now(),
    })
}
fn valid_date(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit())
    {
        return false;
    }
    let year = raw[..4].parse::<u32>().unwrap_or(0);
    let month = raw[5..7].parse::<usize>().unwrap_or(0);
    let day = raw[8..].parse::<u32>().unwrap_or(0);
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    (1..=days[month - 1]).contains(&day)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_cities_and_forecast() {
        let cities=parse_cities(&serde_json::json!({"results":[{"name":"Springfield","admin1":"Illinois","country":"US","latitude":39.8,"longitude":-89.6}]})).unwrap();
        assert_eq!(cities[0].region, "Illinois");
        assert!(parse_cities(
            &serde_json::json!({"results":[{"name":"Invalid","latitude":100,"longitude":0}]})
        )
        .is_err());
        assert!(parse_forecast(&serde_json::json!({})).is_err());
        let forecast=parse_forecast(&serde_json::json!({"current":{"temperature_2m":20,"apparent_temperature":19,"weather_code":3},"daily":{"time":["2026-11-01","2026-11-02","2026-11-03"],"weather_code":[0,3,61],"temperature_2m_min":[10,11,12],"temperature_2m_max":[20,21,22]}})).unwrap();
        assert_eq!(forecast.days.len(), 3);
        assert_eq!(forecast.temperature, 20.0);
        assert!(!valid_date("2026-02-29"));
        assert!(valid_date("2028-02-29"));
        assert!(!valid_date("2026-99-01"));
    }
}
