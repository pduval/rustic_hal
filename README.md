# Rustic Hal

A simple library for serializing (and deserializing coming soon) resources following the [HAL Spec](https://tools.ietf.org/html/draft-kelly-json-hal-08)

[![Build Status](https://travis-ci.org/pduval/rustic_hal.svg)](https://travis-ci.org/pduval/rustic_hal)
[![](http://meritbadge.herokuapp.com/rustic_hal)](https://crates.io/crates/rustic_hal)

## Usage

Add the dependency to your Cargo.toml:

```toml

[dependencies]
rustic_hal="0.2"
serde="1.0"
serde_json="1.0"
serde_derive="1.0"

```
and to use:

```rust

extern crate rustic_hal;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rustic_hal::*;
use serde_json::to_string;

#[derive(Serialize)]
pub struct MyResource {
    pub test: String,
}

fn main() {
    let mr = MyResource {
        test: "Hello, World!".to_string(),
    };
    let hal_res = HalResource::new(mr).with_link("self", "/api/myresource/0");
    println!("json representation: {:?}", to_string(&hal_res));
}

```
## Documentation

see [https://pduval.github.io/rustic_hal/rustic_hal/](https://pduval.github.io/rustic_hal/rustic_hal/) for the cargo-doc pages.

## Credits

This library is heavily inspired by the [hal-rs](https://github.com/hjr3/hal-rs) library by Herman J. Radtke III.

