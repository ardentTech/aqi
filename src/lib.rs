#![no_std]
use core::fmt::Write;
use heapless::String;
use pmsa003i::{AirQuality, AirQualityLevel, Reading};

pub struct EnvReading {
    aqi_pm2_5: Option<AirQuality>,
    aqi_pm10: Option<AirQuality>,
    pm1: u16,
    pm2_5: u16,
    pm10: u16,
}

impl EnvReading {
    pub fn aqi_pm2_5_str(&self) -> String<26> {
        let mut msg: String<26> = String::new();
        write!(&mut msg, "PM2.5 = {}",
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
        write!(&mut msg, "PM10 = {}",
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

    pub fn pm1_str(&self) -> String<22> {
        let mut msg: String<22> = String::new();
        write!(&mut msg, "PM1.0 = {}μg/𝑚3", self.pm1).unwrap();
        msg
    }

    pub fn pm2_5_str(&self) -> String<22> {
        let mut msg: String<22> = String::new();
        write!(&mut msg, "PM2.5 = {}μg/𝑚3", self.pm2_5).unwrap();
        msg
    }

    pub fn pm10_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, "PM10 = {}μg/𝑚3", self.pm10).unwrap();
        msg
    }
}

impl From<Reading> for EnvReading {
    fn from(reading: Reading) -> Self {
        Self {
            aqi_pm2_5: reading.aqi_pm2_5.ok(),
            aqi_pm10: reading.aqi_pm10.ok(),
            pm1: reading.pm1,
            pm2_5: reading.pm2_5,
            pm10: reading.pm10,
        }
    }
}

enum DisplayView {
    Aqi,
    Pm,
}

enum Event {
    AqiRead,
    DisplayRendered
}