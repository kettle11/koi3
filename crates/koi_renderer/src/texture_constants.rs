use crate::Texture;
use kgraphics::GraphicsContextTrait;
use koi_assets::*;

impl Texture {
    pub const WHITE: Handle<Texture> = Handle::<Texture>::from_index(0);
    pub const BLACK: Handle<Texture> = Handle::<Texture>::from_index(1);

    /// A texture that produces normals that all face outwards.
    /// The color is (0.5, 0.5, 1.0)
    pub const DEFAULT_NORMAL: Handle<Texture> = Handle::<Texture>::from_index(2);
}

pub fn initialize_textures(renderer: &mut crate::Renderer) -> koi_assets::AssetStore<Texture> {
    let texture = Texture(
        renderer
            .raw_graphics_context
            .new_texture(
                1,
                1,
                Some(&[255, 255, 255, 255]),
                kgraphics::PixelFormat::RGBA8Unorm,
                kgraphics::TextureSettings {
                    srgb: false,
                    ..Default::default()
                },
            )
            .unwrap(),
    );
    let mut textures = koi_assets::AssetStore::new(texture);

    textures.add_and_leak(
        Texture(
            renderer
                .raw_graphics_context
                .new_texture(
                    1,
                    1,
                    Some(&[0, 0, 0, 255]),
                    kgraphics::PixelFormat::RGBA8Unorm,
                    kgraphics::TextureSettings {
                        srgb: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ),
        &Texture::BLACK,
    );

    textures.add_and_leak(
        Texture(
            renderer
                .raw_graphics_context
                .new_texture(
                    1,
                    1,
                    Some(&[128, 128, 255, 255]),
                    kgraphics::PixelFormat::RGBA8Unorm,
                    kgraphics::TextureSettings {
                        srgb: false,
                        ..Default::default()
                    },
                )
                .unwrap(),
        ),
        &Texture::DEFAULT_NORMAL,
    );
    textures
}
