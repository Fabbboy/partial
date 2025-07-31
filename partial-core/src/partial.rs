#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::patch::Patchable;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum PartialBox<T>
where
    T: Patchable,
{
    Unpatched(T),
    Patched(T::Patched),
}

impl<T> PartialBox<T>
where
    T: Patchable,
{
    pub fn patch(self, value: T::Args) -> Self {
        match self {
            PartialBox::Unpatched(unpatched) => PartialBox::Patched(unpatched.patch(value)),
            PartialBox::Patched(patched) => PartialBox::Patched(patched),
        }
    }
}
