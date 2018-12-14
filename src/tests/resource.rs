use super::{
    super::{resource::*, HalLink},
    Test1,
};
use serde_json::{from_str, to_string};

//#[derive(Serialize, Deserialize)]
// struct Test1 {
//    a: String
//}

#[test]
fn check_data_gets_serialized() {
    let f: HalResource = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    let s = to_string(&f).unwrap();
    assert_eq!(s, r#"{"a":"Test"}"#);
}

#[test]
fn check_link_gets_serialized_without_empty_attributes() {
    let mut f: HalResource = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    f.with_link("self", "https://self.com");
    let s = to_string(&f).unwrap();
    assert_eq!(
        s,
        r#"{"_links":{"self":{"href":"https://self.com"}},"a":"Test"}"#
    );
}

#[test]
fn check_link_arrays_get_serialized() {
    let mut f: HalResource = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    f.with_link("self", "https://self.com")
        .with_link("alfa", "https://self.com/beta")
        .with_link("alfa", "https://self.com/gamma");
    let s = to_string(&f).unwrap();
    assert_eq!(s, r#"{"_links":{"alfa":[{"href":"https://self.com/beta"},{"href":"https://self.com/gamma"}],"self":{"href":"https://self.com"}},"a":"Test"}"#);
}

#[test]
fn check_links_get_fully_serialized() {
    let mut f: HalResource = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    f.with_link(
        "self",
        HalLink::new("https://self.com")
            .with_title("Self Link")
            .with_name("moi")
            .with_deprecation("http://explain.com/why"),
    );
    let s = to_string(&f).unwrap();
    assert_eq!(s, "{\"_links\":{\"self\":{\"href\":\"https://self.com\",\"deprecation\":\"http://explain.com/why\",\"name\":\"moi\",\"title\":\"Self Link\"}},\"a\":\"Test\"}");
}

#[test]
fn check_embedded_resource_gets_serialized() {
    let mut r1 = HalResource::new(Test1 {
        a: "Test2".to_string(),
    });
    r1.with_link("self", "https://self2.com");

    let mut f = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    f.with_link("self", "https://self.com")
        .with_resource("child", r1);

    let s = to_string(&f).unwrap();
    let target = "{\"_links\":{\"self\":{\"href\":\"https://self.com\"}},\"_embedded\":{\"child\":{\"_links\":{\"self\":{\"href\":\"https://self2.com\"}},\"a\":\"Test2\"}},\"a\":\"Test\"}";
    assert_eq!(s, target);
}

#[test]
fn check_curies_get_serialized_in_links() {
    let mut r1 = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    r1.with_curie("cur", "https://curie.org")
        .with_link("self", "https://self.com");
    let s = to_string(&r1).unwrap();
    let target = "{\"_links\":{\"curies\":[{\"href\":\"https://curie.org\",\"templated\":true,\"name\":\"cur\"}],\"self\":{\"href\":\"https://self.com\"}},\"a\":\"Test\"}";
    assert_eq!(s, target);
}

#[test]
fn check_simple_resource_gets_deserialized() {
    let source = r#"{ "_links":{"self":{"href": "https://www.test.com"}}, "a": "123"}"#;
    let hal: HalResource = from_str(source).unwrap();
    assert_eq!(hal.get_self(), Some(&HalLink::new("https://www.test.com")));
}

#[test]
fn check_extra_fields_get_serialized() {
    let mut f: HalResource = HalResource::new(Test1 {
        a: "Test".to_string(),
    });
    f.with_extra_data("int", 123);
    f.with_extra_data("string", "Hello!?");
    let s = to_string(&f).unwrap();
    assert_eq!(s, r#"{"a":"Test","int":123,"string":"Hello!?"}"#);
}

#[test]
fn check_extra_fields_get_deserialized() {
    let source = r#"{ "_links":{"self":{"href": "https://www.test.com"}}, "a": "123", "b":456}"#;
    let hal: HalResource = from_str(source).unwrap();
    assert_eq!(hal.get_extra_data::<i32>("b").unwrap(), 456);
}

#[test]
fn check_force_many_serializes_to_empty_array_if_empty_resources() {
    let mut resource = HalResource::new("");
    resource.with_resources("empty_array", Vec::new());

    let s = to_string(&resource).unwrap();
    assert_eq!(s, r#"{"_embedded":{"empty_array":[]}}"#);
}
