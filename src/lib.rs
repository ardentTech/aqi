#![no_std]
use core::fmt::Write;
use heapless::String;
use pmsa003i::Reading;

pub struct EnvReading {
    pm1: u16,
    pm2_5: u16,
    pm10: u16
}

impl EnvReading {
    pub fn pm1_str(&self) -> String<22> {
        let mut msg: String<22> = String::new();
        write!(&mut msg, "PM1 = {}", self.pm1).unwrap();
        msg
    }

    pub fn pm2_5_str(&self) -> String<24> {
        let mut msg: String<24> = String::new();
        write!(&mut msg, "PM2.5 = {}", self.pm2_5).unwrap();
        msg
    }

    pub fn pm10_str(&self) -> String<23> {
        let mut msg: String<23> = String::new();
        write!(&mut msg, "PM10 = {}", self.pm10).unwrap();
        msg
    }
}

impl From<Reading> for EnvReading {
    fn from(reading: Reading) -> Self {
        Self {
            pm1: reading.pm1,
            pm2_5: reading.pm2_5,
            pm10: reading.pm10,
        }
    }
}