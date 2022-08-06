use hypersurface::HyperSurfaceMeta;

fn main() {
    let meta = HyperSurfaceMeta::new(3, 1);

    let mut count = 0;
    for point in meta.all_coords::<3>() {
        println!("{:?}", point);
        count += 1;
    }

    dbg!(count);
}
