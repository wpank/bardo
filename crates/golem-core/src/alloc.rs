//! Per-tick arena allocator.

use bumpalo::Bump;

/// Bump-allocated arena that can be reset at tick boundaries.
#[derive(Debug)]
pub struct TickArena {
    inner: Bump,
}

impl TickArena {
    /// Creates a new arena.
    pub fn new() -> Self {
        Self { inner: Bump::new() }
    }

    /// Resets the arena, invalidating all previously allocated values.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Allocates a value in the arena.
    pub fn alloc<T>(&self, val: T) -> &T {
        self.inner.alloc(val)
    }

    /// Allocates a copied slice in the arena.
    pub fn alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &[T] {
        self.inner.alloc_slice_copy(slice)
    }
}

impl Default for TickArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TickArena;

    #[test]
    fn tick_arena_reset() {
        let mut arena = TickArena::new();
        let value = arena.alloc(7u32);
        assert_eq!(*value, 7);

        let copied = arena.alloc_slice_copy(&[1u8, 2, 3]);
        assert_eq!(copied, &[1, 2, 3]);

        arena.reset();
        let post_reset = arena.alloc(11u32);
        assert_eq!(*post_reset, 11);
    }
}
