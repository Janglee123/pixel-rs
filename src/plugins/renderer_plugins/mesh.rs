use wgpu::Buffer;

use crate::plugins::core::render_plugin::Gpu;

use super::vertex::Vertex;
use std::sync::Arc;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Arc<[Vertex]>,
    pub indices: Arc<[u16]>,
}

impl Mesh {
    pub fn get_quad_mesh() -> Self {
        Self {
            vertices: Arc::new([
                Vertex {
                    position: [0.5, 0.5, 1.0],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [-0.5, 0.5, 1.0],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 1.0],
                    color: [1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [0.5, -0.5, 1.0],
                    color: [1.0, 1.0, 1.0],
                },
            ]),
            indices: Arc::new([0, 1, 2, 0, 2, 3]),
        }
    }

    pub fn get_hex_mesh() -> Self {
        Self {
            vertices: Arc::new([
                Vertex {
                    position: [0.0, 0.5, 1.0],
                    color: [1.0; 3],
                },
                Vertex {
                    position: [0.433012702, 0.25, 1.0],
                    color: [1.0; 3],
                },
                Vertex {
                    position: [0.433012702, -0.25, 1.0],
                    color: [1.0; 3],
                },
                Vertex {
                    position: [0.0, -0.5, 1.0],
                    color: [1.0; 3],
                },
                Vertex {
                    position: [-0.433012702, -0.25, 1.0],
                    color: [1.0; 3],
                },
                Vertex {
                    position: [-0.433012702, 0.25, 1.0],
                    color: [1.0; 3],
                },
            ]),
            indices: Arc::new([0, 2, 1, 2, 4, 3, 0, 5, 4, 0, 4, 2]),
        }
    }
}
