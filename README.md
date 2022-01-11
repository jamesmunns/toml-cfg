# `toml-cfg`

Rough ideas:

* Crates can declare variables that can be overridden
    * Anything const, e.g. usize, strings, etc.
* (Only) The "root crate" can override these variables by including a `cfg.toml` file

## Config file

```toml
# a toml-cfg file

[lib-one]
buffer_size = 4096

[lib-two]
greeting = "Guten tag!"
```

## In the library

```rust
// lib-one
#[toml_cfg::toml_config]
pub struct Config {
    #[default(32)]
    buffer_size: usize,
}

// lib-two
#[toml_cfg::toml_config]
pub struct Config {
    #[default("hello")]
    greeting: &'static str,
}

```

## Look at what we get!

```shell
# Print the "buffer_size" value from the `lib-one` crate.
# Since it has no cfg.toml, we just get the default value.
$ cd pkg-example/lib-one
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/lib-one`
32

# Print the "greeting" value from the `lib-two` crate.
# Since it has no cfg.toml, we just get the default value.
$ cd ../lib-two
$ cargo run
   Compiling lib-two v0.1.0 (/home/james/personal/toml-cfg/pkg-example/lib-two)
    Finished dev [unoptimized + debuginfo] target(s) in 0.32s
     Running `target/debug/lib-two`
hello

# Print the "buffer_size" value from `lib-one`, and "greeting"
# from `lib-two`. Since we HAVE defined a `cfg.toml` file, the
# values defined there are used instead.
$ cd ../application
$ cargo run
   Compiling lib-two v0.1.0 (/home/james/personal/toml-cfg/pkg-example/lib-two)
   Compiling application v0.1.0 (/home/james/personal/toml-cfg/pkg-example/application)
    Finished dev [unoptimized + debuginfo] target(s) in 0.30s
     Running `target/debug/application`
4096
Guten tag!
```
