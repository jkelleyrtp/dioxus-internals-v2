use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(pub NonZeroUsize);

impl Default for ElementId {
    fn default() -> Self {
        Self(NonZeroUsize::new(1).unwrap())
    }
}

pub struct Arena {
    counter: NonZeroUsize,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            counter: NonZeroUsize::new(1).unwrap(),
        }
    }
}

impl Arena {
    pub fn next(&mut self) -> ElementId {
        let id = self.counter;
        self.counter = NonZeroUsize::new(self.counter.get() + 1).unwrap();
        ElementId(id)
    }
}
