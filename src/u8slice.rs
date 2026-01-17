/// Trait is needed to use bytemuck's conversion on external types
pub trait ToU8Slice {
    fn cast_slice(&self) -> &[u8];
}

impl<T: bytemuck::Pod> ToU8Slice for T {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

pub trait ToU8SliceArray {
    fn cast_slice(&self) -> &[u8];
}

impl<T: bytemuck::Pod> ToU8SliceArray for [T] {
    fn cast_slice(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}