use crate::mem::optional_ref::OptionalRef;

use super::any_internals::AnyInternal;

pub type Any = OptionalRef<AnyInternal>;
