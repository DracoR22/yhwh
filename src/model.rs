use std::path::Path;
use std::{
    io::{BufReader, Cursor},
};
use cgmath::{SquareMatrix, Zero};
use gltf::buffer::Data;
use gltf::mesh::Bounds;
use math::aabb::Aabb;
use wgpu::util::DeviceExt;

use crate::animation::animation::{load_animations, Animations, PlaybackMode, PlaybackState};
use crate::animation::node::Nodes;
use crate::animation::skin::{create_skins_from_gltf, Skin};

use crate::{
    renderer_common::{CUBE_INDICES, CUBE_VERTICES, PLANE_INDICES, PLANE_VERTICES},
    utils::file::load_file_string_from_env,
    vertex::Vertex,
};

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub aabb: Aabb<f32>
}

pub struct Model {
    pub name: String,
    pub meshes: Vec<Mesh>,
    pub animations: Option<Animations>,
    pub nodes: Nodes,
    pub global_transform: cgmath::Matrix4<f32>,
    pub skins: Vec<Skin>
}

pub async fn load_obj_model(
    file_name: &str,
    device: &wgpu::Device,
    name: &str,
) -> anyhow::Result<Model> {
    let obj_text = load_file_string_from_env("models", file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, _obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_file_string_from_env("models", &p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        Vertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
                            joints: [0; 4],
                            weights: [0.0; 4]
                        }
                    } else {
                        Vertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
                            joints: [0; 4],
                            weights: [0.0; 4]
                        }
                    }
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;

            // calculate tangent and bitangent
            Vertex::calc_tan_vectors(&mut vertices, indices);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                aabb: Aabb::new(cgmath::Vector3::zero(), cgmath::Vector3::zero())
                //material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(Model {
        meshes,
        name: name.to_string(),
        animations: Default::default(),
        nodes: Default::default(),
        global_transform: cgmath::Matrix4::identity(),
        skins: Vec::new()
    })
}


pub fn load_cube(device: &wgpu::Device, name: &str) -> anyhow::Result<Model> {
    let mut meshes = Vec::new();

    let mut vertices = CUBE_VERTICES.to_vec();
    let indices = CUBE_INDICES.to_vec();

    Vertex::calc_tan_vectors(&mut vertices, &indices);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex_Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index_Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let cube_mesh = Mesh {
        name: String::from("Cube_Mesh"),
        vertex_buffer,
        index_buffer,
        num_elements: indices.len() as u32,
        aabb: Aabb::new(cgmath::Vector3::zero(), cgmath::Vector3::zero())
    };

    meshes.push(cube_mesh);

    Ok(Model {
        meshes,
        name: name.to_string(),
        animations: Default::default(),
        nodes: Default::default(),
        global_transform: cgmath::Matrix4::identity(),
        skins: Vec::new()
    })
}

pub fn load_plane(device: &wgpu::Device, name: &str) -> anyhow::Result<Model> {
    let mut meshes = Vec::new();

    let mut vertices = PLANE_VERTICES.to_vec();
    let indices = PLANE_INDICES.to_vec();

    Vertex::calc_tan_vectors(&mut vertices, &indices);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex_Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index_Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let cube_mesh = Mesh {
        name: String::from("Plane_Mesh"),
        vertex_buffer,
        index_buffer,
        num_elements: indices.len() as u32,
        aabb: Aabb::new(cgmath::Vector3::zero(), cgmath::Vector3::zero())
    };

    meshes.push(cube_mesh);

    Ok(Model {
        meshes,
        name: name.to_string(),
        animations: Default::default(),
        nodes: Default::default(),
        global_transform: cgmath::Matrix4::identity(),
        skins: Vec::new()
    })
}

pub fn load_glb_model(device: &wgpu::Device) -> anyhow::Result<Model> {
    let path = "res/models/glock.glb";
    let file_name = Path::new(path).file_name().and_then(|f| f.to_str()).unwrap();

    let (gltf, buffers, _images) = gltf::import(path).expect("Failed to import glTF/GLB file");

    let mut meshes: Vec<Mesh> = Vec::new();

    // load meshes
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            visit_node(&node, &buffers, &mut meshes, device);
        }
    }   

    // load animations
    let animations = load_animations(gltf.animations(), &buffers);

    for (i, anim ) in animations.as_ref().unwrap().animations().iter().enumerate() {
        println!("LOADED ANIMATION: {}", anim.get_name())
    }

    // load skins
    let mut skins = create_skins_from_gltf(gltf.skins(), &buffers);

    // load nodes
    let mut nodes = Nodes::from_gltf_nodes(gltf.nodes(), &gltf.default_scene().unwrap()); //TODO: REMOVE UNWRAP

    let global_transform = {
            let aabb = compute_aabb(&nodes, &meshes);
            let transform = compute_unit_cube_at_origin_transform(aabb);
            nodes.transform(Some(transform));
            nodes
                .get_skins_transform()
                .iter()
                .for_each(|(index, transform)| {
                    let skin = &mut skins[*index];
                    skin.compute_joints_matrices(*transform, nodes.nodes());
                });
            transform
    };
    

    Ok(Model {
        name: file_name.to_string(),
        meshes,
        animations,
        nodes,
        global_transform,
        skins
    })
}

fn visit_node(node: &gltf::Node, data: &[Data], meshes: &mut Vec<Mesh>, device: &wgpu::Device) {
     if let Some(mesh) = node.mesh() {
                let mesh_name = mesh.name().map(|n| n.to_string()).unwrap_or_else(|| format!("UnNamedMesh"));

                for primitive in mesh.primitives() {
                    let mut vertices: Vec<Vertex> = Vec::new();
                    let mut indices: Vec<u32> = Vec::new();

                    let reader = primitive.reader(|buffer| Some(&data[buffer.index()]));

                    let aabb = get_aabb(&primitive.bounding_box());

                    // position attribute
                    if let Some(iter) = reader.read_positions() {
                        for position in iter {
                            vertices.push(Vertex {
                                position,
                                tex_coords: Default::default(),
                                normal: Default::default(),
                                tangent: Default::default(),
                                bitangent: Default::default(),
                                joints: Default::default(),
                                weights: Default::default()
                            });
                        }
                    }

                    // normal attribute
                    if let Some(iter) = reader.read_normals() {
                        let mut normal_index = 0;

                        for normal in iter {
                            vertices[normal_index].normal = normal;
                            normal_index += 1;
                        }
                    }

                    // texture coords attribute
                    if let Some(iter) = reader.read_tex_coords(0) {
                        let mut tex_coord_index = 0;
                        
                        for tex_coord in iter.into_f32() {
                            vertices[tex_coord_index].tex_coords = tex_coord;
                            tex_coord_index += 1;
                        }
                    }

                    // tangents attribure
                    if let Some(iter) = reader.read_tangents() {
                        let mut tangent_index = 0;

                        for tangent in iter {
                            vertices[tangent_index].tangent = [tangent[0], tangent[1], tangent[2]];
                            tangent_index += 1;
                        }
                    }

                    // joints
                    if let Some(iter) = reader.read_joints(0) {
                       let mut joint_index = 0;

                       for joint in iter.into_u16() {
                        vertices[joint_index].joints = [joint[0], joint[1], joint[2], joint[3]];
                        joint_index += 1;
                       }
                    }

                    // weights
                    if let Some(iter) = reader.read_weights(0) {
                        let mut weight_index = 0;

                        for weight in iter.into_f32() {
                            vertices[weight_index].weights = [weight[0], weight[1], weight[2], weight[3]];
                            weight_index += 1;
                        }
                    }

                    // indices
                    if let Some(iter) = reader.read_indices() {
                        indices.append(&mut iter.into_u32().collect::<Vec<u32>>());
                    }

                    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&mesh_name),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                    });

                    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&mesh_name),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                    });

                    meshes.push(Mesh {
                        name: mesh_name.to_string(),
                        vertex_buffer,
                        index_buffer,
                        num_elements: indices.len() as u32,
                        aabb
                    });
                }
            }

        for child in node.children() {
            visit_node(&child, data, meshes, device);
        }
}

impl Model {
    pub fn update(&mut self, delta_time: f32) -> bool {
        let updated = if let Some(animations) = self.animations.as_mut() {
            animations.update(&mut self.nodes, delta_time)
        } else {
            false
        };

        if updated {
            self.nodes.transform(Some(self.global_transform));
            self.nodes
                .get_skins_transform()
                .iter()
                .for_each(|(index, transform)| {
                    let skin = &mut self.skins[*index];
                    skin.compute_joints_matrices(*transform, self.nodes.nodes());
                });
        }

        updated
    }
}

// animations stuff
impl Model {
    pub fn get_animation_playback_state(&self) -> Option<PlaybackState> {
        self.animations
            .as_ref()
            .map(Animations::get_playback_state)
            .copied()
    }

    pub fn set_current_animation(&mut self, animation_index: usize) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_current(animation_index);
        }
    }

    pub fn set_animation_playback_mode(&mut self, playback_mode: PlaybackMode) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_playback_mode(playback_mode);
        }
    }

    pub fn toggle_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.toggle();
        }
    }

    pub fn stop_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.stop();
        }
    }

    pub fn reset_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.reset();
        }
    }
}

fn compute_aabb(nodes: &Nodes, meshes: &[Mesh]) -> Aabb<f32> {
    let aabbs = nodes
        .nodes()
        .iter()
        .filter(|n| n.mesh_index().is_some())
        .map(|n| {
            let mesh = &meshes[n.mesh_index().unwrap()];
            mesh.aabb * n.transform()
        })
        .collect::<Vec<_>>();
    Aabb::union(&aabbs).unwrap()
}

fn compute_unit_cube_at_origin_transform(aabb: Aabb<f32>) -> cgmath::Matrix4<f32> {
    let larger_side = aabb.get_larger_side_size();
    let scale_factor = (1.0_f32 / larger_side) * 10.0;

    let aabb = aabb * scale_factor;
    let center = aabb.get_center();

    let translation = cgmath::Matrix4::from_translation(-center);
    let scale = cgmath::Matrix4::from_scale(scale_factor);
    translation * scale
}

fn get_aabb(bounds: &Bounds<[f32; 3]>) -> Aabb<f32> {
    let min = bounds.min;
    let min = cgmath::Vector3::new(min[0], min[1], min[2]);

    let max = bounds.max;
    let max = cgmath::Vector3::new(max[0], max[1], max[2]);

    Aabb::new(min, max)
}