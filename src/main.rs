use protocol::*;
use protocol_derive::{Decode, Encode};

#[derive(Debug, Decode, Encode)]
struct TestData {
    var_int: Var<u32>,
    string: String,
}

fn main() {
    let data = TestData {
        var_int: Var(42),
        string: "Hello world!".to_string(),
    };

    let mut buf = Vec::new();
    data.encode(&mut buf).unwrap();
    println!("encoded: {:?}", buf);

    let mut reader = buf.as_slice();
    let decoded = TestData::decode(&mut reader).unwrap();
    println!("decoded: {:#?}", decoded);
}
