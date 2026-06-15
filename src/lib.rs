#![no_std]
use core::fmt::Write;
use heapless::String;
use pmsa003i::{AirQuality, AirQualityLevel, Reading};

pub struct EnvReading {
    aqi_pm2_5: Option<AirQuality>,
    aqi_pm10: Option<AirQuality>,
    p_gt_0_3: u16,
    p_gt_0_5: u16,
    p_gt_1: u16,
    p_gt_2_5: u16,
    p_gt_5: u16,
    p_gt_10: u16,
    pm1: u16,
    pm2_5: u16,
    pm10: u16,
    pm1_env: u16,
    pm2_5_env: u16,
    pm10_env: u16,
}

impl EnvReading {
    pub fn aqi_pm2_5_str(&self) -> String<26> {
        let mut msg: String<26> = String::new();
        write!(&mut msg, "PM2.5: {}",
               if let Some(aqi_pm2_5) = &self.aqi_pm2_5 {
                   match aqi_pm2_5.level() {
                       AirQualityLevel::Good => "Good",
                       AirQualityLevel::Moderate => "Moderate",
                       AirQualityLevel::UnhealthySensitive => "UnhealthySensitive",
                       AirQualityLevel::Unhealthy => "Unhealthy",
                       AirQualityLevel::VeryUnhealthy => "VeryUnhealthy",
                       AirQualityLevel::Hazardous => "Hazardous",
                   }
               } else { "n/a" }
        ).unwrap();
        msg
    }

    pub fn aqi_pm10_str(&self) -> String<25> {
        let mut msg: String<25> = String::new();
        write!(&mut msg, "PM10 : {}",
               if let Some(aqi_pm10) = &self.aqi_pm10 {
                   match aqi_pm10.level() {
                       AirQualityLevel::Good => "Good",
                       AirQualityLevel::Moderate => "Moderate",
                       AirQualityLevel::UnhealthySensitive => "UnhealthySensitive",
                       AirQualityLevel::Unhealthy => "Unhealthy",
                       AirQualityLevel::VeryUnhealthy => "VeryUnhealthy",
                       AirQualityLevel::Hazardous => "Hazardous",
                   }
               } else { "n/a" }
        ).unwrap();
        msg
    }

    pub fn p_gt_0_3_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">0.3 µm: {}", self.p_gt_0_3).unwrap();
        msg
    }

    pub fn p_gt_0_5_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">0.5 µm: {}", self.p_gt_0_5).unwrap();
        msg
    }

    pub fn p_gt_1_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">1 µm  : {}", self.p_gt_1).unwrap();
        msg
    }

    pub fn p_gt_2_5_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">2.5 µm: {}", self.p_gt_2_5).unwrap();
        msg
    }

    pub fn p_gt_5_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">5 µm  : {}", self.p_gt_5).unwrap();
        msg
    }

    pub fn p_gt_10_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, ">10 µm : {}", self.p_gt_10).unwrap();
        msg
    }

    pub fn pm1_str(&self) -> String<23> {
        let mut msg: String<23> = String::new();
        write!(&mut msg, "1.0: {} µg/m³", self.pm1).unwrap();
        msg
    }

    pub fn pm2_5_str(&self) -> String<23> {
        let mut msg: String<23> = String::new();
        write!(&mut msg, "2.5: {} µg/m³", self.pm2_5).unwrap();
        msg
    }

    pub fn pm10_str(&self) -> String<22> {
        let mut msg: String<22> = String::new();
        write!(&mut msg, "10 : {} µg/m³", self.pm10).unwrap();
        msg
    }

    pub fn pm1_env_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, "1.0: {} µg/m³", self.pm1).unwrap();
        msg
    }

    pub fn pm2_5_env_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, "2.5: {} µg/m³", self.pm2_5).unwrap();
        msg
    }

    pub fn pm10_env_str(&self) -> String<20> {
        let mut msg: String<20> = String::new();
        write!(&mut msg, "10 : {} µg/m³", self.pm10).unwrap();
        msg
    }
}

impl From<Reading> for EnvReading {
    fn from(reading: Reading) -> Self {
        Self {
            aqi_pm2_5: reading.aqi_pm2_5.ok(),
            aqi_pm10: reading.aqi_pm10.ok(),
            p_gt_0_3: reading.particles_larger_than_0_3,
            p_gt_0_5: reading.particles_larger_than_0_5,
            p_gt_1: reading.particles_larger_than_1,
            p_gt_2_5: reading.particles_larger_than_2_5,
            p_gt_5: reading.particles_larger_than_5,
            p_gt_10: reading.particles_larger_than_10,
            pm1_env: reading.env_pm1,
            pm2_5_env: reading.env_pm2_5,
            pm10_env: reading.env_pm10,
            pm1: reading.pm1,
            pm2_5: reading.pm2_5,
            pm10: reading.pm10,
        }
    }
}

pub enum View {
    Aqi,
    ParticleDiameter1,
    ParticleDiameter2,
    Pm,
    PmEnv,
}

pub struct State {
    pub env_reading: Option<EnvReading>,
    pub view: View
}
impl State {
    pub const fn new() -> Self {
        Self { env_reading: None, view: View::Pm }
    }
}

pub enum AppEvent {
    LeftBtnClicked,
    EnvReadingTaken(Reading),
    RightBtnClicked,
}