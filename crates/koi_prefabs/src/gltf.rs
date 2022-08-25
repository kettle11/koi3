use crate::{Prefab, PrefabLoadResult};

use kgltf::*;
use kmath::*;
use koi_assets::*;
use koi_ecs::*;
use koi_resources::*;
use koi_transform::*;

pub(crate) struct GlTfLoadResult {
    path: String,
    gltf: kgltf::GlTf,
    data: Option<Vec<u8>>,
    mesh_primitive_data: Vec<MeshPrimitiveData>,
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

// Step 1: Fetch the glTF file off the main thread and ready its data.
pub(crate) async fn load_gltf(path: String) -> Option<PrefabLoadResult> {
    let bytes = koi_fetch::fetch_bytes(&path)
        .await
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path));

    let s = std::str::from_utf8(&bytes).ok()?;
    let gltf = <kgltf::GlTf as kgltf::FromJson>::from_json(s)?;

    let mesh_primitive_data = load_mesh_primitive_data(&path, &gltf, None).await;
    Some(super::PrefabLoadResult::GlTf(GlTfLoadResult {
        path,
        gltf,
        data: None,
        mesh_primitive_data,
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

    let GlTfLoadResult {
        path,
        gltf,
        data,
        mesh_primitive_data,
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
                            &mut *textures,
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
                            &mut *textures,
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
                    &mut *textures,
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
                new_material.shader = koi_renderer::Shader::UNLIT;
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

    for node in &scene.nodes {
        initialize_nodes(
            &mut new_world,
            &materials,
            &gltf_materials,
            &mesh_primitives,
            &gltf.nodes,
            *node,
            None,
        )
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
            koi_renderer::kgraphics::TextureSettings {
                srgb,
                ..Default::default()
            },
        )
    } else {
        let buffer_view = &gltf.buffer_views[image.buffer_view.unwrap()];
        let byte_offset = buffer_view.byte_offset;
        let byte_length = buffer_view.byte_length;

        let _bytes = &data.unwrap()[byte_offset..byte_offset + byte_length];
        let _extension = match image.mime_type.as_ref().unwrap() {
            kgltf::ImageMimeType::ImageJpeg => "jpeg",
            kgltf::ImageMimeType::ImagePng => "png",
        };

        todo!()
        /*
        textures.load_with_data_and_options_and_extension(
            // Todo: this is likely to be a large allocation. Perhaps an `Arc` should be used instead?
            bytes.to_vec(),
            extension.to_string(),
            koi_renderer::kgraphics::TextureSettings {
                srgb,
                ..Default::default()
            },
        )
        */
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
    materials: &AssetStore<koi_renderer::Material>,
    gltf_materials: &[Handle<koi_renderer::Material>],
    mesh_primitives: &[Vec<(Handle<koi_renderer::Mesh>, Option<usize>)>],
    nodes: &[kgltf::Node],
    node: usize,
    parent: Option<Entity>,
) {
    let node = &nodes[node];
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
        // This commented out condition flattened the hierarchy slightly if an Entity only had
        // one mesh primitive. This might be useful in some cases, but for now for simplicity and clarity
        // we're going to ignore that case.
        /*  if mesh_primitives.len() == 1 {
            let (mesh, material_index) = &mesh_primitives[0];
            let material_handle =
                material_index.map_or_else(Handle::default, |i| gltf_materials[i].clone());
            gltf_world.spawn((
                mesh.clone(),
                material_handle,
                RenderFlags::DEFAULT,
                transform,
            ))
        } else */
        {
            let entity_root = gltf_world.spawn((transform,));
            for (mesh, material_index) in mesh_primitives {
                let material_handle = material_index
                    .map_or_else(|| Handle::PLACEHOLDER, |i| gltf_materials[i].clone());
                let primitive_entity =
                    gltf_world.spawn((mesh.clone(), material_handle, Transform::new()));
                gltf_world
                    .set_parent(entity_root, primitive_entity)
                    .unwrap();
            }
            entity_root
        }
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

    for child in &node.children {
        initialize_nodes(
            gltf_world,
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
    path: &str,
    gltf: &kgltf::GlTf,
    data: Option<&[u8]>,
) -> Vec<MeshPrimitiveData> {
    let mut buffers = Vec::with_capacity(gltf.buffers.len());
    for buffer in &gltf.buffers {
        buffers.push(if let Some(uri) = &buffer.uri {
            let path = std::path::Path::new(path).parent().unwrap().join(uri);
            // klog::log!("FETCHING BUFFER!: {:?}", path);
            Some(
                koi_fetch::fetch_bytes(path.to_str().unwrap())
                    .await
                    .unwrap(),
            )
        } else {
            None
        })
    }

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
                        positions = Some(
                            get_buffer::<Vec3, _, _>(gltf, &data, &buffers, *accessor_index, |v| v)
                                .await,
                        );
                    }
                    "TEXCOORD_0" => {
                        texture_coordinates = Some(match accessor_component_type {
                            AccessorComponentType::UnsignedByte => {
                                get_buffer::<Vector<u8, 2>, _, _>(
                                    gltf,
                                    &data,
                                    &buffers,
                                    *accessor_index,
                                    |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                )
                                .await
                            }
                            AccessorComponentType::UnsignedShort => {
                                get_buffer::<Vector<u16, 2>, _, _>(
                                    gltf,
                                    &data,
                                    &buffers,
                                    *accessor_index,
                                    |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                )
                                .await
                            }
                            AccessorComponentType::Float => {
                                get_buffer::<Vec2, _, _>(
                                    gltf,
                                    &data,
                                    &buffers,
                                    *accessor_index,
                                    |v| v,
                                )
                                .await
                            }
                            _ => unimplemented!(),
                        });
                    }
                    "NORMAL" => {
                        normals = Some(
                            get_buffer::<Vec3, _, _>(gltf, &data, &buffers, *accessor_index, |v| v)
                                .await,
                        );
                    }
                    "COLOR_0" => {
                        // COLOR_0 can be different accessor types according to the spec.
                        // Here we make them always a `Vec4`
                        match accessor_type {
                            kgltf::AccessorType::Vec4 => {
                                colors = Some(match accessor_component_type {
                                    AccessorComponentType::Float => {
                                        get_buffer::<Vec4, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |v| v,
                                        )
                                        .await
                                    }
                                    AccessorComponentType::UnsignedByte => {
                                        get_buffer::<Vector<u8, 4>, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                        )
                                        .await
                                    }
                                    AccessorComponentType::UnsignedShort => {
                                        get_buffer::<Vector<u16, 4>, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                        )
                                        .await
                                    }
                                    _ => unimplemented!(),
                                });
                            }
                            kgltf::AccessorType::Vec3 => {
                                let colors_vec3 = match accessor_component_type {
                                    AccessorComponentType::Float => {
                                        get_buffer::<Vec3, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |v| v,
                                        )
                                        .await
                                    }
                                    AccessorComponentType::UnsignedByte => {
                                        get_buffer::<Vector<u8, 3>, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u8::MAX as f32)),
                                        )
                                        .await
                                    }
                                    AccessorComponentType::UnsignedShort => {
                                        get_buffer::<Vector<u16, 3>, _, _>(
                                            gltf,
                                            &data,
                                            &buffers,
                                            *accessor_index,
                                            |b| b.map(|v| *v as f32 / (u16::MAX as f32)),
                                        )
                                        .await
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
                let indices = get_indices(gltf, &data, &buffers, indices).await;

                let mesh_data = koi_renderer::MeshData {
                    positions: positions.unwrap(),
                    normals: normals.unwrap_or_else(Vec::new),
                    texture_coordinates: texture_coordinates.unwrap_or_else(Vec::new),
                    colors: colors.unwrap_or_else(Vec::new),
                    indices,
                };

                primitives.push((mesh_data, primitive.material))
            } else {
                klog::log!("Warning: GLTF primitive does not have indices.");
            }
        }
        meshes.push(MeshPrimitiveData { primitives });
    }
    meshes
}

async fn get_indices(
    gltf: &kgltf::GlTf,
    data: &Option<&[u8]>,
    buffers: &[Option<Vec<u8>>],
    accessor: usize,
) -> Vec<[u32; 3]> {
    let accessor = &gltf.accessors[accessor];
    let count = accessor.count;

    let buffer_view = accessor.buffer_view.unwrap();
    let buffer_view = &gltf.buffer_views[buffer_view];
    let buffer = &gltf.buffers[buffer_view.buffer];

    let byte_offset = accessor.byte_offset + buffer_view.byte_offset;
    let byte_length = buffer_view.byte_length;

    let bytes = if buffer.uri.is_some() {
        &buffers[buffer_view.buffer].as_ref().unwrap()[byte_offset..byte_offset + byte_length]
    } else {
        &data.as_ref().unwrap()[byte_offset..byte_offset + byte_length]
    };

    unsafe {
        match accessor.component_type {
            kgltf::AccessorComponentType::UnsignedByte => {
                let bytes = &bytes[0..count * std::mem::size_of::<u8>()];
                let (_prefix, shorts, _suffix) = bytes.align_to::<u8>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            kgltf::AccessorComponentType::UnsignedShort => {
                let bytes = &bytes[0..count * std::mem::size_of::<u16>()];
                let (_prefix, shorts, _suffix) = bytes.align_to::<u16>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            kgltf::AccessorComponentType::UnsignedInt => {
                let bytes = &bytes[0..count * std::mem::size_of::<u32>()];
                let (_prefix, shorts, _suffix) = bytes.align_to::<u32>();
                shorts
                    .chunks_exact(3)
                    .map(|u| [u[0] as u32, u[1] as u32, u[2] as u32])
                    .collect()
            }
            _ => unreachable!(), // Should error instead
        }
    }
}

async fn get_buffer<T: Copy, TOut, F: FnMut(T) -> TOut>(
    gltf: &kgltf::GlTf,
    data: &Option<&[u8]>,
    buffers: &[Option<Vec<u8>>],
    accessor: usize,
    convert_value: F,
) -> Vec<TOut> {
    let accessor = &gltf.accessors[accessor];
    let count = accessor.count;

    let buffer_view = accessor.buffer_view.unwrap();
    let buffer_view = &gltf.buffer_views[buffer_view];
    let buffer = &gltf.buffers[buffer_view.buffer];

    let byte_offset = accessor.byte_offset + buffer_view.byte_offset;
    // let byte_length = buffer_view.byte_length;

    /*
    let bytes = if let Some(uri) = &buffer.uri {
        buffers[buffer_view.buffer].as_ref().unwrap()[byte_offset..byte_offset + byte_length];
    } else {
        &data.as_ref().unwrap()[byte_offset..byte_offset + byte_length]
    };
    */

    if buffer.uri.is_some() {
        let bytes = buffers[buffer_view.buffer].as_ref().unwrap();
        unsafe {
            bytes_to_buffer(
                &bytes[byte_offset..byte_offset + count * std::mem::size_of::<T>()],
                convert_value,
            )
        }
    } else {
        // Use the built in data buffer
        unsafe {
            bytes_to_buffer(
                &data.as_ref().unwrap()
                    [byte_offset..byte_offset + count * std::mem::size_of::<T>()],
                convert_value,
            )
        }
    }
}

unsafe fn bytes_to_buffer<T: Copy, TOut, F: FnMut(T) -> TOut>(
    bytes: &[u8],
    convert_value: F,
) -> Vec<TOut> {
    let (_prefix, shorts, _suffix) = bytes.align_to::<T>();
    shorts.iter().copied().map(convert_value).collect()
}