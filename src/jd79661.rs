use cortex_m::prelude::_embedded_hal_blocking_spi_Write;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

#[cfg(rp2350)]
use rp235x_hal as hal;

#[cfg(rp2040)]
use rp2040_hal as hal;

use hal::gpio;
use hal::gpio::DynPinId;
use hal::gpio::Pin;
use hal::spi;

type CommandData<'a> = (u8, &'a [u8]);

enum Command<'a> {
    Misc(u8, &'a [u8]),
    PSR(&'a [u8; 2]),
    PWR(&'a [u8; 6]),
    POF,
    PON,
    BTST(&'a [u8; 7]),
    DSLP,
    DTM(&'a [u8]),
    DSP, // XXX there is supposed to be an output parameter to read
    DRF(&'a [u8; 1]),
    AUTO(&'a [u8; 1]),
    PLL(&'a [u8; 1]),
    CDI(&'a [u8; 1]),
    TRES(&'a [u8; 4]),
}

impl<'a> From<&'a Command<'a>> for CommandData<'a> {
    fn from(value: &'a Command) -> Self {
        use Command::*;
        match value {
            Misc(c, d) => (*c, d),
            PSR(d) => (0x00, d.as_slice()),
            PWR(d) => (0x01, d.as_slice()),
            POF => (0x02, &[0x00]),
            PON => (0x04, &[]),
            BTST(d) => (0x06, d.as_slice()),
            DSLP => (0x07, &[0xA5]),
            DTM(d) => (0x10, d),
            DSP => (0x11, &[]),
            DRF(d) => (0x12, d.as_slice()),
            AUTO(d) => (0x17, d.as_slice()),
            PLL(d) => (0x30, d.as_slice()),
            CDI(d) => (0x50, d.as_slice()),
            TRES(d) => (0x61, d.as_slice()),
        }
    }
}

// rp2040_hal likes to use infallible results and I don't really want to
// propagate that across the entire program. So add a trait to unwrap such
// results without accidentally unwrapping anything else.
trait UnwrapInf {
    type Output;

    fn unwrap_inf(self) -> Self::Output;
}

impl<T> UnwrapInf for Result<T, core::convert::Infallible> {
    type Output = T;

    fn unwrap_inf(self) -> Self::Output {
        self.unwrap()
    }
}

const START_SEQUENCE: &[Command] = &[
    Command::Misc(0x4D, &[0x78]),
    Command::PSR(&[0x8F, 0x29]), // PSR, Display resolution is 128x250; scan up first line G1->G2, shift right first data S1->S2
    Command::PWR(&[0x07, 0x00, 0, 0, 0, 0]), // PWR
    Command::Misc(0x03, &[0x10, 0x54, 0x44]), // POFS
    Command::BTST(&[0x05, 0x00, 0x3F, 0x0A, 0x25, 0x12, 0x1A]),
    Command::CDI(&[0x37]),              // CDI
    Command::Misc(0x60, &[0x02, 0x02]), // TCON
    Command::TRES(&[0, 128, 0, 250]),   // TRES
    Command::Misc(0xE7, &[0x1C]),
    Command::Misc(0xE3, &[0x22]),
    Command::Misc(0xB4, &[0xD0]),
    Command::Misc(0xB5, &[0x03]),
    Command::Misc(0xE9, &[0x01]),
    Command::PLL(&[0x08]),
    Command::PON,
];

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 250;
pub const PIXDEPTH: usize = 2;

pub struct JD79661<D, P>
where
    D: spi::SpiDevice,
    P: spi::ValidSpiPinout<D>,
{
    spi: hal::Spi<spi::Enabled, D, P>,
    dc_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
    rst_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
    cs_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
    busy_pin: Pin<DynPinId, gpio::FunctionSioInput, gpio::PullDown>,
}

impl<D, P> JD79661<D, P>
where
    D: spi::SpiDevice,
    P: spi::ValidSpiPinout<D>,
{
    pub fn begin(
        spi: hal::Spi<spi::Enabled, D, P>,
        dc_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        rst_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        mut cs_pin: Pin<DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        busy_pin: Pin<DynPinId, gpio::FunctionSioInput, gpio::PullDown>,
    ) -> Self {
        cs_pin.set_high().unwrap_inf();

        Self {
            spi,
            dc_pin,
            rst_pin,
            cs_pin,
            busy_pin,
        }
    }

    pub fn hardware_reset(&mut self, timer: &mut impl DelayNs) {
        self.rst_pin.set_high().unwrap_inf();
        timer.delay_ms(20);
        self.rst_pin.set_low().unwrap_inf();
        timer.delay_ms(40);
        self.rst_pin.set_high().unwrap_inf();
        timer.delay_ms(50);
    }

    fn busy_wait(&mut self, timer: &mut impl DelayNs) {
        while self.busy_pin.is_low().unwrap_inf() {
            timer.delay_ms(10);
        }
    }

    pub fn power_up(&mut self, timer: &mut impl DelayNs) {
        self.hardware_reset(timer);
        self.busy_wait(timer);

        timer.delay_ms(10);
        self.command_list(START_SEQUENCE);
        self.busy_wait(timer);
    }

    pub fn power_down(&mut self, timer: &mut impl DelayNs) {
        self.command_list(&[Command::POF]);
        self.busy_wait(timer);
        self.command_list(&[Command::DSLP]);
        timer.delay_ms(100);
    }

    pub fn update(&mut self, timer: &mut impl DelayNs) {
        self.command_list(&[Command::DRF(&[0x00])]);
        self.busy_wait(timer);
    }

    pub fn sleeping_update(&mut self, timer: &mut impl DelayNs) {
        // PON -> DRF -> POF
        self.command_list(&[Command::AUTO(&[0xA5])]);
        self.busy_wait(timer);
    }

    fn command_list(&mut self, commands: &[Command]) {
        for command in commands {
            let (c, d) = CommandData::from(command);

            // XXX Some of this is probably unnecessary if this spi driver
            // manages the transaction differently.
            self.cs_pin.set_high().unwrap_inf();
            self.dc_pin.set_low().unwrap_inf();
            self.cs_pin.set_low().unwrap_inf();

            self.spi.write(&[c]).unwrap_inf();

            self.dc_pin.set_high().unwrap_inf();
            self.spi.write(d).unwrap_inf();

            self.cs_pin.set_high().unwrap_inf();
        }
    }

    pub fn write_buffer(&mut self, buffer: &[u8; WIDTH * HEIGHT * PIXDEPTH / 8]) {
        self.command_list(&[Command::DTM(buffer.as_slice()), Command::DSP]);
    }
}
