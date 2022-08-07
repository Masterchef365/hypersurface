use hypersurface::{Extent, HyperCoord, HyperSurface, HyperSurfaceMeta, NeighborAccel};
use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    shader: Shader,

    camera: MultiPlatformCamera,

    sim: Simulation<4>,
    meta: HyperSurfaceMeta<4>,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let meta = HyperSurfaceMeta::new(150, 2);
        let mut sim = Simulation::new(meta);

        sim.data_mut()[0] = 100.;

        let vertices = draw_surface4(meta, sim.data());
        let indices = linear_indices(&vertices);

        Ok(Self {
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
        let vertices = draw_surface4(self.meta, self.sim.data());
        ctx.update_vertices(self.verts, &vertices)?;

        self.sim.step(1e-1);

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
    write: Vec<f32>,
    read: Vec<f32>,
    prev: Vec<f32>,
    first: bool,
}

impl<const N: usize> Simulation<N> {
    pub fn new(meta: HyperSurfaceMeta<N>) -> Self {
        let accel = NeighborAccel::new(meta);
        Self {
            write: vec![0.; accel.len()],
            read: vec![0.; accel.len()],
            prev: vec![0.; accel.len()],
            accel,
            first: true,
        }
    }

    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.read
    }

    pub fn data(&mut self) -> &[f32] {
        &self.read
    }

    pub fn step(&mut self, dt: f32) {
        self.accel.neighbors(|coord, neighbors| {
            let prev = self.prev[coord];
            let center = self.read[coord];
            let sum: f32 = neighbors.iter().map(|&c| self.read[c]).sum();

            let cfd = sum - neighbors.len() as f32 * center;
            let cfd = 0.5 * dt * cfd;

            self.write[coord] = if self.first {
                center - cfd
            } else {
                -prev + 2. * center + cfd
            };
        });

        self.first = false;

        std::mem::swap(&mut self.read, &mut self.prev);
        std::mem::swap(&mut self.read, &mut self.write);
    }
}

fn color_fn(v: f32) -> [f32; 3] {
    if v > 0. {
        [v, v * 0.2, v * 0.01]
    } else {
        [-v * 0.01, -v * 0.2, -v]
    }
}

fn draw_surface4(meta: HyperSurfaceMeta<4>, data: &[f32]) -> Vec<Vertex> {
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
