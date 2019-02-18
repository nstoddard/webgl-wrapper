use std::slice;
use web_sys::*;

use crate::context::*;
use crate::texture::*;

/// Holds uniforms for a given program.
///
/// Example implementation:
/// ```
/// struct ExampleUniforms<'a> {
///     matrix: Matrix4<f32>,
///     tex: &'a Texture2d,
/// }
///
/// struct ExampleUniformsGl {
///     matrix: Matrix4Uniform,
///     tex: TextureUniform,
/// }
///
/// impl<'a> Uniforms for ExampleUniforms<'a> {
///     type GlUniforms = ExampleUniformsGl;
///
///     fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms) {
///         gl_uniforms.matrix.set(context, &self.matrix);
///         gl_uniforms.tex.set(context, self.tex, 0);
///     }
/// }
///
/// impl GlUniforms for ExampleUniformsGl {
///     fn new(context: &GlContext, program: &WebGlProgram) -> Self {
///         ExampleUniformsGl {
///             matrix: Matrix4Uniform::new("matrix", context, program),
///             tex: TextureUniform::new("tex", context, program),
///         }
///     }
/// }
/// ```
pub trait Uniforms {
    /// The `GlUniforms` instance corresponding to this `Uniforms`.
    type GlUniforms: GlUniforms;

    /// Updates the given `GlUniforms` from this `Uniforms`. Should call `set` on each uniform
    /// in the associated `GlUniforms`.
    fn update(&self, context: &GlContext, gl_uniforms: &Self::GlUniforms);
}

/// A type used to hold the uniform locations, which can be updated from a corresponding instance of the `Uniforms` trait.
///
/// See the `Uniforms` trait for an example implementation.
pub trait GlUniforms {
    fn new(context: &GlContext, program: &WebGlProgram) -> Self;
}

// TODO: these structs are probably redundant
pub struct Matrix4Uniform {
    loc: WebGlUniformLocation,
}

impl Matrix4Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, mat: &impl AsRef<[f32; 16]>) {
        // Unsafe is necessary because from_raw_parts_mut is needed to construct a slice from a Mat4 (which is safe because Mat4 is repr(C))
        context.inner.uniform_matrix4fv_with_f32_array(Some(&self.loc), false, unsafe {
            slice::from_raw_parts_mut(mat.as_ref() as *const f32 as *mut f32, 16)
        });
    }
}

pub struct TextureUniform {
    loc: WebGlUniformLocation,
}

impl TextureUniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, texture: &Texture2d, texture_unit: u32) {
        context.inner.uniform1i(Some(&self.loc), texture_unit as i32);
        texture.bind(context, texture_unit);
    }
}

pub struct Vector2Uniform {
    loc: WebGlUniformLocation,
}

impl Vector2Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 2]>) {
        let val = val.as_ref();
        context.inner.uniform2f(Some(&self.loc), val[0], val[1]);
    }
}

pub struct Vector3Uniform {
    loc: WebGlUniformLocation,
}

impl Vector3Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 3]>) {
        let val = val.as_ref();
        context.inner.uniform3f(Some(&self.loc), val[0], val[1], val[2]);
    }
}

pub struct Vector4Uniform {
    loc: WebGlUniformLocation,
}

impl Vector4Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: &impl AsRef<[f32; 4]>) {
        let val = val.as_ref();
        context.inner.uniform4f(Some(&self.loc), val[0], val[1], val[2], val[3]);
    }
}

pub struct Array2Uniform {
    loc: WebGlUniformLocation,
}

impl Array2Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 2]) {
        context.inner.uniform2f(Some(&self.loc), val[0], val[1]);
    }
}

pub struct Array3Uniform {
    loc: WebGlUniformLocation,
}

impl Array3Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 3]) {
        context.inner.uniform3f(Some(&self.loc), val[0], val[1], val[2]);
    }
}

pub struct Array4Uniform {
    loc: WebGlUniformLocation,
}

impl Array4Uniform {
    pub fn new(name: &str, context: &GlContext, program: &WebGlProgram) -> Self {
        Self { loc: context.inner.get_uniform_location(program, name).unwrap() }
    }

    // TODO: guarantee that the program is bound when this is called
    pub fn set(&self, context: &GlContext, val: [f32; 4]) {
        context.inner.uniform4f(Some(&self.loc), val[0], val[1], val[2], val[3]);
    }
}
