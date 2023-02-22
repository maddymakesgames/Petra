use std::marker::PhantomData;

pub struct Registry<T> {
    data: Vec<T>,
}

impl<T> Registry<T> {
    pub fn new() -> Registry<T> {
        Registry { data: Vec::new() }
    }

    pub fn add(&mut self, val: T) -> Handle<T> {
        let handle = Handle::new(self.data.len());
        self.data.push(val);
        handle
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.data.get(handle.0)
    }

    pub(crate) fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.data.get_mut(handle.0)
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a Registry<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Registry<T> {
    type IntoIter = std::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

pub struct Handle<T>(usize, PhantomData<T>);
impl<T> Handle<T> {
    pub(crate) const fn new(val: usize) -> Handle<T> {
        Handle(val, PhantomData)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle::new(self.0)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        other.0 == self.0
    }
}

impl<T> Eq for Handle<T> {}
