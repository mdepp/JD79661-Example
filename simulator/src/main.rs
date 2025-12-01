use std::time::{SystemTime, UNIX_EPOCH};

use common::{
    logic::draw_frame,
    rtclock::{InstantSecs, RealTimeClock},
    theme::Theme,
};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

struct SimulatorTheme;

impl Theme for SimulatorTheme {
    type Color = BinaryColor;

    fn background(&self) -> Self::Color {
        BinaryColor::Off
    }

    fn text(&self) -> Self::Color {
        BinaryColor::On
    }
}

struct SimulatorClock;

impl RealTimeClock for SimulatorClock {
    fn get_time(&self) -> InstantSecs {
        let ticks = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        InstantSecs::from_ticks(ticks)
    }
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 250));
    let theme = SimulatorTheme {};
    let clock = SimulatorClock {};

    draw_frame(&mut display, &theme, &clock)?;

    let output_settings = OutputSettingsBuilder::new()
        .theme(embedded_graphics_simulator::BinaryColorTheme::OledWhite)
        .build();
    Window::new("Sundial", &output_settings).show_static(&display);

    Ok(())
}
