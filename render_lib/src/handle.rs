use std::marker::PhantomData;

pub struct Handle<T>(usize, PhantomData<T>);
impl<T> Handle<T> {
    pub(crate) const fn new(val: usize) -> Handle<T> {
        Handle(val, PhantomData)
    }

    pub(crate) const fn index(&self) -> usize {
        self.0
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle::new(self.0)
    }
}

impl<T> Copy for Handle<T> {
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        other.0 == self.0
    }
}

impl<T> Eq for Handle<T> {
}
