use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
   pub position: [f32; 3],
   pub tex_coords: [f32; 2],
   pub normal: [f32; 3],
   pub tangent: [f32; 3],
   pub bitangent: [f32; 3],
   pub joints: [u16; 4],
   pub weights: [f32; 4]
}

impl Vertex {
   const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
      0 => Float32x3, // position
      1 => Float32x2, // tex coords
      2 => Float32x3, // normals
      3 => Float32x3, // tangents
      4 => Float32x3, // bitangents
      5 => Uint16x4, // joints
      6 => Float32x4 // weights
   ];
   
   pub fn desc() -> wgpu::VertexBufferLayout<'static> {
       wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex,                         
            attributes: &Self::ATTRIBUTES
        }
   }

   pub fn calc_tan_vectors(vertices: &mut Vec<Vertex>, indices: &Vec<u32>) {
        let mut triangles_included = vec![0; vertices.len()];

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0: cgmath::Vector3<_> = v0.position.into();
            let pos1: cgmath::Vector3<_> = v1.position.into();
            let pos2: cgmath::Vector3<_> = v2.position.into();

            let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
            let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
            let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

            // Calculate the edges of the triangle
            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            // This will give us a direction to calculate the
            // tangent and bitangent
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            // Solving the following system of equations will
            // give us the tangent and bitangent.
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            // Luckily, the place I found this equation provided
            // the solution!
            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            // We flip the bitangent to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            // We'll use the same tangent/bitangent for each vertex in the triangle
            vertices[c[0] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            vertices[c[1] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            vertices[c[2] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
            vertices[c[0] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

            // Used to average the tangents/bitangents
            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

         for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let mut v = &mut vertices[i];
            v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
            v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
        }

   }

//    pub fn gen_list(vertices: &[f32], vertices_count: i64) {
//      let stride = 8;

//      let ret = Vec::with_capacity(vertices_count);

//      for i in 0..vertices_count {
//          let base = i * stride;
//             let v = Vertex {
//                 position: [
//                     vertices[base + 0],
//                     vertices[base + 1],
//                     vertices[base + 2],
//                 ],
//                 normal: [
//                     vertices[base + 3],
//                     vertices[base + 4],
//                     vertices[base + 5],
//                 ],
//                 tex_coords: [
//                     vertices[base + 6],
//                     vertices[base + 7],
//                 ],

//             };
//             ret.push(v);
//      }
//    }
}