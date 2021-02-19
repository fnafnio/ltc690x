# LTC6903/6904 embedded-hal I2C driver crate

![](https://img.shields.io/crates/v/ltc690x.svg)
![](https://docs.rs/ltc690x/badge.svg)

Rust HAL implementation (using I2C traits from embedded-hal) for Linear Technologies LTC6903/6904 programmable 1kHz to 68MHz oscillator.

[Datasheet](https://www.analog.com/en/products/ltc6903.html)

## Usage

Include [library](link) as a dependency in your Cargo.toml

```TOML
[dependencies.ltc690x]
version = "*"
```

And use embedded-hal implementations for I2C to connect 

```rust
        // create config with address pin low I2C address
        let ltc = ltc690x::LTC6904::new(i2c, Address::AddressLow);
        // configure output to use positive and negative edge
        ltc.set_output_conf(OutputSettings::ClkBoth);
        // set a frequency
        ltc.set_frequency(1_000_000).ok().unwrap();
        // write the current configuration
        ltc.write_out().unwrap();

```