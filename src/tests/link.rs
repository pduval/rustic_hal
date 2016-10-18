//use serde::de::Deserialize;
use serde_json::{from_str};
use super::super::link::{HalLink};

#[test]
fn ensure_href_gets_deserialized() {
    let link: HalLink = from_str(r#"{"href":"https://test.com"}"#).unwrap();
    assert_eq!(link.href, "https://test.com");
}

#[test]
fn ensure_templated_gets_deserialized() {
    let link: HalLink = from_str(r#"{"href":"https://test.com","templated":true}"#).unwrap();
    assert_eq!(link.href, "https://test.com");
    assert_eq!(link.templated, true);
}

#[test]
fn ensure_full_link_gets_deserialized() {
    let link : HalLink = from_str(r#"
{
   "href": "https://www.google.com",
   "name": "google",
   "templated": false,
   "hreflang": "en-US",
   "deprecation": "https://www.google.com/deprecation",
   "title": "Google Search"
}"#).unwrap();

    assert_eq!(link.href, "https://www.google.com");
    assert_eq!(link.name, Some("google".to_string()));
    assert_eq!(link.templated, false);
    assert_eq!(link.hreflang, Some("en-US".to_string()));
    assert_eq!(link.deprecation, Some("https://www.google.com/deprecation".to_string()));
    assert_eq!(link.title, Some("Google Search".to_string()));
}



                                  
