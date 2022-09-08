use crate::{Prefab, PrefabLoadResult};

use kgltf::*;
use kmath::*;
use koi_animation::TypedAnimationClip;
use koi_assets::*;
use koi_ecs::*;
use koi_renderer::new_texture_from_bytes;
use koi_resources::*;
use koi_transform::*;

pub(crate) struct GlTfLoadResult {
    path: String,
    gltf: kgltf::GlTf,
    data: Option<Vec<u8>>,
    mesh_primitive_data: Vec<MeshPrimitiveData>,
    buffers: Vec<Option<Vec<u8>>>,
}
pub(super) struct MeshPrimitiveData {
    /// The data for this mesh and its material attributes
    // The way this is structured means that multiple things that share attributes will duplicate the attribute data.
    primitives: Vec<(koi_renderer::MeshData, Option<usize>)>,
}

#[derive(Clone)]
struct TextureLoadState {
    linear: Option<Handle<koi_renderer::Texture>>,
    srgb: Option<Handle<koi_renderer::Texture>>,
}

async fn fetch_buffers(path: &str, gltf: &kgltf::GlTf) -> Vec<Option<Vec<u8>>> {
    let mut buffers = Vec::with_capacity(gltf.buffers.len());
    for buffer in &gltf.buffers {
        buffers.push(if let Some(uri) = &buffer.uri {
            let path = std::path::Path::new(&path).parent().unwrap().join(uri);
            Some(
                koi_fetch::fetch_bytes(path.to_str().unwrap())
                    .await
                    .unwrap(),
            )
        } else {
            None
        })
    }
    buffers
}

pub(crate) async fn load_glb(path: String) -> Option<PrefabLoadResult> {
    let bytes = koi_fetch::fetch_bytes(&path)
        .await
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path));

    let glb = kgltf::GLB::from_bytes(&bytes).ok()?;

    let buffers = fetch_buffers(&path, &glb.gltf).await;

    println!("BINARY DATA: {:?}", glb.binary_data.is_some());
    let mesh_primitive_data =
        load_mesh_primitive_data(&glb.gltf, glb.binary_data.as_deref(), &buffers).await?;

    Some(super::PrefabLoadResult::GlTf(GlTfLoadResult {
        path,
        gltf: glb.gltf,
        // TODO: Can this copy be avoided?
        data: glb.binary_data.map(|d| d.into_owned()),
        mesh_primitive_data,
        buffers,
    }))
}

// Step 1: Fetch the glTF file off the main thread and ready its data.
pub(crate) async fn load_gltf(path: String) -> Option<PrefabLoadResult> {
    let bytes = koi_fetch::fetch_bytes(&path)
        .await
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path));

    let s = std::str::from_utf8(&bytes).ok()?;
    let gltf = <kgltf::GlTf as kgltf::FromJson>::from_json(s)?;

    let buffers = fetch_buffers(&path, &gltf).await;

    let mesh_primitive_data = load_mesh_primitive_data(&gltf, None, &buffers).await?;

    Some(super::PrefabLoadResult::GlTf(GlTfLoadResult {
        path,
        gltf,
        data: None,
        mesh_primitive_data,
        buffers,
    }))
}

// Step 2: Load the glTF on the main thread.
pub(crate) fn finalize_gltf_load(
    resources: &Resources,
    gltf_load_result: GlTfLoadResult,
) -> Option<Prefab> {
    let mut new_world = koi_ecs::World::new();

    let graphics = &mut resources
        .get::<koi_renderer::Renderer>()
        .raw_graphics_context;

    let mut materials = resources.get::<AssetStore<koi_renderer::Material>>();
    let mut textures = resources.get::<AssetStore<koi_renderer::Texture>>();
    let mut meshes = resources.get::<AssetStore<koi_renderer::Mesh>>();
    let mut animations = resources.get::<AssetStore<koi_animation::Animation>>();

    let GlTfLoadResult {
        path,
        gltf,
        data,
        mesh_primitive_data,
        buffers,
    } = gltf_load_result;

    let data = data.as_ref().map(|d| &d[..]);

    for extension in &gltf.extensions_required {
        match extension.as_str() {
            "KHR_materials_unlit" => {}
            _ => panic!("Unsupported Gltf extension: {}", extension),
        }
    }

    let scene = gltf.scene.unwrap();
    let scene = &gltf.scenes[scene];

    let mut texture_load_states = vec![
        TextureLoadState {
            linear: None,
            srgb: None,
        };
        gltf.textures.len()
    ];

    let gltf_materials: Vec<_> = gltf
        .materials
        .iter()
        .map(|material| {
            let mut new_material = koi_renderer::Material::default();
            if let Some(pbr_metallic_roughness) = &material.pbr_metallic_roughness {
                let base_color = pbr_metallic_roughness.base_color_factor;

                new_material.base_color = koi_renderer::Color::new(
                    base_color[0],
                    base_color[1],
                    base_color[2],
                    base_color[3],
                );
                new_material.metallicness = pbr_metallic_roughness.metallic_factor;
                new_material.perceptual_roughness = pbr_metallic_roughness.roughness_factor;
                new_material.base_color_texture =
                    pbr_metallic_roughness.base_color_texture.as_ref().map(|t| {
                        get_texture(
                            &gltf,
                            &data,
                            &path,
                            graphics,
                            &mut textures,
                            &mut texture_load_states,
                            true,
                            t.index,
                        )
                    });

                new_material.metallic_roughness_texture = pbr_metallic_roughness
                    .metallic_roughness_texture
                    .as_ref()
                    .map(|t| {
                        get_texture(
                            &gltf,
                            &data,
                            &path,
                            graphics,
                            &mut textures,
                            &mut texture_load_states,
                            false,
                            t.index,
                        )
                    });
            }

            new_material.normal_texture = material.normal_texture.as_ref().map(|t| {
                get_texture(
                    &gltf,
                    &data,
                    &path,
                    graphics,
                    &mut textures,
                    &mut texture_load_states,
                    false,
                    t.index,
                )
            });

            /*
            // Is this correct?
            new_material.emissiveness = Vec3::new(
                material.emissive_factor[0],
                material.emissive_factor[1],
                material.emissive_factor[2],
            );

            new_material.emissive_texture = material.emissive_texture.as_ref().map(|t| {
                get_texture(
                    &gltf,
                    &data,
                    &path,
                    textures,
                    &mut texture_load_states,
                    true,
                    t.index,
                )
            });
            */

            let unlit = material.extensions.contains_key("KHR_materials_unlit");
            let transparent = match material.alpha_mode {
                kgltf::MaterialAlphaMode::Blend => true,
                kgltf::MaterialAlphaMode::Opaque => false,
                kgltf::MaterialAlphaMode::Mask => {
                    klog::log!("NOT YET HANDLED GLTF MASK MATERIAL");
                    true
                }
            };

            if unlit {
                let shader = match (transparent, material.double_sided) {
                    (false, false) => koi_renderer::Shader::UNLIT,
                    (false, true) => koi_renderer::Shader::UNLIT_DOUBLE_SIDED,
                    _ => {
                        klog::log!("KOI WARNING: Unimplemented material!");
                        koi_renderer::Shader::UNLIT
                    }
                    // (true, false) => Shader::PHYSICALLY_BASED_TRANSPARENT,
                    // (false, true) => Shader::PHYSICALLY_BASED_DOUBLE_SIDED,
                    // (true, true) => Shader::PHYSICALLY_BASED_TRANSPARENT_DOUBLE_SIDED,
                };
                new_material.shader = shader;
            } else {
                let shader = match (transparent, material.double_sided) {
                    (false, false) => koi_renderer::Shader::PHYSICALLY_BASED,
                    (false, true) => koi_renderer::Shader::PHYSICALLY_BASED_DOUBLE_SIDED,
                    _ => {
                        klog::log!("KOI WARNING: Unimplemented material!");
                        koi_renderer::Shader::PHYSICALLY_BASED
                    }
                    // (true, false) => Shader::PHYSICALLY_BASED_TRANSPARENT,
                    // (false, true) => Shader::PHYSICALLY_BASED_DOUBLE_SIDED,
                    // (true, true) => Shader::PHYSICALLY_BASED_TRANSPARENT_DOUBLE_SIDED,
                };
                new_material.shader = shader;
            };

            materials.add(new_material)
        })
        .collect();

    let mut mesh_primitives = Vec::with_capacity(mesh_primitive_data.len());

    for mesh_primitive_data in &mesh_primitive_data {
        let mut primitives = Vec::with_capacity(mesh_primitive_data.primitives.len());
        for (mesh_data, material_index) in &mesh_primitive_data.primitives {
            let new_mesh = meshes.add(koi_renderer::Mesh::new(graphics, mesh_data.clone()));
            primitives.push((new_mesh, *material_index));
        }
        mesh_primitives.push(primitives);
    }

    let mut node_index_to_entity = Vec::new();
    // scene.nodes only includes root nodes.
    for node in &scene.nodes {
        initialize_nodes(
            &mut new_world,
            &mut node_index_to_entity,
            &materials,
            &gltf_materials,
            &mesh_primitives,
            &gltf.nodes,
            *node,
            None,
        )
    }

    let mut model_animations = std::collections::HashMap::new();

    let mut first_key = String::new();

    // Initialize animations.
    for (index, gltf_animation) in gltf.animations.iter().enumerate() {
        let mut animation_clips = Vec::new();
        let mut associated_entities = Vec::new();

        for channel in gltf_animation.channels.iter() {
            if let Some(node) = channel.target.node {
                let sampler = &gltf_animation.samplers[channel.sampler];

                let timestamp_accessor_index = sampler.input;
                let value_accessor_index = sampler.output;

                let animation_curve = match sampler.interpolation {
                    AnimationSamplerInterpolation::Linear => {
                        koi_animation::animation_curves::linear
                    }
                    AnimationSamplerInterpolation::Step => koi_animation::animation_curves::step,
                    AnimationSamplerInterpolation::Cubicspline => {
                        todo!()
                    }
                };

                let key_frames = get_buffer::<f32, _, _>(
                    &gltf,
                    &data,
                    &buffers,
                    timestamp_accessor_index,
                    |v| v,
                )?;

                let accessor = gltf.accessors.get(value_accessor_index)?;
                match channel.target.path {
                    AnimationChannelTargetPath::Translation => {
                        let accessor_type = accessor.type_.clone();

                        if accessor_type != AccessorType::Vec3 {
                            println!("Malformed glTF: Translation animation channel does not match accessor");
                            return None;
                        }
                        let translations = get_buffer::<Vec3, _, _>(
                            &gltf,
                            &data,
                            &buffers,
                            value_accessor_index,
                            |v| v,
                        )?;
                        animation_clips.push(koi_animation::AnimationClip {
                            animation_curve,
                            entity_mapping_index: associated_entities.len(),
                            typed_animation_clip: Box::new(TypedAnimationClip {
                                set_property: |e, v0: &Vec3, v1, t| {
                                    e.get::<&mut Transform>().unwrap().position = v0.lerp(*v1, t)
                                },
                                key_frames,
                                values: translations,
                            }),
                        })
                    }
                    AnimationChannelTargetPath::Rotation => {
                        let accessor_type = accessor.type_.clone();

                        if accessor_type != AccessorType::Vec4 {
                            println!("Malformed glTF: Rotation animation channel does not match accessor");
                            return None;
                        }
                        let rotations = get_buffer::<Quat, _, _>(
                            &gltf,
                            &data,
                            &buffers,
                            value_accessor_index,
                            |v| v,
                        )?;
                        animation_clips.push(koi_animation::AnimationClip {
                            animation_curve,
                            entity_mapping_index: associated_entities.len(),
                            typed_animation_clip: Box::new(TypedAnimationClip {
                                set_property: |e, v0: &Quat, v1, t| {
                                    e.get::<&mut Transform>().unwrap().rotation = v0.slerp(*v1, t)
                                },
                                key_frames,
                                values: rotations,
                            }),
                        })
                    }
                    AnimationChannelTargetPath::Scale => {
                        let accessor_type = accessor.type_.clone();

                        if accessor_type != AccessorType::Vec3 {
                            println!(
                                "Malformed glTF: Scale animation channel does not match accessor"
                            );
                            return None;
                        }
                        let scales = get_buffer::<Vec3, _, _>(
                            &gltf,
                            &data,
                            &buffers,
                            value_accessor_index,
                            |v| v,
                        )?;
                        animation_clips.push(koi_animation::AnimationClip {
                            animation_curve,
                            entity_mapping_index: associated_entities.len(),
                            typed_animation_clip: Box::new(TypedAnimationClip {
                                set_property: |e, v0: &Vec3, v1, t| {
                                    e.get::<&mut Transform>().unwrap().scale = v0.lerp(*v1, t)
                                },
                                key_frames,
                                values: scales,
                            }),
                        });
                    }
                    AnimationChannelTargetPath::Weights => todo!(),
                }
                associated_entities.push(node_index_to_entity[node].map(|e| e.0));
            }
        }

        let animation = koi_animation::Animation::new(animation_clips);
        let animation_handle = animations.add(animation);
        let name = gltf_animation
            .name
            .clone()
            .unwrap_or_else(|| index.to_string());
        if first_key.is_empty() {
            first_key = name.clone()
        }
        model_animations.insert(name, (associated_entities, animation_handle));
    }

    if !model_animations.is_empty() {
        println!("ANIMATION NAME: {:?}", first_key);
        // let (entity_mapping, animation_handle) = model_animations.get(&first_key).unwrap();
        let _ = new_world.spawn((koi_animation::AnimationPlayer {
            playing_animations: Vec::new(), /*vec![PlayingAnimation {
                                                time: 0.0,
                                                animation: animation_handle.clone(),
                                                entity_mapping: entity_mapping.clone(),
                                            }],*/
            animations: model_animations,
        },));
    }

    Some(Prefab(new_world))
}

// This helper function is used to load-textures late.
// Because `koi` must known the color-space of an image when loaded, but it varies depending on how the
// gltf uses the texture.
fn get_texture(
    gltf: &kgltf::GlTf,
    data: &Option<&[u8]>,
    path: &str,
    graphics: &mut koi_graphics_context::GraphicsContext,
    textures: &mut AssetStore<koi_renderer::Texture>,
    texture_load_states: &mut [TextureLoadState],
    srgb: bool,
    texture_index: usize,
) -> Handle<koi_renderer::Texture> {
    let image_index = gltf.textures[texture_index].source.unwrap();
    if srgb {
        if let Some(handle) = texture_load_states[texture_index].srgb.clone() {
            return handle;
        }
    } else if let Some(handle) = texture_load_states[texture_index].linear.clone() {
        return handle;
    }

    let image = &gltf.images[image_index];
    let new_handle = if let Some(uri) = &image.uri {
        let path = std::path::Path::new(path).parent().unwrap().join(uri);

        textures.load(
            path.to_str().unwrap(),
            koi_renderer::koi_graphics_context::TextureSettings {
                srgb,
                ..Default::default()
            },
        )
    } else {
        let buffer_view = &gltf.buffer_views[image.buffer_view.unwrap()];

        let bytes = &data.unwrap()
            [buffer_view.byte_offset..buffer_view.byte_offset + buffer_view.byte_length];
        let extension = match image.mime_type.as_ref().unwrap() {
            kgltf::ImageMimeType::ImageJpeg => "jpeg",
            kgltf::ImageMimeType::ImagePng => "png",
        };

        // TODO: This should be decoded off the main thread.

        textures.add(
            new_texture_from_bytes(
                graphics,
                extension,
                bytes,
                koi_graphics_context::TextureSettings {
                    srgb,
                    ..Default::default()
                },
            )
            .unwrap(),
        )
    };

    if srgb {
        texture_load_states[texture_index].srgb = Some(new_handle.clone())
    } else {
        texture_load_states[texture_index].linear = Some(new_handle.clone())
    }
    new_handle
}

fn initialize_nodes(
    gltf_world: &mut World,
    node_index_to_entity: &mut Vec<Option<(Entity, Transform)>>,
    materials: &AssetStore<koi_renderer::Material>,
    gltf_materials: &[Handle<koi_renderer::Material>],
    mesh_primitives: &[Vec<(Handle<koi_renderer::Mesh>, Option<usize>)>],
    nodes: &[kgltf::Node],
    node_index: usize,
    parent: Option<Entity>,
) {
    let node = &nodes[node_index];
    let transform: Transform = if let Some(matrix) = &node.matrix {
        Transform::from_mat4(matrix.try_into().unwrap())
    } else {
        Transform {
            position: node.translation.map_or(Vec3::ZERO, |t| t.into()),
            rotation: node.rotation.map_or(Quat::IDENTITY, |q| q.into()),
            scale: node.scale.map_or(Vec3::ONE, |s| s.into()),
        }
    };

    let entity = if let Some(mesh) = node.mesh {
        let mesh_primitives = &mesh_primitives[mesh];

        let entity_root = gltf_world.spawn((transform,));
        for (mesh, material_index) in mesh_primitives {
            let material_handle =
                material_index.map_or_else(|| Handle::PLACEHOLDER, |i| gltf_materials[i].clone());
            let primitive_entity =
                gltf_world.spawn((mesh.clone(), material_handle, Transform::new()));
            gltf_world
                .set_parent(entity_root, primitive_entity)
                .unwrap();
        }
        entity_root
    } else {
        gltf_world.spawn((transform,))
    };

    /*
    if let Some(name) = &node.name {
        gltf_world
            .add_component(entity, Name(name.clone()))
            .unwrap();
    }
    */

    if let Some(parent) = parent {
        gltf_world.set_parent(parent, entity).unwrap();
    }

    node_index_to_entity.resize(node_index_to_entity.len().max(node_index + 1), None);
    node_index_to_entity[node_index] = Some((entity, transform));

    for child in &node.children {
        initialize_nodes(
            gltf_world,
            node_index_to_entity,
            materials,
            gltf_materials,
            mesh_primitives,
            nodes,
            *child,
            Some(entity),
        );
    }
}

pub(super) async fn load_mesh_primitive_data(
    gltf: &kgltf::GlTf,
    data: Option<&[u8]>,
    buffers: &[Option<Vec<u8>>],
) -> Option<Vec<MeshPrimitiveData>> {
    let mut meshes = Vec::with_capacity(gltf.meshes.len());
    for mesh in &gltf.meshes {
        let mut primitives = Vec::with_capacity(mesh.primitives.len());

        for primitive in &mesh.primitives {
            let mut positions = None;
            let mut normals = None;
            let mut texture_coordinates = None;
            let mut colors = None;

            for (attribute, accessor_index) in &primitive.attributes {
                // https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#meshes
                let accessor_type = gltf.accessors[*accessor_index].type_.clone();
                let accessor_component_type =
                    gltf.accessors[*accessor_index].component_type.clone();

                // Table of what accessor types need to be implemented for each attribute:
                // https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#meshes-overview
                match attribute.as_str() {
                    "POSITION" => {
                        positions = Some(get_buffer::<Vec3, _, _>(
                            gltf,
                            &data,
                            buffers,
                            *accessor_index,
                            |v| v,
                        )?);
                    }
                    "TEXCOORD_0" => {
                        texture_coordinates = Some(match accessor_component_type {
                            AccessorComponentType::UnsignedByte => {
                                get_buffer::<Vector<u8, 2>, _, _>(
                                    gltf,
                                    &data,
                                    buffers,
                                    *accessor_index,
                                    |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                )?
                            }
                            AccessorComponentType::UnsignedShort => {
                                get_buffer::<Vector<u16, 2>, _, _>(
                                    gltf,
                                    &data,
                                    buffers,
                                    *accessor_index,
                                    |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                )?
                            }
                            AccessorComponentType::Float => get_buffer::<Vec2, _, _>(
                                gltf,
                                &data,
                                buffers,
                                *accessor_index,
                                |v| v,
                            )?,
                            _ => unimplemented!(),
                        });
                    }
                    "NORMAL" => {
                        normals = Some(get_buffer::<Vec3, _, _>(
                            gltf,
                            &data,
                            buffers,
                            *accessor_index,
                            |v| v,
                        ))?;
                    }
                    "COLOR_0" => {
                        // COLOR_0 can be different accessor types according to the spec.
                        // Here we make them always a `Vec4`
                        match accessor_type {
                            kgltf::AccessorType::Vec4 => {
                                colors = Some(match accessor_component_type {
                                    AccessorComponentType::Float => get_buffer::<Vec4, _, _>(
                                        gltf,
                                        &data,
                                        buffers,
                                        *accessor_index,
                                        |v| v,
                                    )?,
                                    AccessorComponentType::UnsignedByte => {
                                        get_buffer::<Vector<u8, 4>, _, _>(
                                            gltf,
                                            &data,
                                            buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                        )?
                                    }
                                    AccessorComponentType::UnsignedShort => {
                                        get_buffer::<Vector<u16, 4>, _, _>(
                                            gltf,
                                            &data,
                                            buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                        )?
                                    }
                                    _ => unimplemented!(),
                                });
                            }
                            kgltf::AccessorType::Vec3 => {
                                let colors_vec3 = match accessor_component_type {
                                    AccessorComponentType::Float => get_buffer::<Vec3, _, _>(
                                        gltf,
                                        &data,
                                        buffers,
                                        *accessor_index,
                                        |v| v,
                                    )?,
                                    AccessorComponentType::UnsignedByte => {
                                        get_buffer::<Vector<u8, 3>, _, _>(
                                            gltf,
                                            &data,
                                            buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                        )?
                                    }
                                    AccessorComponentType::UnsignedShort => {
                                        get_buffer::<Vector<u16, 3>, _, _>(
                                            gltf,
                                            &data,
                                            buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                        )?
                                    }
                                    _ => unimplemented!(),
                                };
                                colors = Some(colors_vec3.iter().map(|v| v.extend(1.0)).collect());
                            }
                            _ => unimplemented!(),
                        }
                    }
                    "TANGENT" => {}
                    "TEXCOORD_1" => {}
                    "JOINTS_0" => {}
                    "WEIGHTS_0" => {}
                    _ => {} // Unimplemented
                }
            }

            if let Some(indices) = primitive.indices {
                let indices = get_indices(gltf, &data, buffers, indices).await?;

                for index in indices.iter() {
                    for i in index {
                        if *i >= positions.as_ref().unwrap().len() as u32 {
                            panic!();
                        }
                    }
                }
                let mesh_data = koi_renderer::MeshData {
                    positions: positions.unwrap(),
                    normals: normals.unwrap_or_default(),
                    texture_coordinates: texture_coordinates.unwrap_or_default(),
                    colors: colors.unwrap_or_default(),
                    indices,
                };

                primitives.push((mesh_data, primitive.material))
            } else {
                klog::log!("Warning: GLTF primitive does not have indices.");
            }
        }
        meshes.push(MeshPrimitiveData { primitives });
    }
    Some(meshes)
}

fn read_accessor_bytes<'a>(
    gltf: &'a kgltf::GlTf,
    data: &'a Option<&[u8]>,
    buffers: &'a [Option<Vec<u8>>],
    accessor_index: usize,
) -> Option<(&'a [u8], &'a Accessor)> {
    let accessor = &gltf.accessors.get(accessor_index)?;
    let buffer_view = &gltf.buffer_views.get(accessor.buffer_view.unwrap())?;

    let member_size = match accessor.component_type {
        AccessorComponentType::Byte => std::mem::size_of::<u8>(),
        AccessorComponentType::UnsignedByte => std::mem::size_of::<u8>(),
        AccessorComponentType::Short => std::mem::size_of::<i16>(),
        AccessorComponentType::UnsignedShort => std::mem::size_of::<u16>(),
        AccessorComponentType::UnsignedInt => std::mem::size_of::<u32>(),
        AccessorComponentType::Float => std::mem::size_of::<f32>(),
    };

    let items_per_member = match accessor.type_ {
        kgltf::AccessorType::Scalar => 1,
        kgltf::AccessorType::Vec2 => 2,
        kgltf::AccessorType::Vec3 => 3,
        kgltf::AccessorType::Vec4 => 4,
        kgltf::AccessorType::Mat2 => 4,
        kgltf::AccessorType::Mat3 => 9,
        kgltf::AccessorType::Mat4 => 16,
    };

    let len_bytes = accessor.count * member_size * items_per_member;

    let start = buffer_view.byte_offset + accessor.byte_offset;
    let end = start + len_bytes;

    let buffer = gltf.buffers.get(buffer_view.buffer)?;
    Some((
        if buffer.uri.is_some() {
            buffers.get(buffer_view.buffer)?.as_ref()?.get(start..end)?
        } else {
            data.as_ref().unwrap().get(start..end)?
        },
        accessor,
    ))
}

async fn get_indices(
    gltf: &kgltf::GlTf,
    data: &Option<&[u8]>,
    buffers: &[Option<Vec<u8>>],
    accessor: usize,
) -> Option<Vec<[u32; 3]>> {
    let (bytes, accessor) = read_accessor_bytes(gltf, data, buffers, accessor)?;

    unsafe {
        Some(match accessor.component_type {
            kgltf::AccessorComponentType::UnsignedByte => {
                let (_prefix, shorts, _suffix) = bytes.align_to::<u8>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            kgltf::AccessorComponentType::UnsignedShort => {
                let (_prefix, shorts, _suffix) = bytes.align_to::<u16>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            kgltf::AccessorComponentType::UnsignedInt => {
                let (_prefix, shorts, _suffix) = bytes.align_to::<u32>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            _ => unreachable!(), // Should error instead
        })
    }
}

fn get_buffer<T: Copy, TOut, F: FnMut(T) -> TOut>(
    gltf: &kgltf::GlTf,
    data: &Option<&[u8]>,
    buffers: &[Option<Vec<u8>>],
    accessor: usize,
    convert_value: F,
) -> Option<Vec<TOut>> {
    let (bytes, _) = read_accessor_bytes(gltf, data, buffers, accessor).unwrap();
    unsafe { Some(bytes_to_buffer(bytes, convert_value)) }
}

unsafe fn bytes_to_buffer<T: Copy, TOut, F: FnMut(T) -> TOut>(
    bytes: &[u8],
    convert_value: F,
) -> Vec<TOut> {
    let (_prefix, shorts, _suffix) = bytes.align_to::<T>();
    shorts.iter().copied().map(convert_value).collect()
}
