use wgpu::util::DeviceExt;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 16;
const BLOCK_SIZE: f32 = 1.0;

const FRONT_COLOR: [f32; 3] = [0.4, 1.0, 0.52];

/*
const BACK_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
const RIGHT_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
const LEFT_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
const TOP_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
const BOTTOM_COLOR: [f32; 3] = [0.4, 1.0, 0.52];
*/

const BACK_COLOR: [f32; 3] = [0.4, 1.0, 1.0];
const RIGHT_COLOR: [f32; 3] = [1.0, 0.53, 0.43]; //orang
const LEFT_COLOR: [f32; 3] = [0.82, 0.32, 1.0];
const TOP_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
const BOTTOM_COLOR: [f32; 3] = [0.0, 0.0, 0.0];

const MAX_VOXEL_COUNT_PER_CHUNK: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;
//const VERTEX_PER_VOXEL: usize = 36;
//const MAX_VERTEX_PER_CHUNK: usize = VERTEX_PER_VOXEL * MAX_VOXEL_COUNT_PER_CHUNK;
//const MAX_INDEX_COUNT_PER_CHUNK: usize = MAX_VERTEX_PER_CHUNK;

pub struct ChunkMeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    num_of_faces: u32,
    pub chunk_data: Vec<u8>, //storing local coordinates
    world_coordinates: cgmath::Vector3<usize>,
}

//might be wrong (oopsies)
fn to_1d_array(x: usize, y: usize, z: usize) -> usize {
    y + x * CHUNK_HEIGHT + (z * CHUNK_WIDTH * CHUNK_HEIGHT)
}

impl ChunkMeshData {
    pub fn new(world_coordinates: cgmath::Vector3<usize>) -> Self {
        let chunk_data = vec![0; MAX_VOXEL_COUNT_PER_CHUNK];

        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            num_of_faces: 0,
            chunk_data,
            world_coordinates,
        }
    }

    fn generate_mesh_face_data(&mut self, x: usize, y: usize, z: usize, face_type: FaceType) {
        let vertex_face = generate_voxel_face(
            x as f32,
            y as f32,
            z as f32,
            &self.world_coordinates,
            face_type,
        );
        for vertex in vertex_face.iter() {
            self.vertices.push(*vertex);
        }
        let indices = generate_index_for_face(self.num_of_faces);
        for index in indices.iter() {
            self.indices.push(*index);
        }
        self.num_of_faces += 1;
    }

    pub fn generate_data(&mut self) {
        for y in 0..CHUNK_HEIGHT {
            for z in y..CHUNK_DEPTH {
                for x in 0..CHUNK_WIDTH {
                    self.chunk_data[to_1d_array(x, y, z)] = 1;
                }
            }
        }
    }

    pub fn generate_mesh(&mut self, _left_chunk_data: &Vec<u8>) -> u32 {
        for y in 0..CHUNK_HEIGHT {
            for z in y..CHUNK_DEPTH {
                for x in 0..CHUNK_WIDTH {
                    if z == CHUNK_DEPTH - 1 || self.chunk_data[to_1d_array(x, y, z + 1)] != 1 {
                        self.generate_mesh_face_data(x, y, z, FaceType::Front);
                    }
                    if z == 0 || self.chunk_data[to_1d_array(x, y, z - 1)] != 1 {
                        self.generate_mesh_face_data(x, y, z, FaceType::Back);
                    }
                    if x == CHUNK_WIDTH - 1 || self.chunk_data[to_1d_array(x + 1, y, z)] != 1 {
                        self.generate_mesh_face_data(x, y, z, FaceType::Right);
                    }
                    //firstly checking neighbouring blocks in this chunk
                    //(we have to have x == 0 so the second statement won't assert cuz x would be -1 in the argument to the function)
                    if x == 0 || self.chunk_data[to_1d_array(x - 1, y, z)] != 1 {
                        /*
                        //then checking neighbouring blocks in the neighbouring chunks
                        let mut num = 0;
                        if x == 0{
                            let index = to_1d_array(CHUNK_WIDTH - 1, y, z);
                            num = left_chunk_data[index];
                        }
                        //println!("num {}", num);
                        if num != 1{
                            */
                        self.generate_mesh_face_data(x, y, z, FaceType::Left);
                        //}
                    }
                    if y == CHUNK_HEIGHT - 1 || self.chunk_data[to_1d_array(x, y + 1, z)] != 1 {
                        self.generate_mesh_face_data(x, y, z, FaceType::Top);
                    }
                    if y == 0 || self.chunk_data[to_1d_array(x, y - 1, z)] != 1 {
                        self.generate_mesh_face_data(x, y, z, FaceType::Bottom);
                    }
                }
            }
        }
        self.num_of_faces
        //println!("Number of faces {}", self.num_of_faces);
    }

    pub fn build(&mut self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        (
            vertex_buffer,
            index_buffer,
            self.indices.len().try_into().unwrap(),
        )
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

fn generate_voxel_face(
    x: f32,
    y: f32,
    z: f32,
    world_coordinates: &cgmath::Vector3<usize>,
    face_type: FaceType,
) -> [Vertex; 4] {
    let x = x + world_coordinates.x as f32;
    let z = z + world_coordinates.z as f32;

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let color_index = rng.gen_range(0..6);
    let mut color = [0.0, 0.0, 0.0];
    if color_index == 0 {
        color = FRONT_COLOR;
    } else if color_index == 1 {
        color = BACK_COLOR;
    } else if color_index == 2 {
        color = RIGHT_COLOR;
    } else if color_index == 3 {
        color = LEFT_COLOR;
    } else if color_index == 4 {
        color = BOTTOM_COLOR;
    } else if color_index == 5 {
        color = TOP_COLOR;
    }

    match face_type {
        FaceType::Front => {
            let v1 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: color,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: color,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Back => {
            let v1 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: color,
            };
            let v2 = Vertex {
                position: [x, y, z],
                color: color,
            };
            let v3 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: color,
            };
            let v4 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: color,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Right => {
            let v1 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: color,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: color,
            };
            let v3 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: color,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Left => {
            let v1 = Vertex {
                position: [x, y, z],
                color: color,
            };
            let v2 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: color,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: color,
            };
            let v4 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Bottom => {
            let v1 = Vertex {
                position: [x, y, z],
                color: color,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y, z],
                color: color,
            };
            let v3 = Vertex {
                position: [x, y, z + BLOCK_SIZE],
                color: color,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
                color: color,
            };
            [v1, v2, v3, v4]
        }
        FaceType::Top => {
            let v1 = Vertex {
                position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            let v2 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
                color: color,
            };
            let v3 = Vertex {
                position: [x, y + BLOCK_SIZE, z],
                color: color,
            };
            let v4 = Vertex {
                position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
                color: color,
            };
            [v1, v2, v3, v4]
        }
    }
}

/*
fn size_of_slice<T: Sized>(slice: &[T]) -> usize {
    std::mem::size_of::<T>() * slice.len()
}
*/
//pub const U32_SIZE: wgpu::BufferAddress = std::mem::size_of::<u32>() as wgpu::BufferAddress;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    /*
    pub fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Self>() as wgpu::BufferAddress
    }
    */
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
