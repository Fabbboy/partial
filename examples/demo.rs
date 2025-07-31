use partial::Partial;

#[derive(Partial)]
struct Example {
    field1: i32,
    #[partial]
    field2: String,
}

#[derive(Partial)]
struct Lol(#[partial] pub usize, u32);

fn main() {
    let examp = Example {
        field1: 10,
        field2: "Hello".to_string(),
    };

    println!(
        "Example: field1 = {}, field2 = {}",
        examp.field1, examp.field2
    );
}
