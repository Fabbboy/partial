use partial::{partial::PartialBox, Partial};

#[derive(Partial, Debug)]
struct Example {
    field1: i32,
    #[partial]
    field2: String,
}

#[derive(Partial)]
struct Lol(#[partial] pub usize, u32);

fn main() {
    let mut exmp = PartialBox::Unpatched(ExampleUnpatched {field1: 10});
    println!("Unpatched Example: {:?}", exmp);
}
