use hypersurface::HyperSurfaceMeta;

fn main() {
    let meta = HyperSurfaceMeta::new(1, 2);
    const N_DIMS: usize = 3;

    let mut count = 0;
    for coord in meta.dense_coords::<N_DIMS>() {
        println!("{:?}", meta.coord_euclid(coord));
        for neighbor in meta.neighbors(coord) {
            println!("    {:?}", meta.coord_euclid(neighbor));
        }
        count += 1;
    }

    /*
    dbg!(count);

    for plane in meta.all_planes::<N_DIMS>() {
        println!("{:?}", plane);
    }
    */
}
