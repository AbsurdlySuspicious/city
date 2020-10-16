use std::iter;
use std::ops::{Index, IndexMut};

pub type IndexXY = (usize, usize);

#[derive(Debug)]
pub struct Vec2D<T> {
    data: Vec<T>,
    size_x: usize,
    size_y: usize,
}

impl<T> Vec2D<T> {
    pub fn new(size_x: usize, size_y: usize, init: impl Fn() -> T) -> Vec2D<T> {
        let data = Vec::with_capacity(size_x * size_y);
        let mut vec2d = Vec2D { data, size_x, size_y };

        vec2d.fill_with_fn(init);
        vec2d
    }

    pub fn fill_with_fn(&mut self, f: impl Fn() -> T) {
        let cap = self.data.capacity();
        self.data.clear();
        self.data.extend(iter::repeat_with(f).take(cap));
    }

    pub fn get_row(&self, y: usize) -> &[T] {
        &self.data[raw_idx(self.size_x, (0, y))..][..self.size_x]
    }

    pub fn get_row_mut(&mut self, y: usize) -> &mut [T] {
        &mut self.data[raw_idx(self.size_x, (0, y))..][..self.size_x]
    }

    pub fn row_iter(&self) -> impl Iterator<Item=&[T]> {
        let (sx, sy) = (self.size_x, self.size_y);
        (0..sx * sy).step_by(sx).map(move |y| self.get_row(y))
    }
}

impl<T: Copy> Vec2D<T> {
    pub fn fill_with(&mut self, f: T) {
        let cap = self.data.capacity();
        self.data.clear();
        self.data.extend(iter::repeat(f).take(cap));
    }
}

#[inline]
fn raw_idx(size_x: usize, (x, y): IndexXY) -> usize {
    (y * size_x) + x
}

impl<T> Index<IndexXY> for Vec2D<T> {
    type Output = T;

    fn index(&self, idx: IndexXY) -> &Self::Output {
        &self.data[raw_idx(self.size_x, idx)]
    }
}

impl<T> IndexMut<IndexXY> for Vec2D<T> {
    fn index_mut(&mut self, idx: IndexXY) -> &mut Self::Output {
        &mut self.data[raw_idx(self.size_x, idx)]
    }
}
