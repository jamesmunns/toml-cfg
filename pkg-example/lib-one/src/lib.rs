#![no_std]

#[toml_cfg::toml_config]
pub struct Config {
    #[default(32)]
    buffer_size: usize,
}
