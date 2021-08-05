//! This is a platform agnostic Rust driver for the LTC6904 I2C programmable Oscillator
//! base on the [`embedded-hal`] traits.
//!
//! The LTC6903 uses SPI as interface which is not yet implemented, but should be straightforward
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to
//! - configure output to posive, negative, both or none of the edges
//! - set the generated frequency
//! - after setting up the configuration, [`write_out()`] needs to be called to write the configuration to the IC
//!
//! [`write_out()`]: (struct.LTC6904.html#method.write_out)
//!
//! The 'output enable' pin of the device needs to be pulled from the application independently of the driver

#![no_std]

use core::result::Result;
use embedded_hal::{self as hal, digital::v2::OutputPin};

use hal::blocking::i2c::{Read, Write, WriteRead};

pub struct LTC6904<I2C, PIN>
where
    I2C: Read + Write + WriteRead,
    PIN: OutputPin,
{
    i2c: I2C,
    reg: u16,
    addr: Address,
    frequ: u32,
    out_enable: PIN,
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum OutputSettings {
    ClkNeg = 0,
    ClkBoth = 1,
    ClkPos = 2,
    PowerDown = 3,
}

impl Into<u16> for OutputSettings {
    fn into(self) -> u16 {
        return self as u16;
    }
}

impl From<u16> for OutputSettings {
    fn from(x: u16) -> Self {
        match x {
            0 => OutputSettings::ClkNeg,
            1 => OutputSettings::ClkBoth,
            2 => OutputSettings::ClkPos,
            _ => OutputSettings::PowerDown,
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub enum Address {
    AddressHigh,
    AddressLow,
}

impl Into<u8> for Address {
    fn into(self) -> u8 {
        match self {
            Address::AddressLow => Self::ADDRESS_0,
            Address::AddressHigh => Self::ADDRESS_1,
        }
    }
}

impl Address {
    const ADDRESS_0: u8 = 0x17; // 7 bit address address pin low
    const ADDRESS_1: u8 = 0x16; // 7 bit address address pin high
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub enum FrequencyError {
    TooLow,
    TooHigh,
}

// currently needs the git version of defmt
// #[cfg(feature = "defmt")]
// impl<I2C> defmt::Format for LTC6904<I2C>
// where
//     I2C: Read + Write + WriteRead,
// {
//     fn format(&self, fmt: defmt::Formatter) {
//         defmt::write!(
//             fmt,
//             "LTC@{:x}: {} Hz -> reg={:x} => oct: {3=12..15:x}  dac: {3=2..11:x} cnf: {3=0..1:b}",
//             self.addr,
//             self.frequ,
//             self.reg,
//             self.reg,
//         );
//     }
// }

#[allow(dead_code)]
impl<I2C, E, PIN> LTC6904<I2C, PIN>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    PIN: OutputPin,
{
    const OCT: [(u32, u32); 16] = [
        /* 0 */ (1_039, 2_076),
        /* 1 */ (2_078, 4_152),
        /* 2 */ (4_156, 8_304),
        /* 3 */ (8_312, 16_610),
        /* 4 */ (16_620, 33_220),
        /* 5 */ (33_250, 66_430),
        /* 6 */ (66_500, 132_900),
        /* 7 */ (133_000, 265_700),
        /* 8 */ (266_000, 531_400),
        /* 9 */ (532_000, 1063_000),
        /* 10 */ (1_064_000, 2_126_000),
        /* 11 */ (2_128_000, 4_252_000),
        /* 12 */ (4_256_000, 8_503_000),
        /* 13 */ (8_511_000, 17_010_000),
        /* 14 */ (17_020_000, 34_010_000),
        /* 15 */ (34_050_000, 68_030_000),
    ];

    const OCT_POS: u16 = 12;
    const OCT_SIZE: u16 = 4;
    const DAC_POS: u16 = 2;
    const DAC_SIZE: u16 = 10;
    const CNF_POS: u16 = 0;
    const CNF_SIZE: u16 = 2;

    const OCT_MASK: u16 = 0b1111_0000_0000_0000;
    const DAC_MASK: u16 = 0b0000_1111_1111_1100;
    const CNF_MASK: u16 = 0b0000_0000_0000_0011;

    const FREQU_MIN: u32 = 1_039;
    const FREQU_MAX: u32 = 68_030_000;

    pub fn new(i2c: I2C, address: Address, out_enable: PIN) -> Self {
        Self {
            i2c,
            reg: 0,
            addr: address.into(),
            frequ: Self::FREQU_MIN,
            out_enable,
        }
    }

    pub fn enable_output(&mut self) -> Result<(), <PIN as OutputPin>::Error> {
        self.out_enable.set_high()
    }

    pub fn disable_output(&mut self) -> Result<(), <PIN as OutputPin>::Error> {
        self.out_enable.set_low()
    }

    fn set_oct(&mut self, oct: u16) {
        self.reg &= !Self::OCT_MASK;
        self.reg |= oct << 12;
    }

    pub fn get_oct(&self) -> u16 {
        (self.reg & Self::OCT_MASK) >> Self::OCT_POS
    }

    fn set_dac(&mut self, dac: u16) {
        self.reg &= !Self::DAC_MASK;
        self.reg |= dac << Self::DAC_POS;
    }

    pub fn get_dac(&self) -> u16 {
        self.reg & Self::DAC_MASK >> Self::DAC_POS
    }

    fn set_cnf(&mut self, cnf: u16) {
        self.reg &= !Self::CNF_MASK;
        self.reg |= cnf << Self::CNF_POS;
    }

    pub fn get_cnf(&self) -> u16 {
        self.reg & Self::CNF_MASK >> Self::CNF_POS
    }

    pub fn get_reg(&self) -> u16 {
        self.reg
    }

    fn update(&mut self) -> Result<(), E> {
        let mut buffer = [0; 2];
        self.i2c.read(self.addr.into(), &mut buffer)?;
        self.reg = u16::from_be_bytes(buffer);
        Ok(())
    }

    pub fn write_out(&mut self) -> Result<(), E> {
        let data = self.reg.to_be_bytes();
        Ok(self.i2c.write(self.addr.into(), &data)?)
    }

    pub fn set_output_conf(&mut self, output: OutputSettings) {
        self.set_cnf(output.into());
    }

    pub fn get_output_conf(&self) -> OutputSettings {
        self.get_cnf().into()
    }

    pub(crate) fn calc_oct(f: u32) -> Result<u16, FrequencyError> {
        if f < 1039 {
            return Err(FrequencyError::TooLow);
        } else if f > 68_030_000 {
            return Err(FrequencyError::TooHigh);
        } else {
            let mut result = 0;
            for (i, (min, max)) in Self::OCT.iter().enumerate() {
                if f >= *min && f <= *max {
                    result = i as u16;
                    break;
                }
            }
            Ok(result)
        }
    }

    pub(crate) fn calc_dac(f: u32, oct: u16) -> u16 {
        let dac = 2048u32 - (2078 * 2u32.pow(10u32 + oct as u32)) / f;
        dac as u16
    }

    pub fn set_frequency(&mut self, f: u32) -> Result<u16, FrequencyError> {
        let oct = Self::calc_oct(f)?;
        let dac = Self::calc_dac(f, oct);
        self.frequ = f;
        self.set_oct(oct as u16);
        self.set_dac(dac);
        Ok(self.reg)
    }

    pub fn get_frequency(&self) -> u32 {
        self.frequ
    }

    pub fn free(self) -> I2C {
        self.i2c
    }
}
