#[macro_use]
extern crate criterion;

extern crate rustic_hal;
extern crate serde_json;

use rustic_hal::resource::*;
use rustic_hal::HalLink;
use serde_json::{from_str, to_string};

use criterion::Criterion;

fn speedy_serialisation(c: &mut Criterion) {
    let source = r#"{ "_links":{"self":{"href": "https://www.test.com"}}, "a": "123", "b":456}"#;
    let hal: HalResource = from_str(source).unwrap();
    c.bench_function("simple serialisation", move |b| {
        b.iter(|| to_string(&hal).unwrap())
    });
}

criterion_group!(benches, speedy_serialisation);
criterion_main!(benches);
