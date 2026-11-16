//! Device-local customization. Deliberately absent from backup serialization.
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct City {
    pub name: String,
    pub region: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub audio_background: AudioBackground,
    pub weather_enabled: bool,
    pub gmail_enabled: bool,
    pub spotify_enabled: bool,
    pub spotify_client_id: String,
    pub automatic_location: bool,
    pub fahrenheit: bool,
    pub city: Option<City>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AudioBackground {
    pub enabled: bool,
    pub color: String,
    pub intensity: f32,
    pub style: String,
}
/// Visual styles the new-tab canvas knows how to draw (see `public/new-tab/audio-styles.js`).
pub const AUDIO_STYLES: [&str; 20] = [
    "bars", "wave", "area", "lines", "pulse", "radial", "ring", "concentric", "spiral", "vinyl", "dots",
    "particles", "bubbles", "fireflies", "glow", "orbs", "spectrogram", "aurora", "polygon", "terrain",
];
impl Default for AudioBackground {
    fn default() -> Self {
        Self {
            enabled: false,
            color: "#1db954".into(),
            intensity: 1.0,
            style: "bars".into(),
        }
    }
}
impl AudioBackground {
    pub fn validate(&self) -> Result<(), String> {
        if self.color.len() != 7
            || !self.color.starts_with('#')
            || !self.color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            || !self.intensity.is_finite()
            || !(0.5..=2.0).contains(&self.intensity)
            || !AUDIO_STYLES.contains(&self.style.as_str())
        {
            return Err("invalid_settings".into());
        }
        Ok(())
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        self.audio_background.validate()?;
        if !self.spotify_client_id.is_empty()
            && (self.spotify_client_id.len() != 32
                || !self
                    .spotify_client_id
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit()))
        {
            return Err("invalid_settings".into());
        }
        if let Some(city) = &self.city {
            if city.name.trim().is_empty()
                || [&city.name, &city.region, &city.country]
                    .iter()
                    .any(|s| s.len() > 200)
                || !city.latitude.is_finite()
                || !city.longitude.is_finite()
                || !(-90.0..=90.0).contains(&city.latitude)
                || !(-180.0..=180.0).contains(&city.longitude)
            {
                return Err("invalid_settings".into());
            }
        }
        Ok(())
    }
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "new-tab-widgets-v1", |conn| {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS new_tab_settings (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL)").map_err(|_| "settings_unavailable".into())
    })?;
    super::migrations::apply(conn, "new-tab-spotify-account-v1", |conn| {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS new_tab_spotify_account (id INTEGER PRIMARY KEY CHECK(id=1), disconnected INTEGER NOT NULL)").map_err(|_| "settings_unavailable".into())
    })?;
    super::migrations::apply(conn, "new-tab-account-revocation-v1", |conn| {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS new_tab_account_state (id INTEGER PRIMARY KEY CHECK(id=1), disconnected INTEGER NOT NULL)").map_err(|_|"settings_unavailable".into())
    })
}
pub fn spotify_disconnected(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT disconnected FROM new_tab_spotify_account WHERE id=1",
        [],
        |r| r.get(0),
    )
    .optional()
    .map(|v| v.unwrap_or(true))
    .map_err(|_| "settings_unavailable".into())
}
pub fn set_spotify_disconnected(conn: &Connection, value: bool) -> Result<(), String> {
    conn.execute("INSERT INTO new_tab_spotify_account(id,disconnected) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET disconnected=excluded.disconnected", [value])
        .map(|_| ()).map_err(|_| "settings_unavailable".into())
}
pub fn account_disconnected(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT disconnected FROM new_tab_account_state WHERE id=1",
        [],
        |r| r.get(0),
    )
    .optional()
    .map(|v| v.unwrap_or(false))
    .map_err(|_| "settings_unavailable".into())
}
pub fn set_account_disconnected(conn: &Connection, disconnected: bool) -> Result<(), String> {
    conn.execute("INSERT INTO new_tab_account_state(id,disconnected) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET disconnected=excluded.disconnected",[disconnected]).map(|_|()).map_err(|_|"settings_unavailable".into())
}
pub fn load(conn: &Connection) -> Result<Settings, String> {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM new_tab_settings WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|_| "settings_unavailable")?;
    let settings: Settings = raw
        .map(|v| serde_json::from_str(&v))
        .transpose()
        .map_err(|_| "settings_unavailable")?
        .unwrap_or_default();
    settings.validate()?;
    Ok(settings)
}
pub fn save(conn: &Connection, settings: &Settings) -> Result<(), String> {
    settings.validate()?;
    conn.execute("INSERT INTO new_tab_settings(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value", [serde_json::to_string(settings).map_err(|_| "invalid_settings")?]).map_err(|_| "settings_unavailable")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_preferences_are_bounded_and_old_settings_keep_the_effect_disabled() {
        let mut s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.audio_background, AudioBackground::default());
        s.audio_background.enabled = true;
        for color in ["red", "#12345", "#1234567", "#gggggg"] {
            s.audio_background.color = color.into();
            assert!(s.validate().is_err());
        }
        s.audio_background.color = "#Ab12ef".into();
        for intensity in [f32::NAN, f32::INFINITY, 0.0, 2.1] {
            s.audio_background.intensity = intensity;
            assert!(s.validate().is_err());
        }
        s.audio_background.intensity = 1.5;
        for style in ["", "Bars", "unknown", "wave "] {
            s.audio_background.style = style.into();
            assert!(s.validate().is_err());
        }
        for style in AUDIO_STYLES {
            s.audio_background.style = style.into();
            assert!(s.validate().is_ok());
        }
        s.audio_background.style = "radial".into();
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        save(&db, &s).unwrap();
        assert_eq!(load(&db).unwrap(), s);
    }
    #[test]
    fn spotify_defaults_client_validation_and_disconnection_are_independent() {
        let mut s: Settings = serde_json::from_str(r#"{"weather_enabled":true}"#).unwrap();
        assert!(!s.spotify_enabled);
        assert!(s.spotify_client_id.is_empty());
        s.spotify_client_id = "a".repeat(32);
        assert!(s.validate().is_ok());
        s.spotify_client_id = "g".repeat(32);
        assert!(s.validate().is_err());
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        assert!(spotify_disconnected(&db).unwrap());
        set_spotify_disconnected(&db, false).unwrap();
        assert!(!spotify_disconnected(&db).unwrap());
        set_account_disconnected(&db, true).unwrap();
        assert!(!spotify_disconnected(&db).unwrap());
        set_spotify_disconnected(&db, true).unwrap();
        assert!(spotify_disconnected(&db).unwrap());
    }
    #[test]
    fn migration_defaults_roundtrip_and_validation() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        migrate(&db).unwrap();
        assert_eq!(load(&db).unwrap(), Settings::default());
        let mut s = Settings {
            weather_enabled: true,
            city: Some(City {
                name: "São Paulo".into(),
                region: "SP".into(),
                country: "BR".into(),
                latitude: -23.5,
                longitude: -46.6,
            }),
            ..Default::default()
        };
        save(&db, &s).unwrap();
        assert_eq!(load(&db).unwrap(), s);
        s.city.as_mut().unwrap().latitude = 91.0;
        assert!(save(&db, &s).is_err());
        assert_ne!(load(&db).unwrap(), s);
    }
    #[test]
    fn local_customization_never_enters_backup_data() {
        let db = Connection::open_in_memory().unwrap();
        crate::database::connection::initialize_schema(&db).unwrap();
        save(
            &db,
            &Settings {
                weather_enabled: true,
                spotify_enabled: true,
                spotify_client_id: "0123456789abcdef".repeat(2),
                city: Some(City {
                    name: "private city marker".into(),
                    region: "region".into(),
                    country: "country".into(),
                    latitude: 12.345,
                    longitude: 67.89,
                }),
                ..Default::default()
            },
        )
        .unwrap();
        let exported =
            serde_json::to_string(&crate::database::backup_repo::collect(&db).unwrap()).unwrap();
        assert!(!exported.contains("private city marker"));
        assert!(!exported.contains("weather_enabled"));
        assert!(!exported.contains("spotify_"));
        assert!(!exported.contains("audio_background"));
        assert!(!exported.contains(&"0123456789abcdef".repeat(2)));
    }
}
