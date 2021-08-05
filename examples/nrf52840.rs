#![no_main]
#![no_std]

use core::sync::atomic::AtomicUsize;

use defmt_rtt as _;
use hal::{
    gpio::{self, Output, Pin, PushPull},
    prelude::{OutputPin, StatefulOutputPin},
    timer::Timer,
    twim::Pins,
};
use panic_probe as _;

extern crate ltc690x;
use ltc690x::{Address, OutputSettings, LTC6904};

use nrf52840_hal as hal;

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!(
    "{=usize}",
    COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
);

#[rtic::app(device = nrf52840_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        delay: hal::timer::Timer<hal::pac::TIMER0>,
        ltc: LTC6904<hal::twim::Twim<hal::pac::TWIM0>, hal::gpio::Pin<Output<PushPull>>>,
        led: Pin<Output<PushPull>>,
    }

    #[init()]
    fn init(ctx: init::Context) -> init::LateResources {
        defmt::info!("Booted Up!");
        let _clk = hal::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        defmt::info!("Clocks configured");
        let p0 = gpio::p0::Parts::new(ctx.device.P0);
        let p1 = gpio::p1::Parts::new(ctx.device.P1);

        let _led1 = p0.p0_13.into_push_pull_output(gpio::Level::Low);
        let led2 = p0.p0_14.into_push_pull_output(gpio::Level::High).degrade();

        let scl = p0.p0_28.into_floating_input().degrade();
        let sda = p0.p0_29.into_floating_input().degrade();

        let i2c = hal::twim::Twim::new(
            ctx.device.TWIM0,
            Pins { scl, sda },
            hal::twim::Frequency::K100,
        );
        defmt::debug!("i2c initialised");
        let delay = Timer::new(ctx.device.TIMER0);
        defmt::debug!("delay initialised");
        let ltc = ltc690x::LTC6904::new(
            i2c,
            Address::AddressLow,
            p1.p1_01.into_push_pull_output(gpio::Level::Low).degrade(),
        );
        defmt::debug!("ltc initialised");
        init::LateResources {
            delay,
            ltc,
            led: led2,
        }
    }

    #[idle(resources = [delay, ltc, led])]
    fn idle(ctx: idle::Context) -> ! {
        let delay: &mut hal::timer::Timer<hal::pac::TIMER0> = ctx.resources.delay;
        let ltc: &mut LTC6904<hal::twim::Twim<hal::pac::TWIM0>, Pin<Output<PushPull>>> =
            ctx.resources.ltc;
        let led: &mut Pin<Output<PushPull>> = ctx.resources.led;

        defmt::debug!("idle started");

        defmt::debug!(
            "Oct: {:?}, Dac: {:?}, Cnf: {:?}",
            ltc.get_oct(),
            ltc.get_dac(),
            ltc.get_cnf()
        );
        defmt::debug!("Reg: {:?}", ltc.get_reg());
        defmt::unwrap!(ltc.set_frequency(108_000));
        ltc.set_output_conf(OutputSettings::ClkBoth);

        defmt::debug!(
            "Oct: {:?}, Dac: {:?}, Cnf: {:?}",
            ltc.get_oct(),
            ltc.get_dac(),
            ltc.get_cnf()
        );

        defmt::debug!("Setup the LTC, now call write-out");

        let mut i = 0usize;
        let mut cnt = 0usize;
        loop {
            defmt::info!("{:?}->{:?}: idle loop", cnt, i);
            cnt += 1;
            delay.delay(500_000);

            match i {
                0 => {
                    ltc.set_output_conf(OutputSettings::ClkPos);
                }
                1 => {
                    ltc.set_output_conf(OutputSettings::ClkNeg);
                }
                2 => {
                    ltc.set_output_conf(OutputSettings::ClkBoth);
                }
                3 => {
                    ltc.set_output_conf(OutputSettings::PowerDown);

                    match ltc.get_frequency() {
                        108_000 => {
                            ltc.set_frequency(5000).ok().unwrap();
                        }
                        5_000 => {
                            ltc.set_frequency(1_000_000).ok().unwrap();
                        }
                        1_000_000 => {
                            ltc.set_frequency(108_000).ok().unwrap();
                        }
                        _ => {
                            ltc.set_frequency(108_000).ok().unwrap();
                        }
                    }
                    defmt::debug!(
                        "Oct: {:?}, Dac: {:?}, Cnf: {:?}",
                        ltc.get_oct(),
                        ltc.get_dac(),
                        ltc.get_cnf()
                    );
                }
                _ => {}
            }

            i = if i < 3 { i + 1 } else { 0 };

            match ltc.write_out() {
                Ok(_) => {
                    defmt::debug!("Write Out Ok")
                }
                Err(e) => {
                    defmt::error!("I2C Error: {:?}", defmt::Debug2Format(&e));
                }
            }

            if led.is_set_high().unwrap() {
                led.set_low().unwrap();
            } else {
                led.set_high().unwrap();
            }
        }
    }
};
