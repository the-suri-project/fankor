use std::any::TypeId;

pub mod close;
pub mod realloc;
pub mod rent;
pub mod writers;

/// Gets the type identifier of a given value.
pub fn type_id_of<T: ?Sized + 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}
