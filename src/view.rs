use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
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

const ROW_Y_OFFSET: i32 = 16;

pub enum ViewCmd {
    Refresh(EnvReading),
    Next,
    Prev
}

pub static VIEW_CMD: signal::Signal<CriticalSectionRawMutex, ViewCmd> = signal::Signal::new();

#[derive(Debug, Default)]
pub enum View {
    Aqi(EnvReading),
    Error,
    #[default]
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
        match VIEW_CMD.wait().await {
            ViewCmd::Refresh(reading) => manager.refresh_view(reading).await,
            ViewCmd::Next => manager.render_next_view().await,
            ViewCmd::Prev => manager.render_prev_view().await,
        }
    }
}

pub struct ViewManager {
    display: Display,
    text_style: DisplayTextStyle,
    view: Option<View>,
}
impl ViewManager {
    pub const fn new(display: Display, text_style: DisplayTextStyle) -> Self {
        Self { display, text_style, view: None }
    }

    async fn refresh_view(&mut self, reading: EnvReading) {
        info!("refresh_view");
        if let Some(view) = &self.view {
            match view {
                View::Aqi(_) => self.render_view(View::Aqi(reading)).await,
                View::Init => self.render_view(View::Pm(reading)).await,
                View::ParticleDiameter1(_) => self.render_view(View::ParticleDiameter1(reading)).await,
                View::ParticleDiameter2(_) => self.render_view(View::ParticleDiameter2(reading)).await,
                View::Pm(_) => self.render_view(View::Pm(reading)).await,
                View::PmEnv(_) => self.render_view(View::PmEnv(reading)).await,
                _ => {}
            }
        }
    }

    async fn render_next_view(&mut self) {
        debug!("render_next_view");
        if let Some(view) = &self.view {
            match view {
                View::Aqi(reading) => self.render_view(View::ParticleDiameter1(*reading)).await,
                View::ParticleDiameter1(reading) => self.render_view(View::ParticleDiameter2(*reading)).await,
                View::ParticleDiameter2(reading) => self.render_view(View::Pm(*reading)).await,
                View::Pm(reading) => self.render_view(View::PmEnv(*reading)).await,
                View::PmEnv(reading) => self.render_view(View::Aqi(*reading)).await,
                _ => {}
            }
        }
    }

    async fn render_prev_view(&mut self) {
        debug!("render_prev_view");
        if let Some(view) = &self.view {
            match view {
                View::Aqi(reading) => self.render_view(View::PmEnv(*reading)).await,
                View::ParticleDiameter1(reading) => self.render_view(View::Aqi(*reading)).await,
                View::ParticleDiameter2(reading) => self.render_view(View::ParticleDiameter1(*reading)).await,
                View::Pm(reading) => self.render_view(View::ParticleDiameter2(*reading)).await,
                View::PmEnv(reading) => self.render_view(View::Pm(*reading)).await,
                _ => {}
            }
        }
    }

    async fn render_view(&mut self, view: View) {
        debug!("render_view");
        self.view = Some(view);
        self.display.clear_buffer();
        if let Some(view) = &self.view {
            match view {
                View::Aqi(reading) => self.render_aqi(*reading),
                View::Error => self.render_error(),
                View::Init => self.render_init(),
                View::ParticleDiameter1(reading) => self.render_particle_diameter_1(*reading),
                View::ParticleDiameter2(reading) => self.render_particle_diameter_2(*reading),
                View::Pm(reading) => self.render_pm(*reading),
                View::PmEnv(reading) => self.render_pm_env(*reading),
            }
        }
        self.display.flush().await.unwrap();
    }

    fn render_aqi(&mut self, reading: EnvReading) {
        self.render_title("Air Quality Index");
        self.render_row(&*reading.aqi_pm2_5_str(), ROW_Y_OFFSET);
        self.render_row(&*reading.aqi_pm10_str(), ROW_Y_OFFSET * 2);
    }

    fn render_init(&mut self) {
        self.render_title("Starting up...");
        self.render_row("(takes 30 seconds)", ROW_Y_OFFSET);
    }


    fn render_error(&mut self) {
        self.render_title("Error :(");
    }

    fn render_particle_diameter_1(&mut self, reading: EnvReading) {
        self.render_title("PM Diam / 0.1L Air");
        self.render_row(&*reading.p_gt_0_3_str(), ROW_Y_OFFSET);
        self.render_row(&*reading.p_gt_0_5_str(), ROW_Y_OFFSET * 2);
        self.render_row(&*reading.p_gt_1_str(), ROW_Y_OFFSET * 3);
    }

    fn render_particle_diameter_2(&mut self, reading: EnvReading) {
        self.render_title("PM Diam / 0.1L Air");
        self.render_row(&*reading.p_gt_2_5_str(), ROW_Y_OFFSET);
        self.render_row(&*reading.p_gt_5_str(), ROW_Y_OFFSET * 2);
        self.render_row(&*reading.p_gt_10_str(), ROW_Y_OFFSET * 3);
    }

    fn render_pm(&mut self, reading: EnvReading) {
        self.render_title("PM Concen");
        self.render_row(&*reading.pm1_str(), ROW_Y_OFFSET);
        self.render_row(&*reading.pm2_5_str(), ROW_Y_OFFSET * 2);
        self.render_row(&*reading.pm10_str(), ROW_Y_OFFSET * 3);
    }

    fn render_pm_env(&mut self, reading: EnvReading) {
        self.render_title("PM Concen Atmo Env");
        self.render_row(&*reading.pm1_env_str(), ROW_Y_OFFSET);
        self.render_row(&*reading.pm2_5_env_str(), ROW_Y_OFFSET * 2);
        self.render_row(&*reading.pm10_env_str(), ROW_Y_OFFSET * 3);
    }

    fn render_title(&mut self, text: &str) {
        Text::with_baseline(text, Point::new(0, 0), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }

    fn render_row(&mut self, text: &str, y_offset: i32) {
        Text::with_baseline(text, Point::new(0, y_offset), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }
}