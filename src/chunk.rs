use wgpu::util::{DeviceExt, BufferInitDescriptor};

pub const CHUNK_WIDTH: u64 = 16;
pub const CHUNK_DEPTH: u64 = 16;
pub const CHUNK_HEIGHT: u64 = 16;
const BLOCK_SIZE: f32 = 1.0;
const BLOCK_COLOR: [f32; 3] = [0.3, 1.0, 0.5];

pub const VERTEX_PER_VOXEL: u64 = 36;
pub const MAX_VERTEX_PER_CHUNK: u64 = VERTEX_PER_VOXEL * CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

pub struct ChunkData{
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl ChunkData{
    pub fn new() -> Self{
        Self{
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
    pub fn generate_mesh(&mut self){
        for x in 0..CHUNK_WIDTH{
            for z in 0..CHUNK_DEPTH{
                for y in 0..CHUNK_HEIGHT{
                    let (vertices, indices) = generate_voxel(x as f32, y as f32, z as f32);   
                    for vertex in vertices.iter(){
                        self.vertices.push(*vertex);
                    }
                    for index in indices.iter(){
                        self.indices.push(*index);
                    }
                }
            }
        }
    }
    
    pub fn build(&mut self, device: &wgpu::Device) -> (StagingBuffer, StagingBuffer, u32){
        let vertex_buffer = StagingBuffer::new(device, &self.vertices, false);
        let index_buffer = StagingBuffer::new(device, &self.indices, true);
        let num_of_indices = self.indices.len() as u32;
        (vertex_buffer, index_buffer, num_of_indices)
    }
}

pub struct StagingBuffer{
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl StagingBuffer{
    pub fn new<T: bytemuck::Pod + Sized>(
        device: &wgpu::Device,
        data: &[T],
        is_index_buffer: bool,
    ) -> Self{
        Self{
            buffer: device.create_buffer_init(&BufferInitDescriptor{
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::COPY_SRC |
                if is_index_buffer{
                    wgpu::BufferUsages::INDEX       
                }
                else{
                    wgpu::BufferUsages::empty()        
                },
                label: Some("Staging buffer"),
            }),
            size: size_of_slice(data) as wgpu::BufferAddress,
        }      
    }
    
    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, other: &wgpu::Buffer){
        encoder.copy_buffer_to_buffer(&self.buffer, 0, other, 0, self.size)
    }
}

fn generate_voxel(x: f32, y: f32, z: f32) -> ([Vertex; 24], [u16; 36]){
    let vertices: [Vertex; 24] = [ 
    Vertex {
        position: [x, y, z + BLOCK_SIZE],
        color: BLOCK_COLOR,
    },
    Vertex {
        position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    //back
    Vertex {
        position: [x + BLOCK_SIZE, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
    //right
    Vertex {
        position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
    //left
    Vertex {
        position: [x, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    //bottom
    Vertex {
        position: [x, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    //top
    Vertex {
        position: [x, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z + BLOCK_SIZE],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
    Vertex {
        position: [x + BLOCK_SIZE, y + BLOCK_SIZE, z],
        color: [0.3, 1.0, 0.5],
    },
];

    let indices: [u16; 36] = [2, 1, 3, 2, 0, 1,
                            6, 5, 7, 6, 4, 5,
                            10, 9, 11, 10, 8, 9,
                            14, 13, 15, 14, 12, 13,
                            18, 17, 19, 18, 16, 17,
                            22, 21, 23, 22, 20, 21,
    ];
    (vertices, indices)
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
    pub fn size() -> wgpu::BufferAddress{
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
