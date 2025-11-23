#![no_std]
#![no_main]

mod jd79661;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::MODE_0;

#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;

// Alias for our HAL crate
use hal::entry;

#[cfg(rp2350)]
use rp235x_hal as hal;

#[cfg(rp2040)]
use rp2040_hal as hal;

use hal::Spi;
use hal::fugit::RateExtU32;
use hal::gpio::FunctionSpi;

use crate::jd79661::{HEIGHT, JD79661, WIDTH};

// use bsp::entry;
// use bsp::hal;
// use rp_pico as bsp;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[unsafe(link_section = ".boot2")]
#[used]
#[cfg(rp2040)]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
#[cfg(rp2350)]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp2040 and rp235x peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[entry]
fn main() -> ! {
    info!("Program start");
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    #[cfg(rp2040)]
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    #[cfg(rp2350)]
    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();
    // loop {
    //     led_pin.set_high().unwrap();
    //     timer.delay_ms(200);
    //     led_pin.set_low().unwrap();
    //     timer.delay_ms(200);
    // }

    led_pin.set_high().unwrap();

    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();
    let spi = Spi::new(pac.SPI0, (mosi, sclk)).init(&mut pac.RESETS, 12u32.MHz(), 4.MHz(), MODE_0);

    let dc = pins.gpio6.into_push_pull_output();
    let rst = pins.gpio7.into_push_pull_output();
    let cs = pins.gpio8.into_push_pull_output();
    let busy = pins.gpio9.into_pull_down_input();

    let mut screen = JD79661::begin(
        spi,
        dc.into_dyn_pin(),
        rst.into_dyn_pin(),
        cs.into_dyn_pin(),
        busy.into_dyn_pin(),
    );

    screen.power_up(&mut timer);

    let mut buffer: [u8; _] = [0; 8000];
    for x in 0..WIDTH / 4 {
        for y in 0..HEIGHT {
            let i = y * WIDTH / 4 + x;
            let mut data = 0;
            if x < WIDTH / 8 {
                data ^= 0b01010101;
            }
            if y < HEIGHT / 2 {
                data ^= 0b10101010;
            }
            buffer[i] = data;
        }
    }

    loop {
        led_pin.set_low().unwrap();

        screen.write_buffer(&buffer);
        screen.update(&mut timer);
        timer.delay_ms(1000);

        led_pin.set_high().unwrap();
        screen.write_buffer(&[0b10101010; 8000]);
        screen.update(&mut timer);
        timer.delay_ms(1000);
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"JD79661 Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

// End of file
