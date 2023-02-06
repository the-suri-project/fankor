use std::any::TypeId;

pub mod bpf_writer;
pub mod close;
pub mod realloc;
pub mod rent;

/// Gets the type identifier of a given value.
pub fn type_id_of<T: ?Sized + 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}
