use hypersurface::{HyperSurfaceMeta, NeighborAccel};
use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    shader: Shader,

    camera: MultiPlatformCamera,

    voot: usize,
    sim: Simulation<4>,
    meta: HyperSurfaceMeta<4>,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let meta = HyperSurfaceMeta::new(20, 2);
        let mut sim = Simulation::new(meta);

        let l = sim.data().len();
        let k = 900;
        for i in l - k..l {
            let rand = (i as f32 / l as f32).cos().to_be_bytes()[3] & 1 == 0;
            sim.data_mut()[i] = rand;
        }

        let vertices = draw_surface4(meta, sim.data());
        let indices = linear_indices(&vertices);

        Ok(Self {
            voot: 0,
            meta,
            sim,
            verts: ctx.vertices(&vertices, true)?,
            indices: ctx.indices(&indices, false)?,
            shader: ctx.shader(
                include_bytes!("shaders/unlit.vert.spv"),
                DEFAULT_FRAGMENT_SHADER,
                Primitive::Points,
            )?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        if self.voot % 9 == 0 {
            self.sim.step();

            let vertices = draw_surface4(self.meta, self.sim.data());
            ctx.update_vertices(self.verts, &vertices)?;
            self.voot = 0;
        }

        self.voot += 1;

        Ok(vec![DrawCmd::new(self.verts)
            .indices(self.indices)
            .shader(self.shader)])
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        platform: &mut Platform,
        mut event: Event,
    ) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix())
        }
        idek::close_when_asked(platform, &event);
        Ok(())
    }
}

struct Simulation<const N: usize> {
    accel: NeighborAccel,
    write: Vec<bool>,
    read: Vec<bool>,
    first: bool,
}

impl<const N: usize> Simulation<N> {
    pub fn new(meta: HyperSurfaceMeta<N>) -> Self {
        let accel = NeighborAccel::new(meta);
        Self {
            write: vec![false; accel.len()],
            read: vec![false; accel.len()],
            accel,
            first: true,
        }
    }

    pub fn data_mut(&mut self) -> &mut [bool] {
        &mut self.read
    }

    pub fn data(&mut self) -> &[bool] {
        &self.read
    }

    pub fn step(&mut self) {
        self.accel.neighbors(|coord, neighbors| {
            let center = self.read[coord];
            let sum: u8 = neighbors.iter().map(|&c| self.read[c] as u8).sum();

            self.write[coord] = match center {
                true => matches!(sum, 2 | 3),
                false => sum == 3,
            };
        });

        self.first = false;

        std::mem::swap(&mut self.read, &mut self.write);
    }
}

fn color_fn(v: bool) -> [f32; 3] {
    if v {
        [1.; 3]
    } else {
        [0.01; 3]
    }
}

fn draw_surface4(meta: HyperSurfaceMeta<4>, data: &[bool]) -> Vec<Vertex> {
    let side_len = meta.side_len() as f32;

    meta.all_coords()
        .into_iter()
        .zip(data)
        .map(|(coord, val)| {
            let point = meta
                .coord_euclid(coord)
                .map(|v| v as f32 / side_len)
                .map(|v| v * 2. - 1.);

            let q = point[3] + 2.;

            let pos = [point[0] * q, point[1] * q, point[2] * q];

            Vertex::new(pos, color_fn(*val))
        })
        .collect()
}

/*
fn draw_surface3(surface: &HyperSurface<3, f32>) -> Vec<Vertex> {
    let side_len = surface.meta().side_len() as f32;

    let coord_to_vert = |coord| {
        let point = surface
            .meta()
            .coord_euclid(coord)
            .map(|v| v as f32 / side_len)
            .map(|v| v * 2. - 1.);

        let val = surface[coord];
        Vertex::new(point, color_fn(val))
    };

    surface
        .meta()
        .all_coords()
        .into_iter()
        .map(coord_to_vert)
        .collect()
}
*/

fn linear_indices(v: &[Vertex]) -> Vec<u32> {
    (0..v.len() as u32).collect()
}
