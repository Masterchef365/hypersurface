use hypersurface::ArrayNd;

fn main() {
    let mut nd = ArrayNd::new([3, 4, 5]);
    nd[[1, 2, 3]] = 5_i32;

    dbg!(nd.data());
}
