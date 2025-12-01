use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_8X13_BOLD},
    prelude::*,
    primitives::PrimitiveStyle,
    text::{Alignment, Text},
};

use crate::{calendar::moon, rtclock::RealTimeClock, theme::Theme};

pub fn draw_frame<Color: PixelColor, Error>(
    draw_target: &mut impl DrawTarget<Color = Color, Error = Error>,
    theme: &impl Theme<Color = Color>,
    clock: &impl RealTimeClock,
) -> Result<(), Error> {
    draw_target
        .bounding_box()
        .into_styled(PrimitiveStyle::with_fill(theme.background()))
        .draw(draw_target)?;

    let moon_phase = moon::get_phase(clock.get_time());
    let moon_phase_label = moon::get_phase_label(moon_phase);
    let moon_illumination = moon::get_illumination(moon_phase);

    let mut buf = [0u8; 64];
    let text = format_no_std::show(
        &mut buf,
        format_args!(
            "Phase {:02.0}%\nIllum {:02.0}%\n{}",
            moon_phase * 100.0,
            moon_illumination * 100.0,
            moon_phase_label
        ),
    )
    .unwrap();

    Text::with_alignment(
        text,
        draw_target.bounding_box().center(),
        MonoTextStyle::new(&FONT_8X13_BOLD, theme.text()),
        Alignment::Center,
    )
    .draw(draw_target)?;

    Ok(())
}
