use std::{collections::HashMap, marker::PhantomData};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Extent {
    Positive,
    Negative,
    InBound(usize),
}

pub type HyperCoord<const N: usize> = [Extent; N];

#[derive(Copy, Clone, Debug)]
pub struct HyperSurfaceMeta<const N: usize> {
    max_dim: usize,
    inner_size: usize,
}

pub struct HyperSurface<const N: usize, T> {
    planes: HashMap<HyperCoord<N>, Vec<T>>,
    meta: HyperSurfaceMeta<N>,
}

impl<const N: usize, T> HyperSurface<N, T> {
    pub fn new(meta: HyperSurfaceMeta<N>) -> Self
    where
        T: Default + Clone,
    {
        let mut planes = HashMap::new();

        for plane_id in meta.all_planes() {
            let var_dims = count_var_dims(plane_id) as u32;
            let flat_size = meta.inner_size.pow(var_dims);
            let arr = vec![T::default(); flat_size];
            planes.insert(plane_id, arr);
        }

        Self { planes, meta }
    }

    pub fn meta(&self) -> HyperSurfaceMeta<N> {
        self.meta
    }
}

impl<const N: usize, T> std::ops::Index<HyperCoord<N>> for HyperSurface<N, T> {
    type Output = T;
    fn index(&self, c: HyperCoord<N>) -> &T {
        let plane = self
            .planes
            .get(&set_hypercoord_inbound_vals(c, 0))
            .expect("Invalid plane");
        let idx = self.meta.index_dense(c).expect("Out of hyperbounds index");
        &plane[idx]
    }
}

impl<const N: usize, T> std::ops::IndexMut<HyperCoord<N>> for HyperSurface<N, T> {
    fn index_mut(&mut self, c: HyperCoord<N>) -> &mut T {
        let plane = self
            .planes
            .get_mut(&set_hypercoord_inbound_vals(c, 0))
            .expect("Invalid plane");
        let idx = self.meta.index_dense(c).expect("Out of hyperbounds index");
        &mut plane[idx]
    }
}

impl<const N: usize> HyperSurfaceMeta<N> {
    pub fn new(inner_size: usize, max_dim: usize) -> Self {
        Self {
            max_dim,
            inner_size,
        }
    }

    pub fn index_dense(&self, c: HyperCoord<N>) -> Option<usize> {
        let mut index = 0;
        let mut stride = 1;
        let mut count = 0;

        for p in c {
            match p {
                Extent::InBound(v) => {
                    index += stride * v;
                    stride *= self.inner_size;
                    count += 1;
                }
                _ => (),
            }
        }

        (0..=self.max_dim).contains(&count).then(|| index)
    }

    pub fn all_planes(&self) -> Vec<HyperCoord<N>> {
        let mut planes = vec![];

        // For each possible number of varying dims (e.g. n_var_dims = 2 means index over a plane
        for n_var_dims in 0..=self.max_dim {
            for var_dims in n_choose_m(N, n_var_dims) {
                // For each possible plane this could be on (e.g. top or bottom of cube)
                let n_stable_dims = N - n_var_dims;
                for signs in 0..1u64 << n_stable_dims {
                    let mut bit = 0;
                    let mut sel_dim = 0;
                    let mut plane = [Extent::InBound(0); N];

                    // For each dimension in the hypercoord
                    for i in 0..N {
                        // If this is a variable dim, leave it as InBound(0)
                        if Some(&i) == var_dims.get(sel_dim) {
                            sel_dim += 1;
                        } else {
                            // Otherwise, use the bits to set the sign
                            plane[i] = match (signs >> bit) & 1 == 1 {
                                true => Extent::Positive,
                                false => Extent::Negative,
                            };
                            bit += 1;
                        }
                    }

                    planes.push(plane);
                }
            }
        }

        planes
    }

    pub fn dense_coords(&self) -> Vec<HyperCoord<N>> {
        let mut output = vec![];
        for plane in self.all_planes() {
            self.dense_coords_rec(&mut output, N - 1, plane);
        }
        output
    }

    pub fn dense_coords_rec(
        &self,
        out: &mut Vec<HyperCoord<N>>,
        idx: usize,
        mut plane: HyperCoord<N>,
    ) {
        match plane[idx] {
            Extent::InBound(_) => {
                for pos in 0..self.inner_size {
                    plane[idx] = Extent::InBound(pos);

                    if let Some(lower) = idx.checked_sub(1) {
                        self.dense_coords_rec(out, lower, plane);
                    } else {
                        out.push(plane);
                    }
                }
            }
            _ => {
                if let Some(lower) = idx.checked_sub(1) {
                    self.dense_coords_rec(out, lower, plane);
                } else {
                    out.push(plane);
                }
            }
        }
    }

    pub fn coord_euclid(&self, coord: HyperCoord<N>) -> [usize; N] {
        coord.map(|v| match v {
            Extent::Positive => self.inner_size + 1,
            Extent::Negative => 0,
            Extent::InBound(v) => v + 1,
        })
    }

    pub fn neighbors(self, coord: HyperCoord<N>) -> Neighbors<N> {
        Neighbors::new(self, coord)
    }
}

pub struct Neighbors<const N: usize> {
    meta: HyperSurfaceMeta<N>,
    coord: HyperCoord<N>,
    idx: usize,
    sign: bool,
}

impl<const N: usize> Neighbors<N> {
    fn new(meta: HyperSurfaceMeta<N>, coord: HyperCoord<N>) -> Self {
        Self {
            meta,
            coord,
            idx: 0,
            sign: false,
        }
    }

    fn advance(&mut self) {
        if self.sign {
            self.idx += 1;
        }
        self.sign = !self.sign;
    }
}

impl<const N: usize> Iterator for Neighbors<N> {
    type Item = HyperCoord<N>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx == self.coord.len() {
                return None;
            }

            if let Some(neigh) =
                extent_neighbor(self.coord[self.idx], self.sign, self.meta.inner_size)
            {
                let mut output = self.coord;
                output[self.idx] = neigh;

                self.advance();
                if count_var_dims(output) > self.meta.max_dim {
                    continue;
                }

                return Some(output);
            } else {
                self.advance();
            }
        }
    }
}

fn count_var_dims<const N: usize>(coord: HyperCoord<N>) -> usize {
    coord
        .into_iter()
        .filter(|v| matches!(v, Extent::InBound(_)))
        .count()
}

/// Returns the neighbor extent of `e` in the direction of `sign` on the dimension with the given
/// length `inner_size`, if any.
/// Sign: True means increase, False means decrease.
fn extent_neighbor(e: Extent, sign: bool, inner_size: usize) -> Option<Extent> {
    match e {
        Extent::Positive => match sign {
            true => None,
            false => {
                if inner_size > 0 {
                    Some(Extent::InBound(inner_size - 1))
                } else {
                    Some(Extent::Negative)
                }
            }
        },
        Extent::Negative => match sign {
            true => {
                if inner_size > 0 {
                    Some(Extent::InBound(0))
                } else {
                    Some(Extent::Positive)
                }
            }
            false => None,
        },
        Extent::InBound(v) => match sign {
            true => {
                if v + 1 > inner_size {
                    return None;
                }

                if v + 1 == inner_size {
                    Some(Extent::Positive)
                } else {
                    Some(Extent::InBound(v + 1))
                }
            }
            false => {
                if v == 0 {
                    Some(Extent::Negative)
                } else {
                    v.checked_sub(1).map(Extent::InBound)
                }
            }
        },
    }
}

fn set_hypercoord_inbound_vals<const N: usize>(c: HyperCoord<N>, value: usize) -> HyperCoord<N> {
    c.map(|p| match p {
        Extent::InBound(_) => Extent::InBound(value),
        _ => p,
    })
}

/// Output with the given const size, but use the given value of m
pub fn n_choose_m(n: usize, m: usize) -> Vec<Vec<usize>> {
    let m_minus_one = match m.checked_sub(1) {
        Some(mmo) => mmo,
        None => return vec![vec![]],
    };

    let mut out = vec![];

    for i in m_minus_one..n {
        for mut sub in n_choose_m(i, m_minus_one) {
            sub.push(i);
            out.push(sub);
        }
    }
    out
}
