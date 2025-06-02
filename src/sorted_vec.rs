use std::fmt::Display;
use std::fmt::Debug;



#[derive(Debug, Clone)]
pub struct SortedVec<T>
where
    T: Ord,
{
    pub data: Vec<T>,
    size_limit: usize,
}

impl<T> SortedVec<T>
where
    T: Ord + Display + Debug,
{
    pub fn new(size_limit: usize) -> Self {
        SortedVec {
            data: Vec::with_capacity(size_limit),
            size_limit,
        }
    }

    pub fn merged(left: Self, right: Self) -> Self {
        let size_limit = usize::max(left.size_limit, right.size_limit);
        let mut vec = SortedVec {
            data: Vec::with_capacity(size_limit),
            size_limit,
        };

        for l in left.data {
          vec.insert_ordered(l);
        }

        for r in right.data {
          vec.insert_ordered(r);
        }

        vec
    }

    pub fn insert_ordered(&mut self, item: T) {
        if self.data.len() == self.size_limit {
          if let Some(last) = self.data.last() {
            if item > *last {
              return
            }
          }
        }

        match self.data.binary_search(&item) {
            Ok(pos) | Err(pos) => self.data.insert(pos, item),
        }

        if self.data.len() > self.size_limit {
            self.data.truncate(self.size_limit);
        }
    }
}
