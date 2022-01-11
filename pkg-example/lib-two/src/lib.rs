#[toml_cfg::toml_config]
pub struct Config {
    #[default("hello")]
    greeting: &'static str,
}
