#![no_std]

#[derive(Debug)]
pub enum Choice {
    One,
    Other,
    Third,
}

impl Choice {
    const fn from_int(n: usize) -> Choice {
        match n {
            0 => Choice::One,
            1 => Choice::Other,
            _ => Choice::Third,
        }
    }
}

#[derive(Debug)]
pub enum OtherChoice {
    Foo,
    Bar,
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default(32)]
    buffer_size: usize,

    #[default(Choice::A as usize)]
    choice: usize,

    #[default(OtherChoice::Foo)]
    other_choice: OtherChoice,
}

impl Config {
    pub const fn choice(&self) -> Choice {
        Choice::from_int(self.choice)
    }
}
