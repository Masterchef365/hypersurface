struct HyperCoord<const N: usize>(pub [isize; N]);

struct HyperSurface {}

pub struct ArrayNd<const N: usize, T> {
    data: Vec<T>,
    dims: [usize; N],
}

impl<const N: usize, T> ArrayNd<N, T> {
    pub fn new(dims: [usize; N]) -> Self
    where
        T: Default + Clone,
    {
        let size = dims.into_iter().product();
        let data = vec![T::default(); size];
        Self::from_vec(dims, data)
    }

    pub fn from_vec(dims: [usize; N], data: Vec<T>) -> Self {
        debug_assert_eq!(data.len(), dims.into_iter().product());
        Self { data, dims }
    }

    pub fn calc_index(&self, index: [usize; N]) -> usize {
        let mut linear = 0;
        let mut stride = 1;
        for (dim, pos) in self.dims.into_iter().zip(index) {
            linear += stride * pos;
            stride *= dim;
        }
        linear
    }

    pub fn dims(&self) -> [usize; N] {
        self.dims
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<const N: usize, T> std::ops::Index<[usize; N]> for ArrayNd<N, T> {
    type Output = T;
    fn index(&self, pos: [usize; N]) -> &T {
        &self.data[self.calc_index(pos)]
    }
}

impl<const N: usize, T> std::ops::IndexMut<[usize; N]> for ArrayNd<N, T> {
    fn index_mut(&mut self, pos: [usize; N]) -> &mut T {
        let idx = self.calc_index(pos);
        &mut self.data[idx]
    }
}
