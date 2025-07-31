use partial::{Partial, partial::PartialBox};

#[derive(Partial, Debug)]
struct Example {
    field1: i32,
    #[partial]
    field2: String,
}

fn main() {
    let mut exmp = PartialBox::Unpatched(ExampleUnpatched { field1: 10 });
    println!("Unpatched Example: {:?}", exmp);
    exmp = exmp.patch(("Hello, World!".to_string(),));
    println!("Patched Example: {:?}", exmp);
}
