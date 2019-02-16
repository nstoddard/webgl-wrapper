use crate::context::*;
use cgmath::*;
use js_sys::WebAssembly::Memory;
use js_sys::*;
use wasm_bindgen::{memory, JsCast, JsValue};
use web_sys::*;

// TODO: TextureFormat should support U8U8U8 etc, so they can take less space
#[derive(Copy, Clone, Debug)]
pub enum TextureFormat {
    // Only the red component will be meaningful, the others are undefined.
    Red,
    RGB,
    RGBA,
}

impl TextureFormat {
    pub(crate) fn to_gl_internal_format(self) -> u32 {
        match self {
            TextureFormat::Red => WebGl2::R8,
            TextureFormat::RGB => WebGl2::RGB8,
            TextureFormat::RGBA => WebGl2::RGBA8,
        }
    }

    fn to_gl_format(self) -> u32 {
        match self {
            TextureFormat::Red => WebGl2::RED,
            TextureFormat::RGB => WebGl2::RGB,
            TextureFormat::RGBA => WebGl2::RGBA,
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

/// A data type that can be used as texture data.
pub trait TextureData {
    fn subarray(memory_buffer: &JsValue, data_loc: u32, data_len: u32) -> Object;

    const GL_TYPE: u32;

    const SIZE: u32;
}

impl TextureData for u8 {
    fn subarray(memory_buffer: &JsValue, data_loc: u32, data_len: u32) -> Object {
        // TODO: see if there's a better way to do this
        (*Uint8Array::new(&memory_buffer).subarray(data_loc, data_loc + data_len)).clone()
    }

    const GL_TYPE: u32 = WebGl2::UNSIGNED_BYTE;

    const SIZE: u32 = 1;
}

// TODO: this doesn't currently work, because the internal format has to be changed to match
/*impl TextureData for f32 {
    fn subarray(memory_buffer: &JsValue, data_loc: u32, data_len: u32) -> Object {
        // TODO: see if there's a better way to do this
        (*Float32Array::new(&memory_buffer).subarray(data_loc, data_loc + data_len)).clone()
    }

    const GL_TYPE: u32 = WebGl2::FLOAT;

    const SIZE: u32 = 4;
}*/

/// A 2D texture.
pub struct Texture2d {
    pub(crate) texture: WebGlTexture,
    pub(crate) size: Vector2<u32>,
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
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
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

        Self { texture, size }
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
                u8::GL_TYPE,
                image,
            )
            .unwrap();

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self { texture, size: vec2(image.width(), image.height()) }
    }

    /// Creates a `Texture2d` from data.
    pub fn from_data<T: TextureData>(
        context: &GlContext,
        size: Vector2<u32>,
        data: &[T],
        format: TextureFormat,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        wrap_mode: WrapMode,
    ) -> Self {
        let texture = context.inner.create_texture().unwrap();
        context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&texture));

        let memory_buffer = memory().dyn_into::<Memory>().unwrap().buffer();

        let data_loc = data.as_ptr() as u32 / T::SIZE;
        let array = T::subarray(&memory_buffer, data_loc, data.len() as u32);

        context
            .inner
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                WebGl2::TEXTURE_2D,
                0,
                format.to_gl_internal_format() as i32,
                size.x as i32,
                size.y as i32,
                0,
                format.to_gl_format(),
                T::GL_TYPE,
                Some(&array),
            )
            .unwrap();

        Self::set_tex_parameters(context, min_filter, mag_filter, wrap_mode);

        Self { texture, size }
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

    pub(crate) fn bind(&self, context: &GlContext, texture_unit: u32) {
        // TODO: state caching
        context.inner.active_texture(WebGl2::TEXTURE0 + texture_unit);
        context.inner.bind_texture(WebGl2::TEXTURE_2D, Some(&self.texture));
    }
}
