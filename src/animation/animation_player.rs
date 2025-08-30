use std::{collections::HashMap, time::Instant};

use cgmath::SquareMatrix;

use crate::{animation::keyframes::{interpolate_position, interpolate_rotation, interpolate_scale}, model::{AnimationCip, Channel}};

pub struct AnimationPlayer {
    current_time: Instant,
    final_node_matrices: HashMap<usize, cgmath::Matrix4<f32>>,
    local_node_matrices: HashMap<usize, cgmath::Matrix4<f32>>,
    current_animation: Box<AnimationCip>
}

impl AnimationPlayer {
    fn new(animation: Box<AnimationCip>) -> Self {
        let current_time = Instant::now();
        let final_node_matrices: HashMap<usize, cgmath::Matrix4<f32>> = HashMap::new();
        let local_node_matrices: HashMap<usize, cgmath::Matrix4<f32>> = HashMap::new();
        Self {
          current_animation: animation,
          final_node_matrices,
          local_node_matrices,
          current_time
        }
    }

    fn update_animation(&mut self) {
        if self.current_animation.channels.len() > 0 { 
            let duration = self.current_animation.channels.iter().filter_map(|c| c.timestamps.last()).copied().fold(0.0_f32, |a, b| a.max(b)); 
            let mut current_time = self.current_time.elapsed().as_secs_f32();

            // loop time
            if duration > 0.0 {
              current_time = current_time % duration;
            }

            for channel in &self.current_animation.channels {
                 let node_index = channel.node_index;
                 let mut local_matrix = cgmath::Matrix4::identity();

                // sample translation
                if let Some(t) = interpolate_position(channel, current_time) {
                   local_matrix = local_matrix * cgmath::Matrix4::from_translation(cgmath::Vector3::from(t));
                }

                // sample rotation
                if let Some(r) = interpolate_rotation(channel, current_time) {
                   local_matrix = local_matrix * cgmath::Matrix4::from(cgmath::Quaternion::from(r));
                }

                // sample scale
                if let Some(s) = interpolate_scale(channel, current_time) {
                // local_matrix = local_matrix * cgmath::Matrix4::from_nonuniform_scale(s[0], s[1], s[2]);
                }

                self.local_node_matrices.insert(node_index, local_matrix);

                
            }
        }
    }

// fn compute_global(
//     node_index: usize,
//     parent_matrix: cgmath::Matrix4<f32>,
//     nodes: &Vec<Node>,
//     local: &HashMap<usize, cgmath::Matrix4<f32>>,
//     global: &mut HashMap<usize, cgmath::Matrix4<f32>>,
// ) {
//     // fallback if node has no animation: identity
//     let local_matrix = local.get(&node_index).cloned().unwrap_or(cgmath::Matrix4::identity());

//     let global_matrix = parent_matrix * local_matrix;
//     global.insert(node_index, global_matrix);

//     // recurse into children
//     for &child in &nodes[node_index].children {
//         Self::compute_global(child, global_matrix, nodes, local, global);
//     }
// }
}