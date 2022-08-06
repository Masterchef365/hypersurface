use hypersurface::ArrayNd;

fn main() {
    for arrange in n_choose_m::<3>(5, 3) {
        let mut v = [0; 5];
        for k in arrange {
            v[k] = 1;
        }

        println!("{:?}", v);
    }
}

/// Output with the given const size, but use the given value of m
pub fn n_choose_m<const D: usize>(n: usize, m: usize) -> Vec<[usize; D]> {
    assert!(m <= D);

    let m_minus_one = match m.checked_sub(1) {
        Some(mmo) => mmo,
        None => return vec![[0; D]],
    };

    let mut out = vec![];

    for i in m_minus_one..n {
        for mut sub in n_choose_m(i, m_minus_one) {
            sub[m_minus_one] = i;
            out.push(sub);
        }
    }
    out
}

