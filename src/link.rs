use serde::de;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::{From, Into};
use std::fmt::{Error, Formatter};
use std::ops::Deref;

/// A Link object for linking HAL Resources.
///
/// The link represents a related resource.
/// If follows [the HAL Draft Spec](https://tools.ietf.org/html/draft-kelly-json-hal-08#section-5)
///
/// # Examples
///
/// ```rust
/// use rustic_hal::HalLink;
///
/// let link = HalLink::new("http://sowewhere.com");
/// ```
#[derive(Clone, Debug)]
pub struct HalLink {
    /// The "href" property is REQUIRED.
    ///
    /// Its value is either a URI [RFC3986] or a URI Template [RFC6570].
    ///
    /// If the value is a URI Template then the Link Object SHOULD have a
    /// "templated" attribute whose value is true.
    pub href: String,

    /// The "templated" property is OPTIONAL.
    ///
    /// Its value is boolean and SHOULD be true when the Link Object's "href"
    /// property is a URI Template.
    ///
    /// Its value SHOULD be considered false if it is undefined or any other
    /// value than true.
    pub templated: bool,
    /// The "type" property is OPTIONAL.
    ///
    /// Its value is a string used as a hint to indicate the media type
    /// expected when dereferencing the target resource.
    pub media_type: Option<String>,
    /// The "deprecation" property is OPTIONAL.
    ///
    /// Its presence indicates that the link is to be deprecated (i.e.
    /// removed) at a future date.  Its value is a URL that SHOULD provide
    /// further information about the deprecation.
    ///
    /// A client SHOULD provide some notification (for example, by logging a
    /// warning message) whenever it traverses over a link that has this
    /// property.  The notification SHOULD include the deprecation property's
    /// value so that a client manitainer can easily find information about
    /// the deprecation.
    pub deprecation: Option<String>,

    /// The "name" property is OPTIONAL.
    ///
    /// Its value MAY be used as a secondary key for selecting Link Objects
    /// which share the same relation type.
    pub name: Option<String>,

    /// The "profile" property is OPTIONAL.
    ///
    /// Its value is a string which is a URI that hints about the profile (as
    /// defined by [I-D.wilde-profile-link]) of the target resource.
    pub profile: Option<String>,
    /// The "title" property is OPTIONAL.
    ///
    /// Its value is a string and is intended for labelling the link with a
    /// human-readable identifier (as defined by [RFC5988]).
    pub title: Option<String>,
    /// The "hreflang" property is OPTIONAL.
    ///
    /// Its value is a string and is intended for indicating the language of
    /// the target resource (as defined by [RFC5988]).
    pub hreflang: Option<String>,
}

macro_rules! chainable_string {
    ($x: ident, $y: ident) => {
        pub fn $y(mut self, $x: &str) -> Self {
            self.$x = Some($x.to_string());
            self
        }

        pub fn $x(&self) -> Option<String> {
            self.$x.clone()
        }
    }
}

impl HalLink {
    pub fn new<S>(href: S) -> HalLink
    where
        S: Into<String>,
    {
        HalLink {
            href: href.into(),
            templated: false,
            media_type: None,
            deprecation: None,
            name: None,
            profile: None,
            title: None,
            hreflang: None,
        }
    }

    pub fn templated(mut self, templated: bool) -> Self {
        self.templated = templated;
        self
    }

    chainable_string!(media_type, with_media_type);
    chainable_string!(deprecation, with_deprecation);
    chainable_string!(name, with_name);
    chainable_string!(profile, with_profile);
    chainable_string!(title, with_title);
    chainable_string!(hreflang, with_hreflang);
}

impl<T> From<T> for HalLink
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        HalLink::new(s)
    }
}

impl Serialize for HalLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 1;
        if self.templated {
            len += 1;
        }
        if self.media_type.is_some() {
            len += 1;
        }
        if self.deprecation.is_some() {
            len += 1;
        }
        if self.profile.is_some() {
            len += 1;
        }
        if self.name.is_some() {
            len += 1;
        }
        if self.title.is_some() {
            len += 1;
        }
        if self.hreflang.is_some() {
            len += 1;
        }
        let mut state = try!(serializer.serialize_map(Some(len)));
        try!(state.serialize_entry("href", &self.href));
        if self.templated {
            try!(state.serialize_key("templated"));
            try!(state.serialize_value(&true));
        }
        if let Some(ref s) = self.media_type {
            try!(state.serialize_key("type"));
            try!(state.serialize_value(s));
        };
        if let Some(ref s) = self.deprecation {
            try!(state.serialize_key("deprecation"));
            try!(state.serialize_value(s));
        };
        if let Some(ref s) = self.profile {
            try!(state.serialize_key("profile"));
            try!(state.serialize_value(s));
        };
        if let Some(ref s) = self.name {
            try!(state.serialize_key("name"));
            try!(state.serialize_value(s));
        };
        if let Some(ref s) = self.title {
            try!(state.serialize_key("title"));
            try!(state.serialize_value(s));
        };
        if let Some(ref s) = self.hreflang {
            try!(state.serialize_key("hreflang"));
            try!(state.serialize_value(s));
        };
        state.end()
    }
}

impl<'de> Deserialize<'de> for HalLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LinkVisitor;
        impl LinkVisitor {
            fn new() -> Self {
                LinkVisitor {}
            }
        }
        /// A visitor for deserializing `HalLink` from map representation, ignoring unknown keys,
        /// and allowing missing fields.
        impl<'de> de::Visitor<'de> for LinkVisitor {
            type Value = HalLink;

            fn visit_map<M>(self, mut visitor: M) -> Result<HalLink, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut link = HalLink::new("");
                while let Some(key) = try!(visitor.next_key::<String>()) {
                    if key.deref() == "templated" {
                        link.templated = try!(visitor.next_value());
                    } else {
                        let value = try!(visitor.next_value::<String>());
                        match key.deref() {
                            "href" => link.href = value,
                            "type" => link.media_type = Some(value),
                            "deprecation" => link.deprecation = Some(value),
                            "profile" => link.profile = Some(value),
                            "name" => link.name = Some(value),
                            "title" => link.title = Some(value),
                            "hreflang" => link.hreflang = Some(value),
                            &_ => (),
                        }
                    }
                }
                //try!(visitor.end());
                Ok(link)
            }

            fn expecting(&self, f: &mut Formatter) -> Result<(), Error> {
                Ok(())
            }
        }

        deserializer.deserialize_map(LinkVisitor::new())
    }
}

/// Two links are the same if their href is the same
/// The rest is immaterial
impl PartialEq for HalLink {
    fn eq(&self, other: &HalLink) -> bool {
        self.href == other.href
    }
}
