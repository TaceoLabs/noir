use std::collections::HashMap;

/// A unique ID corresponding to a value of type T.
/// This type can be used to retrieve a value of type T from
/// either a DenseMap<T> or SparseMap<T>.
///
/// Note that there is nothing in an Id binding it to a particular
/// DenseMap or SparseMap. If an Id was created to correspond to one
/// particular map type, users need to take care not to use it with
/// another map where it will likely be invalid.
pub(crate) struct Id<T> {
    index: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    /// Constructs a new Id for the given index.
    /// This constructor is deliberately private to prevent
    /// constructing invalid IDs.
    fn new(index: usize) -> Self {
        Self { index, _marker: std::marker::PhantomData }
    }

    /// Creates a test Id with the given index.
    /// The name of this function makes it apparent it should only
    /// be used for testing. Obtaining Ids in this way should be avoided
    /// as unlike DenseMap::push and SparseMap::push, the Ids created
    /// here are likely invalid for any particularly map.
    #[cfg(test)]
    pub(crate) fn test_new(index: usize) -> Self {
        Self::new(index)
    }
}

// Need to manually implement most impls on Id.
// Otherwise rust assumes that Id<T>: Hash only if T: Hash,
// which isn't true since the T is not used internally.
impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self { index: self.index, _marker: self._marker }
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Deliberately formatting as a tuple with 1 element here and omitting
        // the _marker: PhantomData field which would just clutter output
        f.debug_tuple("Id").field(&self.index).finish()
    }
}

/// A DenseMap is a Vec wrapper where each element corresponds
/// to a unique ID that can be used to access the element. No direct
/// access to indices is provided. Since IDs must be stable and correspond
/// to indices in the internal Vec, operations that would change element
/// ordering like pop, remove, swap_remove, etc, are not possible.
#[derive(Debug)]
pub(crate) struct DenseMap<T> {
    storage: Vec<T>,
}

impl<T> DenseMap<T> {
    /// Returns the number of elements in the map.
    pub(crate) fn len(&self) -> usize {
        self.storage.len()
    }
    /// Adds an element to the map.
    /// Returns the identifier/reference to that element.
    pub(crate) fn push(&mut self, element: T) -> Id<T> {
        let id = Id::new(self.storage.len());
        self.storage.push(element);
        id
    }
}

impl<T> Default for DenseMap<T> {
    fn default() -> Self {
        Self { storage: Vec::new() }
    }
}

impl<T> std::ops::Index<Id<T>> for DenseMap<T> {
    type Output = T;

    fn index(&self, id: Id<T>) -> &Self::Output {
        &self.storage[id.index]
    }
}

impl<T> std::ops::IndexMut<Id<T>> for DenseMap<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut Self::Output {
        &mut self.storage[id.index]
    }
}

/// A SparseMap is a HashMap wrapper where each element corresponds
/// to a unique ID that can be used to access the element. No direct
/// access to indices is provided.
///
/// Unlike DenseMap, SparseMap's IDs are stored within the structure
/// and are thus stable after element removal.
///
/// Note that unlike DenseMap, it is possible to panic when retrieving
/// an element if the element's Id has been invalidated by a previous
/// call to .remove().
#[derive(Debug)]
pub(crate) struct SparseMap<T> {
    storage: HashMap<Id<T>, T>,
}

impl<T> SparseMap<T> {
    /// Returns the number of elements in the map.
    pub(crate) fn len(&self) -> usize {
        self.storage.len()
    }

    /// Adds an element to the map.
    /// Returns the identifier/reference to that element.
    pub(crate) fn push(&mut self, element: T) -> Id<T> {
        let id = Id::new(self.storage.len());
        self.storage.insert(id, element);
        id
    }

    /// Remove an element from the map and return it.
    /// This may return None if the element was already
    /// previously removed from the map.
    pub(crate) fn remove(&mut self, id: Id<T>) -> Option<T> {
        self.storage.remove(&id)
    }
}

impl<T> Default for SparseMap<T> {
    fn default() -> Self {
        Self { storage: HashMap::new() }
    }
}

impl<T> std::ops::Index<Id<T>> for SparseMap<T> {
    type Output = T;

    fn index(&self, id: Id<T>) -> &Self::Output {
        &self.storage[&id]
    }
}

impl<T> std::ops::IndexMut<Id<T>> for SparseMap<T> {
    fn index_mut(&mut self, id: Id<T>) -> &mut Self::Output {
        self.storage.get_mut(&id).expect("Invalid id used in SparseMap::index_mut")
    }
}