extern crate serde_json;
extern crate serde;

pub mod resource;
pub mod link;

pub use self::resource::HalResource;
pub use self::link::HalLink;

#[cfg(test)]
mod tests;
