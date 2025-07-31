use crate::patch::Patchable;

#[derive(Debug)]
pub enum PartialBox<T: Patchable> {
    Unpatched(T),
    Patched(T::Patched),
}
