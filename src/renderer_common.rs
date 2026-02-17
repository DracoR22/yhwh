use crate::vertex::Vertex;

pub const CUBE_VERTICES: &[Vertex] = &[
    // Front face
    Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0,  0.0,  1.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [0.0,  0.0,  1.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },

    // Back face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0,  0.0, -1.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0,  0.0, -1.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },

    // Left face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },

    // Right face
    Vertex { position: [0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [0.5,  0.5,  0.5], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [0.5, -0.5,  0.5], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },

    // Top face
    Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5, 0.5,  0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, 0.5,  0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },

    // Bottom face
    Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0], bitangent: [0.0; 3], tangent: [0.0; 3], joints: [0; 4], weights: [0.0; 4] },
];


pub const CUBE_INDICES: &[u32] = &[
    0, 1, 2, 0, 2, 3,       // front
    4, 5, 6, 4, 6, 7,       // back
    8, 9, 10, 8, 10, 11,    // left
    12, 13, 14, 12, 14, 15, // right
    16, 17, 18, 16, 18, 19, // top
    20, 21, 22, 20, 22, 23, // bottom
];


pub const PLANE_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.0, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], bitangent: [1.0,0.0,0.0], tangent: [1.0,0.0,0.0], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, 0.0, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0,0.0], bitangent: [0.0;3], tangent: [0.0;3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [ 0.5, 0.0,  0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0,1.0], bitangent: [0.0;3], tangent: [0.0;3], joints: [0; 4], weights: [0.0; 4] },
    Vertex { position: [-0.5, 0.0,  0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0,1.0], bitangent: [0.0;3], tangent: [0.0;3], joints: [0; 4], weights: [0.0; 4] },
];

pub const PLANE_INDICES: &[u32] = &[
    0, 1, 2,
    0, 2, 3, 
];

pub const SKYBOX_VERTICES: &[f32] = &[
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,

     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,

    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,

    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0
];

pub const QUAD_VERTICES: &[f32] = &[
    -1.0, -1.0,   0.0, 0.0,
     1.0, -1.0,   1.0, 0.0,
     1.0,  1.0,   1.0, 1.0,

    -1.0, -1.0,   0.0, 0.0,
     1.0,  1.0,   1.0, 1.0,
    -1.0,  1.0,   0.0, 1.0,
];

