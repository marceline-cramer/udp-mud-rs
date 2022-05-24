use protocol::*;
use protocol_derive::Encode;

#[derive(Debug, Encode)]
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
}
