use partial::Partial;

#[derive(Partial)]
struct Example {
    field1: i32,
    #[partial]
    field2: String,
}

fn main() {}
