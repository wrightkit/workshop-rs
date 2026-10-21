use super::ids::Id;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub(crate) const fn new() -> Self {
        Arena { items: Vec::new() }
    }

    pub(crate) fn push(&mut self, value: T) -> Id<T> {
        let id = Id::from_index(self.items.len());
        self.items.push(value);
        id
    }

    pub(crate) fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(id.index())
    }

    pub(crate) fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.items.get_mut(id.index())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn contains(&self, id: Id<T>) -> bool {
        id.index() < self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Arena;

    #[test]
    fn push_assigns_sequential_stable_ids() {
        let mut arena = Arena::new();
        let a = arena.push(10);
        let b = arena.push(20);
        let c = arena.push(30);
        assert_eq!(a.index(), 0);
        assert_eq!(b.index(), 1);
        assert_eq!(c.index(), 2);
        assert_eq!(arena.get(a), Some(&10));
        assert_eq!(arena.get(b), Some(&20));
        assert_eq!(arena.get(c), Some(&30));
    }

    #[test]
    fn out_of_range_ids_return_none_without_panicking() {
        let mut arena = Arena::new();
        let valid = arena.push("x");
        let dangling = super::super::ids::Id::from_index(valid.index() + 1);
        let far_out = super::super::ids::Id::from_index(usize::MAX);
        assert_eq!(arena.get(valid), Some(&"x"));
        assert_eq!(arena.get(dangling), None);
        assert_eq!(arena.get(far_out), None);
        assert!(!arena.contains(dangling));
        assert!(arena.contains(valid));
    }

    #[test]
    fn get_mut_allows_in_place_updates_without_moving() {
        let mut arena = Arena::new();
        let id = arena.push(vec![1, 2]);
        arena.get_mut(id).unwrap().push(3);
        assert_eq!(arena.get(id), Some(&vec![1, 2, 3]));
        assert_eq!(arena.get(id).unwrap().len(), 3);
    }

    #[test]
    fn iteration_is_insertion_ordered() {
        let mut arena = Arena::new();
        arena.push(1);
        arena.push(2);
        arena.push(3);
        let collected: Vec<&i32> = arena.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }
}
