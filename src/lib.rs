use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Extent {
    Positive,
    Negative,
    InBound(usize),
}

pub type HyperCoord<const N: usize> = [Extent; N];

#[derive(Copy, Clone, Debug)]
pub struct HyperSurfaceMeta {
    max_dim: usize,
    side_len: usize,
}

pub struct HyperSurface<const N: usize, T> {
    planes: HashMap<HyperCoord<N>, Vec<T>>,
    meta: HyperSurfaceMeta,
}

impl<const N: usize, T> HyperSurface<N, T> {
    pub fn new(meta: HyperSurfaceMeta) -> Self
    where
        T: Default + Clone,
    {
        let mut planes = HashMap::new();

        for plane_id in meta.all_planes() {
            let max_idx = set_hypercoord_inbound_vals(plane_id, meta.side_len);
            let max_flat_idx = meta.index_dense(max_idx).unwrap();
            let arr = vec![T::default(); max_flat_idx];
            planes.insert(plane_id, arr);
        }

        Self { planes, meta }
    }

    pub fn meta(&self) -> HyperSurfaceMeta {
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

impl HyperSurfaceMeta {
    pub fn new(side_len: usize, max_dim: usize) -> Self {
        Self { max_dim, side_len }
    }

    pub fn index_dense<const N: usize>(&self, c: HyperCoord<N>) -> Option<usize> {
        let mut index = 0;
        let mut stride = 1;
        let mut count = 0;

        for p in c {
            match p {
                Extent::InBound(v) => {
                    index += stride * v;
                    stride *= self.side_len;
                    count += 1;
                }
                _ => (),
            }
        }

        (0..=self.max_dim).contains(&count).then(|| index)
    }

    pub fn all_planes<const N: usize>(&self) -> Vec<HyperCoord<N>> {
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

    pub fn dense_coords<const N: usize>(&self) -> Vec<HyperCoord<N>> {
        let mut output = vec![];
        for plane in self.all_planes() {
            self.dense_coords_rec(&mut output, N - 1, plane);
        }
        output
    }

    pub fn dense_coords_rec<const N: usize>(
        &self,
        out: &mut Vec<HyperCoord<N>>,
        idx: usize,
        mut plane: HyperCoord<N>,
    ) {
        match plane[idx] {
            Extent::InBound(_) => {
                for pos in 1..self.side_len - 1 {
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

    pub fn coord_euclid<const N: usize>(&self, coord: HyperCoord<N>) -> [usize; N] {
        coord.map(|v| match v {
            Extent::Positive => self.side_len - 1,
            Extent::Negative => 0,
            Extent::InBound(v) => v,
        })
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
