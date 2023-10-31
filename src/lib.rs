//! # `toml-cfg`
//!
//! ## Summary
//!
//! * Crates can declare variables that can be overridden
//!     * Anything const, e.g. usize, strings, etc.
//! * (Only) The "root crate" can override these variables by including a `cfg.toml` file
//!
//! ## Config file
//!
//! This is defined ONLY in the final application or "root crate"
//!
//! ```toml
//! # a toml-cfg file
//!
//! [lib-one]
//! buffer_size = 4096
//!
//! [lib-two]
//! greeting = "Guten tag!"
//! ```
//!
//! ## In the library
//!
//! ```rust
//! // lib-one
//! #[toml_cfg::toml_config]
//! pub struct Config {
//!     #[default(32)]
//!     buffer_size: usize,
//! }
//!
//! // lib-two
//! #[toml_cfg::toml_config]
//! pub struct Config {
//!     #[default("hello")]
//!     greeting: &'static str,
//! }
//!
//! ```
//!
//! ## Configuration
//!
//! With the `TOML_CFG` environment variable is set with a value containing
//! "require_cfg_present", the `toml-cfg` proc macro will panic if no valid config
//! file is found. This is indicative of either no `cfg.toml` file existing in the
//! "root project" path, or a failure to find the correct "root project" path.
//!
//! This failure could occur when NOT building with a typical `cargo build`
//! environment, including with `rust-analyzer`. This is *mostly* okay, as
//! it doesn't seem that Rust Analyzer presents this in some misleading way.
//!
//! If you *do* find a case where this occurs, please open an issue!
//!
//! ## Look at what we get!
//!
//! ```shell
//! # Print the "buffer_size" value from the `lib-one` crate.
//! # Since it has no cfg.toml, we just get the default value.
//! $ cd pkg-example/lib-one
//! $ cargo run
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.01s
//!      Running `target/debug/lib-one`
//! 32
//!
//! # Print the "greeting" value from the `lib-two` crate.
//! # Since it has no cfg.toml, we just get the default value.
//! $ cd ../lib-two
//! $ cargo run
//!    Compiling lib-two v0.1.0 (/home/james/personal/toml-cfg/pkg-example/lib-two)
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.32s
//!      Running `target/debug/lib-two`
//! hello
//!
//! # Print the "buffer_size" value from `lib-one`, and "greeting"
//! # from `lib-two`. Since we HAVE defined a `cfg.toml` file, the
//! # values defined there are used instead.
//! $ cd ../application
//! $ cargo run
//!    Compiling lib-two v0.1.0 (/home/james/personal/toml-cfg/pkg-example/lib-two)
//!    Compiling application v0.1.0 (/home/james/personal/toml-cfg/pkg-example/application)
//!     Finished dev [unoptimized + debuginfo] target(s) in 0.30s
//!      Running `target/debug/application`
//! 4096
//! Guten tag!
//! ```
//!

use heck::ToShoutySnekCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Debug)]
struct Config {
    #[serde(flatten)]
    crates: HashMap<String, Defn>,
}

#[derive(Deserialize, Clone, Debug, Default)]
struct Defn {
    #[serde(flatten)]
    vals: HashMap<String, toml::Value>,
}

#[proc_macro_attribute]
pub fn toml_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_defn =
        syn::parse::<syn::ItemStruct>(item).expect("Failed to parse configuration structure!");

    let require_cfg_present = if let Ok(val) = env::var("TOML_CFG") {
        val.contains("require_cfg_present")
    } else {
        false
    };

    let root_path = find_root_path();
    let cfg_path = root_path.clone();
    let cfg_path = cfg_path.as_ref().and_then(|c| {
        let mut x = c.to_owned();
        x.push("cfg.toml");
        Some(x)
    });

    let maybe_cfg = cfg_path.as_ref().and_then(|c| load_crate_cfg(&c));
    let got_cfg = maybe_cfg.is_some();
    if require_cfg_present {
        assert!(
            got_cfg,
            "TOML_CFG=require_cfg_present set, but valid config not found!"
        )
    }
    let cfg = maybe_cfg.unwrap_or_else(|| Defn::default());

    let mut struct_defn_fields = TokenStream2::new();
    let mut struct_inst_fields = TokenStream2::new();

    for field in struct_defn.fields {
        let ident = field
            .ident
            .clone()
            .expect("Failed to find field identifier. Don't use this on a tuple struct.");

        // Determine the default value, declared using the `#[default(...)]` syntax
        let default = field
            .attrs
            .iter()
            .find(|a| a.path.get_ident() == Some(&Ident::new("default", Span::call_site())))
            .expect(&format!(
                "Failed to find `#[default(...)]` attribute for field `{}`.",
                ident.to_string(),
            ))
            .clone();

        let ty = field.ty;

        // Is this field overridden?
        let val = match cfg.vals.get(&ident.to_string()) {
            Some(t) => {
                let t_string = t.to_string();
                let value: TokenStream2 = t_string.parse().expect(&format!(
                    "Failed to parse `{}` as a valid token!",
                    &t_string
                ));

                let default_value = default.tokens.to_string();

                let is_enum = default_value.contains("::")
                    && default_value
                        .starts_with(&format!("({} ::", ty.to_token_stream().to_string()));

                if is_enum {
                    let value_string = format_ident!(
                        "{}",
                        t.as_str().expect(&format!(
                            "Failed to parse `{}` as a valid string!",
                            &t_string
                        ))
                    );
                    quote! { #ty::#value_string }
                } else {
                    quote! { #value }
                }
            }
            None => {
                let default = &default.tokens;
                quote! { #default }
            }
        };

        quote! {
            pub #ident: #ty,
        }
        .to_tokens(&mut struct_defn_fields);

        quote! {
            #ident: #val,
        }
        .to_tokens(&mut struct_inst_fields);
    }

    let struct_ident = struct_defn.ident;
    let shouty_snek: TokenStream2 = struct_ident
        .to_string()
        .TO_SHOUTY_SNEK_CASE()
        .parse()
        .expect("NO NOT THE SHOUTY SNAKE");

    let hack_retrigger = if let Some(cfg_path) = cfg_path {
        let cfg_path = format!("{}", cfg_path.display());
        quote! {
            const _: &[u8] = include_bytes!(#cfg_path);
        }
    } else {
        quote! {}
    };

    quote! {
        pub struct #struct_ident {
            #struct_defn_fields
        }

        pub const #shouty_snek: #struct_ident = #struct_ident {
            #struct_inst_fields
        };

        mod toml_cfg_hack {
            #hack_retrigger
        }
    }
    .into()
}

fn load_crate_cfg(path: &Path) -> Option<Defn> {
    let contents = std::fs::read_to_string(&path).ok()?;
    let parsed = toml::from_str::<Config>(&contents).ok()?;
    let name = env::var("CARGO_PKG_NAME").ok()?;
    parsed.crates.get(&name).cloned()
}

// From https://stackoverflow.com/q/60264534
fn find_root_path() -> Option<PathBuf> {
    // First we get the arguments for the rustc invocation
    let mut args = std::env::args();

    // Then we loop through them all, and find the value of "out-dir"
    let mut out_dir = None;
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = args.next();
        }
    }

    // Finally we clean out_dir by removing all trailing directories, until it ends with target
    let mut out_dir = PathBuf::from(out_dir?);
    while !out_dir.ends_with("target") {
        if !out_dir.pop() {
            // We ran out of directories...
            return None;
        }
    }

    out_dir.pop();

    Some(out_dir)
}
