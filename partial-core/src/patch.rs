use crate::marker::{Patched, Unpatched};

pub unsafe trait Patchable
where
    Self: Unpatched,
{
    type Args;
    type Patched: Patched;

    fn patch(&self, args: Self::Args) -> Self::Patched;
}

#[cfg(test)]
mod tests {
    use crate::{marker::{Patched, Unpatched}, patch::Patchable};

    struct Example {
        value: i32,
    }

    unsafe impl Patched for Example {}
    struct ExampleUnpatched;
    unsafe impl Unpatched for ExampleUnpatched {}

    unsafe impl Patchable for ExampleUnpatched {
        type Args = i32;
        type Patched = Example;

        fn patch(&self, args: Self::Args) -> Self::Patched {
            Example { value: args }
        }
    }

    #[test]
    fn test_unpatched() {
        let example = ExampleUnpatched;
        let patched_example = example.patch(42);
        assert_eq!(patched_example.value, 42);
    }
}
