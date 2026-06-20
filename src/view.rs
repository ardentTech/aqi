use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::select::{select3, Either3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal;
use embedded_graphics::{prelude::*};
use embedded_graphics::mono_font::iso_8859_1::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::{Baseline, Text};
use ssd1306::{I2CDisplayInterface, Ssd1306Async};
use ssd1306::prelude::*;
use crate::{Display, DisplayTextStyle, EnvReading, I2cAsyncMutex};

pub static REFRESH_VIEW: signal::Signal<CriticalSectionRawMutex, EnvReading> = signal::Signal::new();
pub static RENDER_VIEW: signal::Signal<CriticalSectionRawMutex, View> = signal::Signal::new();
pub static RENDER_NEXT_VIEW: signal::Signal<CriticalSectionRawMutex, ()> = signal::Signal::new();

pub enum View {
    Aqi(EnvReading),
    Error,
    Init,
    ParticleDiameter1(EnvReading),
    ParticleDiameter2(EnvReading),
    Pm(EnvReading),
    PmEnv(EnvReading),
}

#[embassy_executor::task]
pub async fn display_task(i2c_bus: &'static I2cAsyncMutex) {
    info!("display_task");
    let interface = I2CDisplayInterface::new(I2cDevice::new(i2c_bus));
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().await.expect("TODO: panic message");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X12)
        .text_color(BinaryColor::On)
        .build();

    let mut manager = ViewManager::new(display, text_style);
    manager.render_view(View::Init).await;

    loop {
        match select3(RENDER_VIEW.wait(), RENDER_NEXT_VIEW.wait(), REFRESH_VIEW.wait()).await {
            Either3::First(view) => manager.render_view(view).await,
            Either3::Second(_) => manager.render_next_view().await,
            Either3::Third(reading) => manager.refresh_view(reading).await,
        }
    }
}

pub struct ViewManager {
    display: Display,
    text_style: DisplayTextStyle,
    view: View,
}
impl ViewManager {
    pub const fn new(display: Display, text_style: DisplayTextStyle) -> Self {
        Self { display, text_style, view: View::Init } // TODO what should the default view be?
    }

    pub async fn refresh_view(&mut self, reading: EnvReading) {
        info!("refresh_view");
        match self.view {
            View::Aqi(_) => self.render_view(View::Aqi(reading)).await,
            View::Init => self.render_view(View::Pm(reading)).await,
            View::ParticleDiameter1(_) => self.render_view(View::ParticleDiameter1(reading)).await,
            View::ParticleDiameter2(_) => self.render_view(View::ParticleDiameter2(reading)).await,
            View::Pm(_) => self.render_view(View::Pm(reading)).await,
            View::PmEnv(_) => self.render_view(View::PmEnv(reading)).await,
            _ => {}
        }
    }

    pub async fn render_next_view(&mut self) {
        info!("render_next_view");
        match self.view {
            View::Aqi(reading) => self.render_view(View::ParticleDiameter1(reading)).await,
            View::ParticleDiameter1(reading) => self.render_view(View::ParticleDiameter2(reading)).await,
            View::ParticleDiameter2(reading) => self.render_view(View::Pm(reading)).await,
            View::Pm(reading) => self.render_view(View::PmEnv(reading)).await,
            View::PmEnv(reading) => self.render_view(View::Aqi(reading)).await,
            _ => {}
        }
    }

    pub async fn render_view(&mut self, view: View) {
        debug!("render_view");
        self.view = view;
        self.display.clear_buffer();
        match self.view {
            View::Aqi(reading) => self.render_aqi(reading),
            View::Error => self.render_error(),
            View::Init => self.render_init(),
            View::ParticleDiameter1(reading) => self.render_particle_diameter_1(reading),
            View::ParticleDiameter2(reading) => self.render_particle_diameter_2(reading),
            View::Pm(reading) => self.render_pm(reading),
            View::PmEnv(reading) => self.render_pm_env(reading),
        }
        self.display.flush().await.unwrap();
    }

    fn render_aqi(&mut self, reading: EnvReading) {
        Text::with_baseline("Air Quality Index", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.aqi_pm2_5_str(), Point::new(0, 16), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.aqi_pm10_str(), Point::new(0, 32), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_init(&mut self) {
        Text::with_baseline("Init", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_error(&mut self) {
        Text::with_baseline("Error", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_particle_diameter_1(&mut self, reading: EnvReading) {
        Text::with_baseline("Part Diam in 0.1L Air", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_0_3_str(), Point::new(0, 16), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_0_5_str(), Point::new(0, 32), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_1_str(), Point::new(0, 48), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_particle_diameter_2(&mut self, reading: EnvReading) {
        Text::with_baseline("Part Diam in 0.1L Air", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_2_5_str(), Point::new(0, 16), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_5_str(), Point::new(0, 32), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.p_gt_10_str(), Point::new(0, 48), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_pm(&mut self, reading: EnvReading) {
        Text::with_baseline("PM Con", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm1_str(), Point::new(0, 16), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm2_5_str(), Point::new(0, 32), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm10_str(), Point::new(0, 48), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_pm_env(&mut self, reading: EnvReading) {
        Text::with_baseline("PM Con Atmo Env", Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm1_env_str(), Point::new(0, 16), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm2_5_env_str(), Point::new(0, 32), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Text::with_baseline(&*reading.pm10_env_str(), Point::new(0, 48), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }
}