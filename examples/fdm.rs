use hypersurface::{HyperSurfaceMeta, NeighborAccel};
use idek::{nalgebra::{Matrix4, Vector4, Vector3}, prelude::*, IndexBuffer, MultiPlatformCamera};

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
        let meta = HyperSurfaceMeta::new(100, 2);
        let mut sim = Simulation::new(meta);

        let mut rng = Rng::new();
        //let k = 3999;//meta.side_len().pow(meta.max_dim() as u32);
        for i in 0..sim.data().len() {
            let rand = rng.gen() & 1 == 0;
            sim.data_mut()[i] = rand;
        }

        let vertices = draw_surface4(meta, sim.data(), Matrix4::identity());
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
        let a = ctx.start_time().elapsed().as_secs_f32() / 10.;

        //let matrix = Matrix4::new_rotation(Vector3::new(a, 0., 0.));

        let matrix = Matrix4::from_column_slice(&[
            a.cos(), 0., 0., a.sin(), 
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            -a.sin(), 0., 0., a.cos(), 
        ]);

        if self.voot % 1 == 0 {
            self.sim.step();

            let vertices = draw_surface4(self.meta, self.sim.data(), matrix);
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

fn color_fn(v: bool, c: [f32; 4]) -> [f32; 3] {
    if v {
        [
            c[0] - c[3],
            c[1] + c[3],
            c[2] - c[3],
        ]
    } else {
        [0.01; 3]
    }
}

fn draw_surface4(meta: HyperSurfaceMeta<4>, data: &[bool], matrix: Matrix4<f32>) -> Vec<Vertex> {
    let side_len = meta.side_len() as f32;

    meta.all_coords()
        .into_iter()
        .zip(data)
        .map(|(coord, val)| {
            let point = meta
                .coord_euclid(coord)
                .map(|v| v as f32 / side_len);

            let vect = Vector4::from(point);
            let vect = vect * 2. - Vector4::from([1.; 4]);

            let vect = matrix * vect;

            let q = vect.w + 2.;
            let pos = [
                vect.x * q,
                vect.y * q,
                vect.z * q,
            ];

            Vertex::new(pos, color_fn(*val, point))
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

/// https://en.wikipedia.org/wiki/Permuted_congruential_generator
pub struct Rng {
    state: u64,
    multiplier: u64,
    increment: u64,
}

impl Rng {
    pub fn new() -> Self {
        Self::from_seed(
            5573589319906701683,
            6364136223846793005,
            1442695040888963407,
        )
    }

    pub fn from_seed(seed: u64, multiplier: u64, increment: u64) -> Self {
        Self {
            state: seed + increment,
            multiplier,
            increment,
        }
    }

    fn u64_to_u32(x: u64) -> u32 {
        let bytes = x.to_le_bytes();
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[2]])
    }

    fn rotr32(x: u32, r: u32) -> u32 {
        x >> r | x << (r.wrapping_neg() & 31)
    }

    pub fn gen(&mut self) -> u32 {
        let mut x = self.state;
        let count = x >> 59;
        self.state = x.wrapping_mul(self.multiplier).wrapping_add(self.increment);
        x ^= x >> 18;
        Self::rotr32(Self::u64_to_u32(x >> 27), Self::u64_to_u32(count))
    }
}
