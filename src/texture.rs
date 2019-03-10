use crate::context::*;
use cgmath::*;
use js_sys::WebAssembly::Memory;
use js_sys::*;
use uid::*;
use wasm_bindgen::{memory, JsCast, JsValue};
use web_sys::*;

#[doc(hidden)]
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub(crate) struct TextureId_(());

pub(crate) type TextureId = Id<TextureId_>;

// TODO: TextureFormat should support U8U8U8 etc, so they can take less space
#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    // Only the red component will be meaningful, the others are undefined.
    Red,
    RGB,
    RGBA,
    SRGB,
    SRGBA,
}

impl TextureFormat {
    pub(crate) fn to_gl_internal_format(self) -> u32 {
        match self {
            TextureFormat::Red => WebGl2::R8,
            TextureFormat::RGB => WebGl2::RGB8,
            TextureFormat::RGBA => WebGl2::RGBA8,
            TextureFormat::SRGB => WebGl2::SRGB8,
            TextureFormat::SRGBA => WebGl2::SRGB8_ALPHA8,
        }
    }

    fn to_gl_format(self) -> u32 {
        match self {
            TextureFormat::Red => WebGl2::RED,
            TextureFormat::RGB => WebGl2::RGB,
            TextureFormat::RGBA => WebGl2::RGBA,
            TextureFormat::SRGB => WebGl2::RGB,
            TextureFormat::SRGBA => WebGl2::RGBA,
        }
    }

    fn is_srgb(self) -> bool {
        match self {
            TextureFormat::SRGB | TextureFormat::SRGBA => true,
            _ => false,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear,
}

impl MinFilter {
    fn as_gl(self) -> u32 {
        match self {
            MinFilter::Nearest => WebGl2::NEAREST,
            MinFilter::Linear => WebGl2::LINEAR,
            MinFilter::NearestMipmapNearest => WebGl2::NEAREST_MIPMAP_NEAREST,
            MinFilter::NearestMipmapLinear => WebGl2::NEAREST_MIPMAP_LINEAR,
            MinFilter::LinearMipmapNearest => WebGl2::LINEAR_MIPMAP_NEAREST,
            MinFilter::LinearMipmapLinear => WebGl2::LINEAR_MIPMAP_LINEAR,
        }
    }

    fn has_mipmap(self) -> bool {
        match self {
            MinFilter::Nearest => false,
            MinFilter::Linear => false,
            MinFilter::NearestMipmapNearest => true,
            MinFilter::NearestMipmapLinear => true,
            MinFilter::LinearMipmapNearest => true,
            MinFilter::LinearMipmapLinear => true,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum MagFilter {
    Nearest,
    Linear,
}

impl MagFilter {
    fn as_gl(self) -> u32 {
        match self {
            MagFilter::Nearest => WebGl2::NEAREST,
            MagFilter::Linear => WebGl2::LINEAR,
        }
    }
}
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum WrapMode {
    ClampToEdge,
    Repeat,
}

impl WrapMode {
    fn as_gl(self) -> u32 {
        match self {
            WrapMode::ClampToEdge => WebGl2::CLAMP_TO_EDGE,
            WrapMode::Repeat => WebGl2::REPEAT,
        }
    }
}

/// A 2D texture.
pub struct Texture2d {
    pub(crate) texture: WebGlTexture,
    pub(crate) size: Vector2<u32>,
    id: TextureId,
    pub(crate) context: GlContext,
    is_srgb: bool,
}

impl Drop for Texture2d {
    fn drop(&mut self) {
        self.context.inner.delete_texture(Some(&self.texture));
    }
}

impl Texture2d {
    /// Creates an empty `Texture2d`. Should typically be rendered to with a `Framebuffer`.
    pub fn empty(
        context: &GlContext,
        size: Vector2<u32>,
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        // TODO: add a method to generate mipmaps after data has been written to the texture
        assert!(!min_filter.has_mipmap());

        let texture = context.inner.create_texture().unwrap();
        context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&texture));
        context
            .inner
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                size.x as i32,
                size.y as i32,
                0,
                format.to_gl_format(),
                WebGl2::UNSIGNED_BYTE,
                None,
            )
            .unwrap();
        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size,
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    /// Creates a `Texture2d` from an `HtmlImageElement`.
    pub fn from_image(
        context: &GlContext,
        image: &HtmlImageElement,
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = context.inner.create_texture().unwrap();
        context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&texture));

        context
            .inner
            .tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                format.to_gl_format(),
                WebGl2::UNSIGNED_BYTE,
                image,
            )
            .unwrap();

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size: vec2(image.width(), image.height()),
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    /// Creates a `Texture2d` from data.
    pub fn from_data(
        context: &GlContext,
        size: Vector2<u32>,
        data: &[u8],
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = context.inner.create_texture().unwrap();
        context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&texture));

        context
            .inner
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                size.x as i32,
                size.y as i32,
                0,
                format.to_gl_format(),
                WebGl2::UNSIGNED_BYTE,
                Some(data),
            )
            .unwrap();

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self {
            texture,
            size,
            id: TextureId::new(),
            context: context.clone(),
            is_srgb: format.is_srgb(),
        }
    }

    pub fn set_contents(&self, format: TextureFormat, data: &[u8]) {
        // TODO: remove texture unit parameter
        self.bind(0);
        self.context.inner.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
            WebGl2::TEXTURE_2D,
            0,
            0,
            0,
            self.size.x as i32,
            self.size.y as i32,
            format.to_gl_format(),
            WebGl2::UNSIGNED_BYTE,
            Some(data),
            ).unwrap();
    }

    fn set_tex_parameters(
        context: &GlContext,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) {
        context.inner.tex_parameteri(
            WebGl2::TEXTURE_2D,
            WebGl2::TEXTURE_MIN_FILTER,
            min_filter.as_gl() as i32,
        );
        context.inner.tex_parameteri(
            WebGl2::TEXTURE_2D,
            WebGl2::TEXTURE_MAG_FILTER,
            mag_filter.as_gl() as i32,
        );
        context.inner.tex_parameteri(
            WebGl2::TEXTURE_2D,
            WebGl2::TEXTURE_WRAP_S,
            wrap_mode.as_gl() as i32,
        );
        context.inner.tex_parameteri(
            WebGl2::TEXTURE_2D,
            WebGl2::TEXTURE_WRAP_T,
            wrap_mode.as_gl() as i32,
        );

        if min_filter.has_mipmap() {
            context.inner.generate_mipmap(WebGl2::TEXTURE_2D);
        }
    }

    pub(crate) fn bind(&self, texture_unit: u32) {
        let mut cache = self.context.cache.borrow_mut();
        if cache.bound_textures[texture_unit as usize] != Some((WebGl2::TEXTURE_2D, self.id)) {
            cache.bound_textures[texture_unit as usize] = Some((WebGl2::TEXTURE_2D, self.id));
            self.context.inner.active_texture(WebGl2::TEXTURE0 + texture_unit);
            self.context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&self.texture));
        }
    }

    /// True if the image uses an sRGB format.
    pub fn is_srgb(&self) -> bool {
        self.is_srgb
    }
}
