#![no_std]

pub mod view;

use core::fmt::Write;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use esp_hal::Async;
use esp_hal::i2c::master::I2c;
use heapless::String;
use pmsa003i::{AirQuality, AirQualityLevel, Reading};
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::prelude::{DisplaySize128x64, I2CInterface};
use ssd1306::Ssd1306Async;

pub type Display = Ssd1306Async<I2CInterface<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>>, DisplaySize128x64, BufferedGraphicsModeAsync<DisplaySize128x64>>;
pub type DisplayTextStyle = MonoTextStyle<'static, BinaryColor>;
pub type I2cAsyncMutex = Mutex<NoopRawMutex, I2c<'static, Async>>;

#[derive(Clone, Copy, Debug)]
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
        write!(&mut msg, "1.0: {} µg/m³", self.pm1_env).unwrap();
        msg
    }

    pub fn pm2_5_env_str(&self) -> String<21> {
        let mut msg: String<21> = String::new();
        write!(&mut msg, "2.5: {} µg/m³", self.pm2_5_env).unwrap();
        msg
    }

    pub fn pm10_env_str(&self) -> String<20> {
        let mut msg: String<20> = String::new();
        write!(&mut msg, "10 : {} µg/m³", self.pm10_env).unwrap();
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

#[derive(Debug, Default, PartialEq)]
pub enum State {
    #[default]
    StartUp,
    Ready,
}

pub struct App {
    pub env_reading: Option<EnvReading>,
    state: State,
}
impl App {
    pub const fn new() -> Self {
        Self { env_reading: None, state: State::StartUp }
    }

    pub fn is_ready(&self) -> bool {
        self.state == State::Ready
    }

    pub fn ready(&mut self) {
        self.state = State::Ready;
    }
}

pub enum AppEvent {
    AqiBtnClicked,
    LeftBtnClicked,
    Pmsa003iReadingTaken(Reading),
    Pmsa003iReady,
    RightBtnClicked,
}