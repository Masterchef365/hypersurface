use hypersurface::HyperSurfaceMeta;

fn main() {
    let meta = HyperSurfaceMeta::new(8, 1);
    const N_DIMS: usize = 3;

    let mut count = 0;
    for point in meta.dense_coords::<N_DIMS>() {
        //println!("{:?}", point);
        println!("{:?}", meta.coord_euclid(point));
        count += 1;
    }

    dbg!(count);

    for plane in meta.all_planes::<N_DIMS>() {
        println!("{:?}", plane);
    }
}
