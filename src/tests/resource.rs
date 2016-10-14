use super::super::resource::*;
use serde_json::to_string;
use super::super::HalLink;
#[derive(Serialize)]
struct Test1 {
    a: String
}

#[test]
fn check_data_gets_serialized() {
    let f: HalResource<Test1> = HalResource::new(Test1 { a: "Test".to_string() });
    let s = to_string(&f).unwrap();
    assert_eq!(s, "{\"a\":\"Test\"}");
}

#[test]
fn check_link_gets_serialized_without_empty_attributes() {
    
    let mut f: HalResource<Test1> = HalResource::new(Test1 { a: "Test".to_string() });
    f.with_link_uri("self", "https://self.com");
    let s = to_string(&f).unwrap();
    assert_eq!(s, "{\"_links\":{\"self\":{\"href\":\"https://self.com\"}},\"a\":\"Test\"}");
        
}

#[test]
fn check_link_arrays_get_serialized() {
    let mut f: HalResource<Test1> = HalResource::new(Test1 { a: "Test".to_string() });
    f.with_link_uri("self", "https://self.com")
        .with_link_uri("alfa", "https://self.com/beta")
        .with_link_uri("alfa", "https://self.com/gamma");
    let s = to_string(&f).unwrap();
    assert_eq!(s, "{\"_links\":{\"alfa\":[{\"href\":\"https://self.com/beta\"},{\"href\":\"https://self.com/gamma\"}],\"self\":{\"href\":\"https://self.com\"}},\"a\":\"Test\"}");
        
}

#[test]
fn check_links_get_fully_serialized() {
    let mut f: HalResource<Test1> = HalResource::new(Test1 { a: "Test".to_string() });
    f.with_link("self", HalLink::new("https://self.com").with_title("Self Link").with_name("moi").with_deprecation("http://explain.com/why"));
    let s = to_string(&f).unwrap();
    assert_eq!(s, "{\"_links\":{\"self\":\"https://self.com\",\"title\":\"Self Link\",\"name\":\"moi\",\"deprecation\":\"http://explain.com/why\"},\"a\":\"Test\"}");
}
