//! # Rustic Hal
//!
//! A simple library for serializing (and deserializing coming soon) resources following the [HAL Spec](https://tools.ietf.org/html/draft-kelly-json-hal-08)
//!
//! ## Usage
//!
//! Add the dependency to your Cargo.toml:
//!
//! ```toml
//!
//! [dependencies]
//! rustic_hal="0.1.0"
//! serde="1.0"
//! serde_json="1.0"
//! serde_derive="1.0"
//!
//! ```
//! and to use:
//!
//! ```rust
//! extern crate rustic_hal;
//! extern crate serde_json;
//! extern crate serde;
//! #[macro_use] extern crate serde_derive;
//!
//! use rustic_hal::*;
//! use serde::Serialize;
//! use serde_json::to_string;
//!
//! #[derive(Serialize)]
//! pub struct MyResource {
//!     pub test: String
//! }
//!
//! # fn main() {
//! let mr = MyResource { test: "Hello, World!".to_string() };
//! let mut hal_res = HalResource::new(mr);
//! hal_res.with_link("self", "/api/myresource/0");
//!
//! println!("json representation: {}", to_string(&hal_res).unwrap());
//!
//! # }
//! ```
//!
//! ## Credits
//!
//! This library is heavily inspired by the [hal-rs](https://github.com/hjr3/hal-rs) library by Herman J. Radtke III.
//!

extern crate serde_json;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod resource;
pub mod link;
pub mod error;

pub use self::resource::HalResource;
pub use self::link::HalLink;
pub use self::error::{HalError, HalResult};

#[cfg(test)]
mod tests;
