#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{error, info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::signal;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::clock::CpuClock;
use esp_hal::{i2c, Async};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
#[allow(unused_imports)]
use {esp_backtrace as _, esp_println as _};
use pmsa003i::{Pmsa003i, Reading};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, Ssd1306Async};
use ssd1306::mode::BufferedGraphicsModeAsync;
use static_cell::StaticCell;
use lib::EnvReading;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

type I2cAsyncMutex = Mutex<NoopRawMutex, I2c<'static, Async>>;

static AQI_READING: signal::Signal<CriticalSectionRawMutex, Reading> = signal::Signal::new();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c3 -o esp32c3-mini-1 -o unstable-hal -o embassy -o defmt -o esp-backtrace

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO2
    // - GPIO8
    // - GPIO9
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO12;
    let _ = peripherals.GPIO13;
    let _ = peripherals.GPIO14;
    let _ = peripherals.GPIO15;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO17;

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<Async>>> = StaticCell::new();
    let i2c = I2c::new(peripherals.I2C0, Config::default().with_frequency(Rate::from_khz(100))).unwrap()
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO20)
        .into_async();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    spawner.spawn(aqi_task(i2c_bus).unwrap());
    spawner.spawn(display_task(i2c_bus).unwrap());

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn aqi_task(i2c_bus: &'static I2cAsyncMutex) {
    info!("aqi_task");
    let mut pmsa003i = Pmsa003i::new(I2cDevice::new(i2c_bus));
    loop {
        match pmsa003i.read().await {
            Ok(reading) => {
                AQI_READING.signal(reading)
            }
            Err(_) => error!("pmsa003i.read failed"),
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn display_task(i2c_bus: &'static I2cAsyncMutex) {
    info!("display_task");
    let interface = I2CDisplayInterface::new(I2cDevice::new(i2c_bus));
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().await.unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let reading = AQI_READING.wait().await;
        let env_reading: EnvReading = reading.into();
        Text::with_baseline(&*env_reading.pm1_str(), Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&*env_reading.pm2_5_str(), Point::new(0, 32), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(&*env_reading.pm10_str(), Point::new(0, 48), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().await.unwrap();
    }
}