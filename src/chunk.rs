use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub const CHUNK_WIDTH: usize = 4;
pub const CHUNK_DEPTH: usize = 4;
pub const CHUNK_HEIGHT: usize = 4;
const BLOCK_SIZE: f32 = 1.0;

const FRONT_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
const BACK_COLOR: [f32; 3] = [0.4, 1.0, 1.0];
const RIGHT_COLOR: [f32; 3] = [1.0, 0.53, 0.43]; //orang
const LEFT_COLOR: [f32; 3] = [0.82, 0.32, 1.0];
const TOP_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
const BOTTOM_COLOR: [f32; 3] = [0.0, 0.0, 0.0];

pub const MAX_VOXEL_COUNT_PER_CHUNK: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;
pub const VERTEX_PER_VOXEL: usize = 36;
pub const MAX_VERTEX_PER_CHUNK: usize = VERTEX_PER_VOXEL * MAX_VOXEL_COUNT_PER_CHUNK;
pub const MAX_INDEX_COUNT_PER_CHUNK: usize = MAX_VERTEX_PER_CHUNK;

pub struct ChunkMeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    num_of_faces: u32,
    chunk_data: Vec<u8>,
}

//might be wrong (oopsies)
fn to_1d_array(x: usize, y: usize, z: usize) -> usize {
    y + x * CHUNK_HEIGHT + (z * CHUNK_WIDTH * CHUNK_HEIGHT)
}

impl ChunkMeshData {
    pub fn new() -> Self {
        let chunk_data = vec![0; MAX_VOXEL_COUNT_PER_CHUNK];
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            num_of_faces: 0,
            chunk_data,
        }
    }

    fn generate_mesh_face_data(&mut self, x: usize, y: usize, z: usize, face_type: FaceType) {
        let vertex_face = generate_voxel_face(x as f32, y as f32, z as f32, face_type);
        for vertex in vertex_face.iter() {
            self.vertices.push(*vertex);
        }
        let indices = generate_index_for_face(self.num_of_faces);
        for index in indices.iter() {
            self.indices.push(*index);
        }
        self.num_of_faces += 1;
    }

    pub fn generate_mesh(&mut self) {
        //generating at which position there is a voxel
        for z in 0..CHUNK_WIDTH {
            for x in 0..CHUNK_DEPTH {
                for y in 0..CHUNK_HEIGHT {
                    //println!("index {}", to_1d_array(x, z, y));
                    self.chunk_data[to_1d_array(x, y, z)] = 1;
                }
            }
        }

        //generating mesh data for visible faces
        for z in 0..CHUNK_DEPTH {
            for x in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
                    //println!("x {} y {} z {}", x, y, z);
                    //println!("{}", to_1d_array(x + 1, y, z));
                    if z == CHUNK_DEPTH - 1 || self.chunk_data[to_1d_array(x, y, z + 1)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Front);
                    }
                    if z == 0 || self.chunk_data[to_1d_array(x, y, z - 1)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Back);
                    }
                    if x == CHUNK_WIDTH - 1 || self.chunk_data[to_1d_array(x + 1, y, z)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Right);
                    }
                    if x == 0 || self.chunk_data[to_1d_array(x - 1, y, z)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Left);
                    }
                    if y == CHUNK_HEIGHT - 1 || self.chunk_data[to_1d_array(x, y + 1, z)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Top);
                    }
                    if y == 0 || self.chunk_data[to_1d_array(x, y - 1, z)] != 1{
                        self.generate_mesh_face_data(x, y, z, FaceType::Bottom);
                    }
                }
            }
        }
        println!("Number of faces {}", self.num_of_faces);
    }

    pub fn build(&mut self, device: &wgpu::Device) -> (StagingBuffer, StagingBuffer, u32) {
        let vertex_buffer = StagingBuffer::new(device, &self.vertices, false);
        let index_buffer = StagingBuffer::new(device, &self.indices, true);
        let num_of_indices = self.indices.len() as u32;
        (vertex_buffer, index_buffer, num_of_indices)
    }
}

pub struct StagingBuffer {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl StagingBuffer {
    pub fn new<T: bytemuck::Pod + Sized>(
        device: &wgpu::Device,
        data: &[T],
        is_index_buffer: bool,
    ) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::COPY_SRC
                    | if is_index_buffer {
                        wgpu::BufferUsages::INDEX
                    } else {
                        wgpu::BufferUsages::empty()
                    },
                label: Some("Staging buffer"),
            }),
            size: size_of_slice(data) as wgpu::BufferAddress,
        }
    }

    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, other: &wgpu::Buffer) {
        encoder.copy_buffer_to_buffer(&self.buffer, 0, other, 0, self.size)
    }
}

fn generate_index_for_face(face_count: u32) -> [u32; 6] {
    let offset = face_count * 4;
    let v1 = offset + 0;
    let v2 = offset + 1;
    let v3 = offset + 3;
    let v4 = offset + 3;
    let v5 = offset + 2;
    let v6 = offset + 0;

    [v1, v2, v3, v4, v5, v6]
}

enum FaceType {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

fn generate_voxel_face(x: f32, y: f32, z: f32, face_type: FaceType) -> [Vertex; 4] {
    match face_type {
        FaceType::Front => {
            let v1 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: FRONT_COLOR,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: FRONT_COLOR,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: FRONT_COLOR,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: FRONT_COLOR,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Back => {
            let v1 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: BACK_COLOR,
            };
            let v2 = Vertex {
                position: [x, y, z],
                color: BACK_COLOR,
            };
            let v3 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: BACK_COLOR,
            };
            let v4 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: BACK_COLOR,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Right => {
            let v1 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: RIGHT_COLOR,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: RIGHT_COLOR,
            };
            let v3 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: RIGHT_COLOR,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: RIGHT_COLOR,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Left => {
            let v1 = Vertex {
                position: [x, y, z],
                color: LEFT_COLOR,
            };
            let v2 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: LEFT_COLOR,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: LEFT_COLOR,
            };
            let v4 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: LEFT_COLOR,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Bottom => {
            let v1 = Vertex {
                position: [x, y, z],
                color: BOTTOM_COLOR,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: BOTTOM_COLOR,
            };
            let v3 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: BOTTOM_COLOR,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: BOTTOM_COLOR,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Top => {
            let v1 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: TOP_COLOR,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: TOP_COLOR,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: TOP_COLOR,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: TOP_COLOR,
            };
            [v1, v2, v3, v4]
        }
    }
}

fn size_of_slice<T: Sized>(slice: &[T]) -> usize {
    std::mem::size_of::<T>() * slice.len()
}
pub const U32_SIZE: wgpu::BufferAddress = std::mem::size_of::<u32>() as wgpu::BufferAddress;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    pub fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
