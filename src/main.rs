use hypersurface::HyperSurfaceMeta;

fn main() {
    let meta = HyperSurfaceMeta::<3>::new(1, 3);

    let mut count = 0;
    for coord in meta.dense_coords() {
        println!("{:?}", meta.coord_euclid(coord));
        for neighbor in meta.neighbors(coord) {
            println!("    {:?}", meta.coord_euclid(neighbor));
        }
        count += 1;
    }

    dbg!(count);

    /*
    for plane in meta.all_planes::<N_DIMS>() {
        println!("{:?}", plane);
    }
    */
}
