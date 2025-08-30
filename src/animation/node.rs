#[derive(Clone, Debug, Default)]
pub struct Nodes {
    nodes: Vec<Node>,
    depth_first_taversal_indices: Vec<(usize, Option<usize>)>,
}

impl Nodes {
    pub fn from_gltf_nodes(gltf_nodes: gltf::iter::Nodes, scene: &gltf::Scene) -> Nodes {
    let roots_indices = scene.nodes().map(|n| n.index()).collect::<Vec<_>>();
    let node_count = gltf_nodes.len();
    let mut nodes = Vec::with_capacity(node_count);

     for node in gltf_nodes {
            let node_index = node.index();
            let local_transform = node.transform();
            let global_transform_matrix = compute_transform_matrix(&local_transform);
            let mesh_index = node.mesh().map(|m| m.index());
            let skin_index = node.skin().map(|s| s.index());
            let children_indices = node.children().map(|c| c.index()).collect::<Vec<_>>();
            let node = Node {
                local_transform,
                global_transform_matrix,
                mesh_index,
                skin_index,
                children_indices,
            };
            nodes.insert(node_index, node);
       }

       let mut nodes: Nodes = Nodes::new(nodes, roots_indices);
       nodes
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }

    fn new(nodes: Vec<Node>, roots_indices: Vec<usize>) -> Self {
        let depth_first_taversal_indices = build_graph_run_indices(&roots_indices, &nodes);
        Self {
            nodes,
            depth_first_taversal_indices,
        }
    }

    pub fn transform(&mut self, global_transform: Option<cgmath::Matrix4<f32>>) {
        for (index, parent_index) in &self.depth_first_taversal_indices {
            let parent_transform = parent_index
                .map(|id| {
                    let parent = &self.nodes[id];
                    parent.global_transform_matrix
                })
                .or(global_transform);

            if let Some(matrix) = parent_transform {
                let node = &mut self.nodes[*index];
                node.apply_transform(matrix);
            }
        }
    }

    pub fn get_skins_transform(&self) -> Vec<(usize, cgmath::Matrix4<f32>)> {
        self.nodes
            .iter()
            .filter(|n| n.skin_index.is_some())
            .map(|n| (n.skin_index.unwrap(), n.transform()))
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    local_transform: gltf::scene::Transform,
    global_transform_matrix: cgmath::Matrix4<f32>,
    mesh_index: Option<usize>,
    skin_index: Option<usize>,
    children_indices: Vec<usize>,
}

impl Node {
     pub fn set_translation(&mut self, translation:  cgmath::Vector3<f32>) {
        if let gltf::scene::Transform::Decomposed {
            rotation, scale, ..
        } = self.local_transform
        {
            self.local_transform =  gltf::scene::Transform::Decomposed {
                translation: [translation.x, translation.y, translation.z],
                rotation,
                scale,
            }
        }
    }

    pub fn set_rotation(&mut self, rotation: cgmath::Quaternion<f32>) {
        if let gltf::scene::Transform::Decomposed {
            translation, scale, ..
        } = self.local_transform
        {
            self.local_transform = gltf::scene::Transform::Decomposed {
                translation,
                rotation: [rotation.v.x, rotation.v.y, rotation.v.z, rotation.s],
                scale,
            }
        }
    }

    pub fn set_scale(&mut self, scale: cgmath::Vector3<f32>) {
        if let gltf::scene::Transform::Decomposed {
            translation,
            rotation,
            ..
        } = self.local_transform
        {
            self.local_transform = gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale: [scale.x, scale.y, scale.z],
            }
        }
    }

    fn apply_transform(&mut self, transform: cgmath::Matrix4<f32>) {
        let new_tranform = transform * compute_transform_matrix(&self.local_transform);
        self.global_transform_matrix = new_tranform;
    }

     pub fn transform(&self) -> cgmath::Matrix4<f32> {
        self.global_transform_matrix
    }

    pub fn mesh_index(&self) -> Option<usize> {
        self.mesh_index
    }

    pub fn skin_index(&self) -> Option<usize> {
        self.skin_index
    }
}

fn build_graph_run_indices(roots_indices: &[usize], nodes: &[Node]) -> Vec<(usize, Option<usize>)> {
    let mut indices = Vec::new();
    for root_index in roots_indices {
        build_graph_run_indices_rec(nodes, *root_index, None, &mut indices);
    }
    indices
}

fn build_graph_run_indices_rec(
    nodes: &[Node],
    node_index: usize,
    parent_index: Option<usize>,
    indices: &mut Vec<(usize, Option<usize>)>,
) {
    indices.push((node_index, parent_index));
    for child_index in &nodes[node_index].children_indices {
        build_graph_run_indices_rec(nodes, *child_index, Some(node_index), indices);
    }
}

fn compute_transform_matrix(transform: &gltf::scene::Transform) -> cgmath::Matrix4<f32> {
    match transform {
        gltf::scene::Transform::Matrix { matrix } => cgmath::Matrix4::from(*matrix),
        gltf::scene::Transform::Decomposed {
            translation,
            rotation: [xr, yr, zr, wr],
            scale: [xs, ys, zs],
        } => {
            let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::from(*translation));
            let rotation = cgmath::Matrix4::from(cgmath::Quaternion::new(*wr, *xr, *yr, *zr));
            let scale = cgmath::Matrix4::from_nonuniform_scale(*xs, *ys, *zs);
            translation * rotation * scale
        }
    }
}