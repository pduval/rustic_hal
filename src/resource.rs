use std::collections::btree_map::Entry;
use std::collections::*;
use std::fmt::{Error as FmtError, Formatter};
use std::ops::Deref;
use std::vec::*;

use serde::de;
use serde::de::Error;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
//use serde::de::Deserialize;
use super::link::HalLink;
use super::{HalError, HalResult};
use serde_json::{from_value, to_value, Map, Value as JsonValue};

/// A Simple wrapper around a vector to allow custom
/// serialization when only 1 element is contained.
///
/// # example
///
/// In the example below, the vector serializes to json as an object if it contains
/// only one value, but as an array if more than one.
///
/// ```rust
/// # extern crate serde_json;
/// # extern crate rustic_hal;
/// use rustic_hal::resource::OneOrMany;
/// use rustic_hal::HalLink;
///
/// use serde_json::to_string;
/// # fn main() {
/// let mut v = OneOrMany::new();
/// v.push(&HalLink::new("http://test.com"));
///
/// assert_eq!(to_string(&v).unwrap(), r#"{"href":"http://test.com"}"#);
///
/// v.push(&HalLink::new("http://test2.com"));
///
/// assert_eq!(to_string(&v).unwrap(), r#"[{"href":"http://test.com"},{"href":"http://test2.com"}]"#);
/// # }
/// ```
#[derive(Clone)]
pub struct OneOrMany<T> {
    force_many: bool,
    content: Vec<Box<T>>,
}

impl<T> OneOrMany<T>
where
    T: Clone,
{
    /// create a new empty object
    pub fn new() -> OneOrMany<T> {
        OneOrMany {
            content: Vec::new(),
            force_many: false,
        }
    }

    /// Force to be serialized as array, even if only one element
    pub fn force_many(mut self) -> Self {
        self.force_many = true;
        self
    }

    /// retrieve the length of the wrapped vector
    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Retrieves a single element if possible.
    ///
    pub fn single(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(self.content[0].deref())
        }
    }

    /// Returns an immutable reference to the
    /// contained links
    pub fn many(&self) -> &Vec<Box<T>> {
        &self.content
    }

    /// Add an element to the wrapped vector.
    pub fn push(&mut self, newval: &T) {
        self.content.push(Box::new(newval.clone()));
    }
}

impl<T> Serialize for OneOrMany<T>
where
    T: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.is_empty() {
            ().serialize(serializer)
        } else if self.len() == 1 && !self.force_many {
            self.single().serialize(serializer)
        } else {
            self.content.serialize(serializer)
        }
    }
}

impl<'de, T> Deserialize<'de> for OneOrMany<T>
where
    for<'d> T: Deserialize<'d> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: JsonValue = try!(Deserialize::deserialize(deserializer));
        let v2 = value.clone();
        match v2 {
            JsonValue::Object(_) => {
                let obj: T = match from_value(value) {
                    Ok(v) => from_value(v).unwrap(),
                    Err(e) => return Err(D::Error::custom(format!("JSON Error: {:?}", e))),
                };
                let mut res = OneOrMany::new();
                res.push(&obj);
                Ok(res)
            }
            JsonValue::Array(_) => {
                let obj: Vec<Box<T>> = match from_value(value) {
                    Ok(v) => from_value(v).unwrap(),
                    Err(e) => return Err(D::Error::custom(format!("JSON Error: {:?}", e))),
                };
                let mut res = OneOrMany::new();
                res.content = obj;
                Ok(res)
            }
            _ => Ok(OneOrMany::new()),
        }
    }
}

/// The HAL Resource structure.
///
///
#[derive(Clone)]
pub struct HalResource {
    /// Map of links to related resources.
    links: BTreeMap<String, OneOrMany<HalLink>>,
    /// Map of set of embedded resources.
    embedded: BTreeMap<String, OneOrMany<HalResource>>,
    /// Documentations Curies
    curies: BTreeMap<String, HalLink>,
    /// The actual resource data
    data: JsonValue,
}

impl HalResource {
    pub fn new<T>(payload: T) -> HalResource
    where
        T: Serialize,
    {
        HalResource {
            links: BTreeMap::new(),
            embedded: BTreeMap::new(),
            curies: BTreeMap::new(),
            data: to_value(payload).unwrap(),
        }
    }

    pub fn with_link<S, L>(&mut self, name: S, link: L) -> &mut Self
    where
        S: Into<String>,
        L: Into<HalLink>,
    {
        let lk_name = name.into();
        match self.links.entry(lk_name.clone()) {
            Entry::Vacant(entry) => {
                let mut lk = OneOrMany::new();

                let mut lk = match lk_name.as_ref() {
                    "curies" => lk.force_many(),
                    _ => lk,
                };

                lk.push(&(link.into()));
                entry.insert(lk);
            }
            Entry::Occupied(mut entry) => {
                let mut content = entry.get_mut(); //&mut HalLinks
                content.push(&(link.into()));
            }
        }
        self
    }

    /// Retrieve one named link if found. Returns the first one if more than one.
    pub fn get_link(&self, name: &str) -> Option<&HalLink> {
        match self.links.get(name) {
            Some(link) => link.single(),
            None => None,
        }
    }

    /// Retrieve the self link
    pub fn get_self(&self) -> Option<&HalLink> {
        self.get_link("self")
    }

    /// Retrieve the list of links for a key
    pub fn get_links(&self, name: &str) -> Option<&Vec<Box<HalLink>>> {
        match self.links.get(name) {
            Some(link) => Some(link.many()),
            None => None,
        }
    }

    pub fn with_resource(&mut self, name: &str, resource: &HalResource) -> &mut Self {
        match self.embedded.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let mut resources = OneOrMany::new();
                resources.push(&resource.clone());
                entry.insert(resources);
            }
            Entry::Occupied(mut entry) => {
                let mut content = entry.get_mut(); //&mut HalLinks
                content.push(&resource.clone());
            }
        }
        self
    }

    pub fn with_resources(&mut self, name: &str, resources: Vec<&HalResource>) -> &mut Self {
        match self.embedded.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let mut _resources = OneOrMany::new().force_many();

                for resource in resources.iter() {
                    _resources.push(resource.clone())
                }
                entry.insert(_resources.clone());
            }
            Entry::Occupied(mut entry) => {
                let mut content = entry.get_mut(); //&mut HalLinks

                for resource in resources.iter() {
                    content.push(&resource.clone());
                }
            }
        }
        self
    }

    pub fn with_curie(&mut self, name: &str, href: &str) -> &mut Self {
        self.with_link("curies", HalLink::new(href).templated(true).with_name(name));
        self
    }

    pub fn with_extra_data<V>(&mut self, name: &str, value: V) -> &mut Self
    where
        V: Serialize,
    {
        match self.data {
            JsonValue::Object(ref mut m) => {
                m.insert(name.to_string(), to_value(value).unwrap());
            }
            _ => {
                let mut data = Map::<String, JsonValue>::new();
                data.insert(name.to_string(), to_value(value).unwrap());
                self.data = JsonValue::Object(data);
            }
        };
        self
    }

    pub fn get_extra_data<V>(&self, name: &str) -> HalResult<V>
    where
        for<'de> V: Deserialize<'de>,
    {
        let data = match self.data {
            JsonValue::Object(ref m) => m,
            _ => return Err(HalError::Custom("Invalid payload".to_string())),
        };
        match data.get(name) {
            Some(v) => from_value::<V>(v.clone()).or_else(|e| Err(HalError::Json(e))),
            None => Err(HalError::Custom(format!("Key {} missing in payload", name))),
        }
    }

    pub fn get_data<V>(&self) -> HalResult<V>
    where
        for<'de> V: Deserialize<'de>,
    {
        from_value::<V>(self.data.clone()).or_else(|e| Err(HalError::Json(e)))
    }
}

impl Serialize for HalResource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = match self.data {
            JsonValue::Object(ref m) => Some(m),
            _ => None,
        };
        let mut length = match map {
            Some(m) => m.len(),
            _ => 0,
        };
        length += self.links.len();
        length += self.embedded.len();

        let mut state = try!(serializer.serialize_map(Some(length)));
        if !self.links.is_empty() {
            try!(state.serialize_key("_links"));
            try!(state.serialize_value(&(self.links)));
        }
        if !self.embedded.is_empty() {
            try!(state.serialize_key("_embedded"));
            try!(state.serialize_value(&(self.embedded)));
        }
        if let Some(map) = map {
            for (k, v) in map {
                try!(state.serialize_key(k));
                try!(state.serialize_value(v));
            }
        };

        state.end()
    }
}

impl<'de> Deserialize<'de> for HalResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResourceVisitor {}

        impl ResourceVisitor {
            fn new() -> Self {
                ResourceVisitor {}
            }
        }

        /// A visitor for deserializing `HalResource` from map representation.
        ///
        /// Implementation deserialize the hal specific keys (_links, _embedded) to maps
        /// as usual, and stores everything else in  a `JsonValue` object.
        /// It then converts the `JsonValue` to the type T
        impl<'de> de::Visitor<'de> for ResourceVisitor {
            type Value = HalResource;
            fn expecting(&self, f: &mut Formatter) -> Result<(), FmtError> {
                Ok(())
            }

            fn visit_map<M>(self, mut visitor: M) -> Result<HalResource, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                //transitory Value to read the data.
                let mut payload: Map<String, JsonValue> = Map::new();
                // Dummy resource to collect the links and embedded resources
                let mut resource: HalResource = HalResource::new(());
                while let Some(key) = try!(visitor.next_key::<String>()) {
                    match key.deref() {
                        "_links" => {
                            resource.links = try!(visitor.next_value());
                        }
                        "_embedded" => {
                            resource.embedded = try!(visitor.next_value());
                        }
                        _ => {
                            payload.insert(key, try!(visitor.next_value()));
                        }
                    }
                }
                //try!(visitor.end());
                resource.data = JsonValue::Object(payload);
                Ok(resource)
            }
        }

        deserializer.deserialize_map(ResourceVisitor::new())
    }
}

impl PartialEq for HalResource {
    fn eq(&self, other: &HalResource) -> bool {
        self.get_self() == other.get_self()
    }
}
