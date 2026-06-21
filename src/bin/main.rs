#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{debug, error, info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::signal;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::Async;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
#[allow(unused_imports)]
use {esp_backtrace as _, esp_println as _};
use pmsa003i::Pmsa003i;
use static_cell::StaticCell;
use lib::{AppEvent, App, I2cAsyncMutex};
use lib::view::{display_task, ViewCmd, VIEW_CMD};
// TODO auto and manual mode

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const PMSA003I_STARTUP_SEC: u64 = 1; // TODO should be 30
static TAKE_ENV_READING: signal::Signal<CriticalSectionRawMutex, ()> = signal::Signal::new();
static EVENT_BUS: Channel<CriticalSectionRawMutex, AppEvent, 8> = Channel::new();
static APP: Mutex<CriticalSectionRawMutex, App> = Mutex::new(App::new());

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c3 -o esp32c3-mini-1 -o unstable-hal -o embassy -o defmt -o esp-backtrace

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    //let rtc = Rtc::new(peripherals.LPWR);

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
    // 100 kHz only for AQI sensor
    let i2c = I2c::new(peripherals.I2C0, Config::default().with_frequency(Rate::from_khz(100))).unwrap()
        .with_sda(peripherals.GPIO3)  // using GPIO21 broke defmt
        .with_scl(peripherals.GPIO20)
        .into_async();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    info!("spawning...");
    spawner.spawn(orchestration().unwrap());
    spawner.spawn(aqi_task(i2c_bus).unwrap());
    spawner.spawn(display_task(i2c_bus).unwrap());
    spawner.spawn(left_btn(Input::new(peripherals.GPIO6, InputConfig::default().with_pull(Pull::Down))).unwrap());
    spawner.spawn(right_btn(Input::new(peripherals.GPIO7, InputConfig::default().with_pull(Pull::Down))).unwrap());
    spawner.spawn(aqi_btn(Input::new(peripherals.GPIO5, InputConfig::default().with_pull(Pull::Down))).unwrap());

    Timer::after(Duration::from_secs(PMSA003I_STARTUP_SEC)).await; // see: PMSA003I datasheet, page 9, section "Circuit Attentions", #4
    EVENT_BUS.sender().send(AppEvent::Pmsa003iReady).await;
}

#[embassy_executor::task]
async fn aqi_btn(mut btn: Input<'static>) {
    debug!("aqi_btn");
    loop {
        btn.wait_for_rising_edge().await;
        debug!("aqi btn clicked");
        EVENT_BUS.sender().send(AppEvent::AqiBtnClicked).await;
    }
}

#[embassy_executor::task]
async fn aqi_task(i2c_bus: &'static I2cAsyncMutex) {
    debug!("aqi_task");
    let mut pmsa003i = Pmsa003i::new(I2cDevice::new(i2c_bus)); // i2c addr: 0x12
    loop {
        TAKE_ENV_READING.wait().await;
        debug!("TAKE_ENV_READING recv");
        match pmsa003i.read().await {
            Ok(reading) => {
                EVENT_BUS.sender().send(AppEvent::Pmsa003iReadingTaken(reading)).await;
            }
            Err(_) => error!("pmsa003i.read failed"),
        }
    }
}

#[embassy_executor::task]
async fn left_btn(mut btn: Input<'static>) {
    debug!("left_btn");
    loop {
        btn.wait_for_rising_edge().await;
        debug!("left btn clicked");
        EVENT_BUS.sender().send(AppEvent::LeftBtnClicked).await;
    }
}

#[embassy_executor::task]
async fn orchestration() {
    debug!("orchestration");
    let receiver = EVENT_BUS.receiver();
    loop {
        let event = receiver.receive().await;
        {
            let mut app = APP.lock().await;
            match event {
                AppEvent::AqiBtnClicked => {
                    if app.is_ready() {
                        TAKE_ENV_READING.signal(());
                    }
                }
                AppEvent::LeftBtnClicked => {
                    if app.is_ready() {
                        VIEW_CMD.signal(ViewCmd::Prev);
                    }
                }
                AppEvent::Pmsa003iReady => {
                    app.ready();
                    TAKE_ENV_READING.signal(()); // automatically take first reading
                }
                AppEvent::Pmsa003iReadingTaken(reading) => {
                    VIEW_CMD.signal(ViewCmd::Refresh(reading.into()));
                }
                AppEvent::RightBtnClicked => {
                    if app.is_ready() {
                        VIEW_CMD.signal(ViewCmd::Next);
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn right_btn(mut btn: Input<'static>) {
    debug!("right_btn");
    loop {
        btn.wait_for_rising_edge().await;
        debug!("right btn clicked");
        EVENT_BUS.sender().send(AppEvent::RightBtnClicked).await;
    }
}