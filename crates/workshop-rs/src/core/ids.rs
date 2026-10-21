use std::marker::PhantomData;

/// A typed, stable index into an [`Arena`](crate::core::arena::Arena).
///
/// `T` is the referenced type and is used only as a marker; `Id<T>` has the
/// size of `u32` and is `Copy`, `Send`, and `Sync` regardless of `T`. The
/// comparison, hashing, and formatting impls are implemented manually so they
/// never require `T` itself to implement them.
pub struct Id<T> {
    index: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.index).finish()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Id<T> {
    /// Build an ID from an arena index.
    ///
    /// This is the only way to construct an ID outside an arena. Prefer
    /// [`Arena::push`](crate::core::arena::Arena::push) so the ID is valid by
    /// construction; `from_index` exists for tests and deserialization paths
    /// that then rely on bounds-checked lookup.
    pub const fn from_index(index: usize) -> Self {
        Id {
            index: index as u32,
            _marker: PhantomData,
        }
    }

    /// The arena index this ID refers to.
    pub const fn index(self) -> usize {
        self.index as usize
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::Id;

    struct File;
    struct Rule;

    #[test]
    fn ids_are_cheap_copyable_and_comparable() {
        let a = Id::<File>::from_index(3);
        let b = Id::<File>::from_index(3);
        let c = Id::<File>::from_index(4);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.index(), 3);
        assert!(a < c);
        assert_eq!(std::mem::size_of::<Id<File>>(), 4);
    }

    #[test]
    fn id_types_are_not_interchangeable() {
        let _file: Id<File> = Id::from_index(0);
        let _rule: Id<Rule> = Id::from_index(0);
        let _ = _file;
        let _ = _rule;
    }

    #[test]
    fn ids_survive_hashing() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Id::<File>::from_index(7));
        assert!(set.contains(&Id::<File>::from_index(7)));
    }
}
