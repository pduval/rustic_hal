use std::collections::*;
use std::collections::btree_map::Entry;
use std::vec::*;
use std::ops::Deref;

use serde::{Serialize, Serializer};
//use serde::de::Deserialize;
use serde_json::{Value, to_value};
use super::link::{HalLink};

/// A Simple wrapper around a vector to allow custom
/// serialization when only 1 element is contained.
///
/// # example
///
/// In the example below, the vector serializes to json as an object if it contains
/// only one value, but as an array if more than one.
///
/// ```
/// extern crate serde_json;
/// extern crate rustic_hal;
///
/// use rustic_hal::resource::OneOrMany;
/// use rustic_hal::HalLink;
/// use serde_json::to_string;
///
/// let mut v = OneOrMany::new();
/// v.push(HalLink::new("http://test.com"));
///
/// assert_eq!(to_string(v), "{\"href\":\"http://test.com\"}");
///
/// v.push(HalLink::new("http://test2.com"));
///
/// assert_eq!(to_string(v), "[{\"href\":\"http://test.com\"},{\"href\":\"http://test2.com\"}]");
/// ```
pub struct OneOrMany<T> {
    content: Vec<Box<T>>
}

impl<T> OneOrMany<T> where T:Clone {

    /// create a new empty object
    pub fn new() -> OneOrMany<T> {
        OneOrMany { content: Vec::new() }
    }

    /// retrieve the length of the wrapped vector
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Retrieves a single element if possible.
    ///
    pub fn single(&self) -> Option<&T> {
        if self.len() <= 0 {
            None
        } else {
            Some(self.content[0].deref())
        }
    }

    /// Add an element to the wrapped vector.
    pub fn push(&mut self, newval: &T) {
        self.content.push(Box::new(newval.clone()));
    }
}

impl<T> Serialize for OneOrMany<T> where T:Serialize+Clone {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        if self.len() == 0 {
            ().serialize(serializer)
        } else if self.len() == 1 {
            self.single().serialize(serializer)
        } else {
            self.content.serialize(serializer)
        }
            
    }
}

/// The HAL Resource structure.
///
///
pub struct HalResource<T>
    where T: Serialize
{
    /// Map of links to related resources.
    links: BTreeMap<String, OneOrMany<HalLink>>,
    /// Map of set of embedded resources.
    embedded: BTreeMap<String, OneOrMany<HalResource<Value>>>,
    /// Documentations Curies
    curies: BTreeMap<String, HalLink>,
    /// The actual resource data
    data: Box<T>,
}


impl<T> HalResource<T>
    where T: Serialize {
    pub fn new(payload: T) -> HalResource<T> {
        HalResource {
            links: BTreeMap::new(),
            embedded: BTreeMap::new(),
            curies: BTreeMap::new(),
            data: Box::new(payload)
        }
    }

    /// Add a simple link to the links map
    pub fn with_link_uri(&mut self, name: &str, href: &str) -> &mut Self {
        let link = HalLink::new(href);
        self.with_link(name, &link)
    }

    pub fn with_link(&mut self, name: &str, link: &HalLink) -> &mut Self {
        match self.links.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let mut lk = OneOrMany::new();
                lk.push(link);
                entry.insert(lk);
            },
            Entry::Occupied(mut entry) => {
                let mut content = entry.get_mut(); //&mut HalLinks
                content.push(link);
            }
        }
        self
    }

    /// Retrieve one named link if found. Returns the first one if more than one.
    pub fn get_link(&self, name: &str) -> Option<&HalLink> {
        match self.links.get(name) {
            Some(link) => link.single(),
            None => None
        }
    }
}

impl<T> Serialize for HalResource<T>
    where T: Serialize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {

        //HACK: We convert to JSON map, and then merge the links/embedded maps
        let value = to_value(&(self.data));
        let map = match value {
            Value::Object(m) => Some(m),
            _ => None
        };
        let mut length = match &map {
            &Some(ref m) => m.len(),
            _ => 0
        };
        length += self.links.len();
        length += self.embedded.len();
        let mut state = try!(serializer.serialize_map(Some(length) ));
        if self.links.len() > 0 {
            try!(serializer.serialize_map_key(&mut state, "_links"));
            try!(serializer.serialize_map_value(&mut state, &(self.links)));
        }
        //TODO: Sort out nested resources
        //if self.embedded.len() > 0 {
        //    try!(serializer.serialize_map_key(&mut state, "_embedded"));
        //    try!(serializer.serialize_map_value(&mut state, &(self.embedded)));
        //}
        match map {
            Some(map) => {
                for (k,v) in map {
                    try!(serializer.serialize_map_key(&mut state, k));
                    try!(serializer.serialize_map_value(&mut state, v));
                }
            },
            _ => {}
        };

        serializer.serialize_map_end(state)
    }
}


