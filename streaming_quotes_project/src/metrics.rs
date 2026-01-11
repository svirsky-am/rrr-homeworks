// src/metrics.rs

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMetrics {
    pub timestamp: u64,   // Unix timestamp в секундах
    // pub temperature: f32, // °C
    // pub humidity: f32,    // %
    // pub pressure: f32,    // hPa
    // pub door_open: bool,
    // pub vibration_level: f32, // — уровень вибрации (для обнаружения взломов)
    // pub light_level: f32, // — уровень освещённости
    // pub noise_level: f32, // — уровень шума
    // pub co2_level: f32, // — уровень CO2
    // pub air_quality: f32, // — качество воздуха
    // pub water_leak_detected: bool, // — обнаружена утечка воды
    // pub fire_detected: bool, // — обнаружен пожар
}

impl RoomMetrics {
    pub fn new(temperature: f32, humidity: f32, pressure: f32, door_open: bool, 
        vibration_level: f32
    //     light_level: f32,
    //     noise_level: f32,
    //     co2_level: f32, 
    //     air_quality: f32,
    //     water_leak_detected: bool,
    //     fire_detected: bool
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            // temperature,
            // humidity,
            // pressure,
            // door_open,
            // vibration_level,
            // light_level,
            // noise_level,
            // co2_level,
            // air_quality,
            // water_leak_detected,
            // fire_detected,

        }
    }

    // Метод для имитации метрик
    #[cfg(feature = "random")]
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self::new(
            rng.gen_range(18.0..25.0),
            rng.gen_range(30.0..60.0),
            rng.gen_range(980.0..1020.0),
            rng.gen_bool(0.1), // 10% chance door is open
            rng.gen_range(5.0..6.0),
            // rng.gen_range(5.0..6.0),
            // rng.gen_range(5.0..6.0),
            // rng.gen_range(5.0..6.0),
            // rng.gen_range(5.0..6.0),
            // rng.gen_bool(0.5),
            // rng.gen_bool(0.5),
            // light_level: f32,
            // noise_level: f32,
            // co2_level: f32, 
            // air_quality: f32,
            // water_leak_detected: bool,
            // fire_detected: bool
        
        )
    }

    // Альтернативная реализация без фичи random
    #[cfg(not(feature = "random"))]
    pub fn random() -> Self {
        // Простая детерминистическая реализация
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();
        
        Self::new(
            20.0 + ((hash % 1000) as f32 / 100.0), // 20.0-30.0
            40.0 + ((hash % 1000) as f32 / 50.0),  // 40.0-60.0
            1000.0 + ((hash % 400) as f32 - 200.0), // 800.0-1200.0
            (hash % 10) == 0, // 10% chance
            4.0 + ((hash % 1000) as f32 / 50.0), 
                        4.0 + ((hash % 1000) as f32 / 50.0), 
                        4.0 + ((hash % 1000) as f32 / 50.0), 
                        4.0 + ((hash % 1000) as f32 / 50.0), 
                        4.0 + ((hash % 1000) as f32 / 50.0), 
                        (hash % 5) == 0,
                        (hash % 2) == 0 

        )
    }

    // Дополнительный метод, доступный только с фичей "sqlite"
    #[cfg(feature = "sqlite")]
    pub fn to_sql(&self) -> String {
        format!(
            "INSERT INTO metrics (timestamp, temperature, humidity, pressure, door_open) VALUES ({}, {:.1}, {:.1}, {:.1}, {})",
            self.timestamp, self.temperature, self.humidity, self.pressure, self.door_open
        )
    }
    // Метод для форматированного отображения времени
    pub fn formatted_time(&self) -> String {
        format!("{}s", self.timestamp)
    }
}