struct Vec<T> { 
    slice: [T]
}

// A mutable vector type implemented as a wrapper around immutable slices.
// A separate type is technically not needed but helps differentiate which operations are mutable.
impl<T> Vec<T> {
    fn new() -> Self {
        Self { slice: [] }
    }

    // Create a Vec containing each element from the given slice.
    // Mutations to the resulting Vec will not affect the original slice.
    fn from_slice(slice: [T]) -> Self {
        Self { slice }
    }

    /// Get an element from the vector at the given index.
    /// Panics if the given index
    /// points beyond the end of the vector.
    fn get(&mut self, index: Field) -> T {
        self.slice[index]
     }

    /// Push a new element to the end of the vector, returning a
    /// new vector with a length one greater than the
    /// original unmodified vector.
    fn push(&mut self, elem: T) { 
        self.slice = self.slice.push_back(elem);
    }

    /// Pop an element from the end of the given vector, returning
    /// a new vector with a length of one less than the given vector,
    /// as well as the popped element.
    /// Panics if the given vector's length is zero.
    fn pop(&mut self) -> T { 
        let (popped_slice, last_elem) = self.slice.pop_back();
        self.slice = popped_slice;
        last_elem
    }

    /// Insert an element at a specified index, shifting all elements 
    /// after it to the right
    fn insert(&mut self, index: Field, elem: T) {
        self.slice = self.slice.insert(index, elem);
    } 

    /// Remove an element at a specified index, shifting all elements
    /// after it to the left, returning the removed element
    fn remove(&mut self, index: Field) -> T {
        let (new_slice, elem) = self.slice.remove(index);
        self.slice = new_slice;
        elem
    }

    /// Returns the number of elements in the vector
    fn len(self: Self) -> Field {
        self.slice.len()
    }
}