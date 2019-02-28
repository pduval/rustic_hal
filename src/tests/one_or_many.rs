//use serde::de::Deserialize;
use super::super::link::HalLink;
use super::super::resource::OneOrMany;
use serde_json::{from_str, to_string};
use serde::*;

#[derive(Serialize, Deserialize)]
struct Boh
{
    oom: OneOrMany<String>
}

#[test]
fn ensure_one_object_gets_serialized_as_one() {
    let boh = Boh { oom: OneOrMany::new().with(&"test".to_owned()) };
    assert_eq!(to_string(&boh).unwrap(), r#"{"oom":"test"}"#);
}

#[test]
fn ensure_two_objects_get_serialized_as_array() {

    let boh = Boh { oom: OneOrMany::new().with(&"test".to_owned()).with(&"test2".to_owned()) };
    assert_eq!(to_string(&boh).unwrap(), r#"{"oom":["test","test2"]}"#);
}


#[test]
fn ensure_array_gets_deserialized() {
    let s = r#"{"oom":["test","test2"]}"#;
    let boh: Boh = from_str(s).unwrap();
    assert_eq!(2, boh.oom.len());
    assert_eq!("test", boh.oom.many()[0]);
    assert_eq!("test2", boh.oom.many()[1]);

}