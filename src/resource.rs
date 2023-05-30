use std::collections::btree_map::Entry;
use std::collections::*;
use std::vec::*;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
#[derive(Clone, Debug)]
pub struct OneOrMany<T> {
    force_many: bool,
    content: Vec<T>,
}

impl<T: Sized + Clone> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> OneOrMany<T>
where
    T: Sized + Clone,
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
            Some(&self.content[0])
        }
    }

    /// Returns an immutable reference to the
    /// contained links
    pub fn many(&self) -> &Vec<T> {
        &self.content
    }

    /// Add an element to the wrapped vector.
    pub fn push(&mut self, newval: &T) {
        self.content.push(newval.clone());
    }

    /// Adds an element to the vector in a chainable way
    pub fn with(mut self, newval: &T) -> Self {
        self.content.push(newval.clone());
        self
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
        if self.is_empty() && !self.force_many {
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


        let value: JsonValue = Deserialize::deserialize(deserializer)?;
        let v2 = value.clone();
        match v2 {
            JsonValue::Object(_) => {
                let obj: T = match from_value(value) {
                    Ok(v) => v,
                    Err(e) => return Err(D::Error::custom(format!("JSON Error: {:?}", e))),
                };
                let mut res = OneOrMany::new();
                res.push(&obj);
                Ok(res)
            }
            JsonValue::Array(_) => {
                let obj: Vec<T> = match from_value(value) {
                    Ok(v) => from_value(v).unwrap(),
                    Err(e) => return Err(D::Error::custom(format!("JSON Error: {:?}", e))),
                };
                let mut res = OneOrMany::new();
                res.content = obj;
                Ok(res)
            }
            _ => {
                let obj: T = match from_value(value) {
                    Ok(v) => v,
                    Err(e) => return Err(D::Error::custom(format!("JSON Error: {:?}", e)))
                };
                let mut res = OneOrMany::new();
                res.push(&obj);
                Ok(res)
            }
        }
    }
}

/// The HAL Resource structure.

#[derive(Clone, Serialize, Deserialize)]
pub struct HalResource {
    #[serde(rename = "_links", default, skip_serializing_if = "BTreeMap::is_empty")]
    /// Map of links to related resources.
    links: BTreeMap<String, OneOrMany<HalLink>>,

    #[serde(
        rename = "_embedded",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    /// Map of set of embedded resources.
    embedded: BTreeMap<String, OneOrMany<HalResource>>,

    #[serde(
        rename = "_curies",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    /// Documentations Curies
    curies: BTreeMap<String, HalLink>,

    #[serde(flatten)]
    /// The actual resource data
    data: Option<JsonValue>,
}

impl HalResource {
    pub fn new<T>(payload: T) -> HalResource
    where
        T: Serialize,
    {
        let val = match to_value(payload) {
            Ok(val) => match val {
                JsonValue::Object(_) => Some(val),
                _ => None,
            },
            _ => None,
        };

        HalResource {
            links: BTreeMap::new(),
            embedded: BTreeMap::new(),
            curies: BTreeMap::new(),
            data: val,
        }
    }

    pub fn with_link<S, L>(mut self, name: S, link: L) -> Self
    where
        S: Into<String>,
        L: Into<HalLink>,
    {
        let lk_name = name.into();
        match self.links.entry(lk_name.clone()) {
            Entry::Vacant(entry) => {
                let lk = OneOrMany::new();

                let mut lk = match lk_name.as_ref() {
                    "curies" => lk.force_many(),
                    _ => lk,
                };

                lk.push(&(link.into()));
                entry.insert(lk);
            }
            Entry::Occupied(mut entry) => {
                let content = entry.get_mut(); //&mut HalLinks
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
    pub fn get_links(&self, name: &str) -> Option<&Vec<HalLink>> {
        match self.links.get(name) {
            Some(link) => Some(link.many()),
            None => None,
        }
    }

    pub fn with_resource(mut self, name: &str, resource: HalResource) -> Self {
        match self.embedded.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let mut resources = OneOrMany::new();
                resources.push(&resource);
                entry.insert(resources);
            }
            Entry::Occupied(mut entry) => {
                let content = entry.get_mut(); //&mut HalLinks
                content.push(&resource);
            }
        }
        self
    }

    pub fn with_resources(mut self, name: &str, resources: Vec<HalResource>) -> Self {
        match self.embedded.entry(name.to_string()) {
            Entry::Vacant(entry) => {
                let mut _resources = OneOrMany::new().force_many();

                for resource in resources.iter() {
                    _resources.push(resource)
                }
                entry.insert(_resources);
            }
            Entry::Occupied(mut entry) => {
                let content = entry.get_mut(); //&mut HalLinks

                for resource in resources.iter() {
                    content.push(resource);
                }
            }
        }
        self
    }

    pub fn with_curie(self, name: &str, href: &str) -> Self {
        self.with_link("curies", HalLink::new(href).templated(true).with_name(name))
    }

    pub fn with_extra_data<V>(mut self, name: &str, value: V) -> Self
    where
        V: Serialize,
    {
        match self.data {
            Some(JsonValue::Object(ref mut m)) => {
                m.insert(name.to_string(), to_value(value).unwrap());
            }
            _ => {
                let mut data = Map::<String, JsonValue>::new();
                data.insert(name.to_string(), to_value(value).unwrap());
                self.data = Some(JsonValue::Object(data));
            }
        };
        self
    }

    pub fn get_extra_data<V>(&self, name: &str) -> HalResult<V>
    where
        for<'de> V: Deserialize<'de>,
    {
        let data = match self.data {
            Some(JsonValue::Object(ref m)) => m,
            _ => return Err(HalError::Custom("Invalid payload".to_string())),
        };
        match data.get(name) {
            Some(v) => from_value::<V>(v.clone()).map_err(HalError::Json),
            None => Err(HalError::Custom(format!("Key {} missing in payload", name))),
        }
    }

    pub fn get_data<V>(&self) -> HalResult<V>
    where
        for<'de> V: Deserialize<'de>,
    {
        match self.data {
            Some(ref val) => from_value::<V>(val.clone()).map_err(HalError::Json),
            None => Err(HalError::Custom("No value".to_owned())),
        }
    }
}

impl PartialEq for HalResource {
    fn eq(&self, other: &HalResource) -> bool {
        self.get_self() == other.get_self()
    }
}

#[cfg(feature = "actix-web")]
mod actix {

    use super::HalResource;
    use actix_web::{HttpRequest, HttpResponse, Responder};
    use actix_web::body::BoxBody;

    impl Responder for HalResource {
        type Body = BoxBody;
        //type Error = Error;
        //type Future = Ready<Result<HttpResponse, Error>>;

        fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
            let body = serde_json::to_string(&self).unwrap();

            // Create response and set content type
            HttpResponse::Ok()
                .content_type("application/hal+json; charset=utf-8")
                .body(body)
        }
    }

    impl From<HalResource> for HttpResponse {
        fn from(resource: HalResource) -> Self {
            if let Ok(res) = serde_json::to_string(&resource) {
                HttpResponse::Ok()
                    .content_type("application/hal+json; charset=utf-8")
                    .body(res)
            } else {
                HttpResponse::InternalServerError()
                    .content_type("application/hal+json; charset=utf-8")
                    .body("{\"error\":\"Internal Server Error\"}")
            }
        }
    }
}

#[cfg(feature = "actix-web")]
pub use self::actix::*;

#[cfg(feature = "axumweb")]
mod axum {
    use axum::{http::{header}, response::{Response, IntoResponse}, body};
    use super::HalResource;


    impl IntoResponse for HalResource {
        fn into_response(self) -> Response {
            if let Ok(res) = serde_json::to_string_pretty(&self) {
                let body = body::boxed(body::Full::from(res));
                Response::builder().status(200).header(header::CONTENT_TYPE, "application/hal+json; charset=urf-8").body(body).unwrap()
            } else {
                let body = body::boxed(body::Full::from(r#"{ "success": false, "error": "Invalid response" }"#.to_string()));
                Response::builder().status(500).header(header::CONTENT_TYPE, "application/hal+json; charset=urf-8").body(body).unwrap()
            }
        }
    }


}

#[cfg(feature = "warp-reply")]
mod warp {

    use super::HalResource;
    use warp::{Reply };
    use serde_json::to_string;
    use http::header::{HeaderValue, CONTENT_TYPE};
    use warp::http::{StatusCode };
    use warp::reply::Response;

    impl Reply for HalResource {
        fn into_response(self) -> Response {
            if let Ok(s) = to_string(&self) {
                let mut response = Response::new(s.into());
                response.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("text/json; charset=utf-8")
                );
                response
            } else {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
                // let mut response = Response::new("Error serializing resource".into());
                // response.headers_mut().insert(
                //     CONTENT_TYPE,
                //     HeaderValue::from_static("text/plain; charset=utf-8"),
                // );
                // response.set_status(StatusCode::InternalServerError);
                // response
            }
        }
    }

}

#[cfg(feature = "warp-reply")]
pub use self::warp::*;

