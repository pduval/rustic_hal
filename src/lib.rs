#![feature(proc_macro)]
extern crate serde_json;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod resource;
pub mod link;

pub use self::resource::HalResource;
pub use self::link::HalLink;

#[cfg(test)]
mod tests;
