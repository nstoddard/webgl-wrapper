use js_sys::WebAssembly::Memory;
use js_sys::*;
use std::marker::PhantomData;
use wasm_bindgen::{memory, JsCast};
use web_sys::*;

use crate::context::*;
use crate::program::*;
use crate::surface::*;
use crate::uniforms::*;

/// An OpenGL primitive.
#[doc(hidden)]
pub trait Primitive {
    const AS_GL: u32;
}

#[derive(Copy, Clone, Debug)]
pub enum MeshUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
}

impl MeshUsage {
    fn as_gl(self) -> u32 {
        match self {
            MeshUsage::StaticDraw => WebGl2::STATIC_DRAW,
            MeshUsage::DynamicDraw => WebGl2::DYNAMIC_DRAW,
            MeshUsage::StreamDraw => WebGl2::STREAM_DRAW,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DrawMode {
    Draw2D,
    Draw3D,
}

impl DrawMode {
    fn bind(self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.draw_mode != Some(self) {
            cache.draw_mode = Some(self);

            match self {
                DrawMode::Draw2D => {
                    context.disable(GlFlag::CullFace);
                    context.disable(GlFlag::DepthTest);
                }
                DrawMode::Draw3D => {
                    context.enable(GlFlag::CullFace);
                    context.enable(GlFlag::DepthTest);
                }
            }
        }
    }
}

/// An index into a mesh.
pub type MeshIndex = u16;

/// A struct that builds a mesh from a collection of primitives.
///
/// This struct only stores the mesh data and indices; to use it in OpenGL, it must be used to
/// build a `Mesh`.
pub struct MeshBuilder<V: Vertex, P: Primitive> {
    vertex_data: Vec<f32>,
    indices: Vec<MeshIndex>,
    next_index: MeshIndex,
    phantom: PhantomData<(V, P)>,
}

impl<V: Vertex, P: Primitive> MeshBuilder<V, P> {
    pub fn new() -> Self {
        MeshBuilder { vertex_data: vec![], indices: vec![], next_index: 0, phantom: PhantomData }
    }

    /// Adds a vertex to the mesh. The vertex won't be rendered unless it's used in a primitive
    /// (currently either `Triangles` or `Lines`, each of which adds a method to this struct to
    /// add the corresponding primitive).
    pub fn vert(&mut self, vert: V) -> MeshIndex {
        assert!(self.next_index < MeshIndex::max_value());
        let index = self.next_index;
        self.next_index += 1;
        vert.add_to_mesh(&mut |data| self.vertex_data.push(data));
        index
    }

    /// Builds a `Mesh` from this `MeshBuilder`.
    pub fn build<U: GlUniforms>(
        &self,
        context: &GlContext,
        program: &GlProgram<V, U>,
        usage: MeshUsage,
        draw_mode: DrawMode,
    ) -> Mesh<V, U, P> {
        let mut mesh = Mesh::new(context, program, draw_mode);
        mesh.build_from(self, usage);
        mesh
    }

    /// Clears all data from the `MeshBuilder`. Does *not* reclaim the memory that had been used,
    /// so reusing the `MeshBuilder` won't have to reallocate unless the new mesh is larger than
    /// the old one.
    pub fn clear(&mut self) {
        self.vertex_data.clear();
        self.indices.clear();
        self.next_index = 0;
    }

    /// Adds all vertices and primitives from the other mesh to this mesh.
    pub fn extend(&mut self, other: MeshBuilder<V, P>) {
        let start_index = self.next_index;
        let num_verts = (other.vertex_data.len() / V::stride() as usize) as u16;
        let num_verts2 = other.next_index;
        // TODO: remove this
        assert_eq!(num_verts as usize * V::stride() as usize, other.vertex_data.len());
        assert_eq!(num_verts, num_verts2);
        self.next_index += num_verts;
        self.vertex_data.extend(other.vertex_data);
        self.indices.extend(other.indices.iter().map(|x| x + start_index));
    }
}

#[derive(Copy, Clone)]
pub struct Triangles;

impl Primitive for Triangles {
    const AS_GL: u32 = WebGl2::TRIANGLES;
}

impl<V: Vertex> MeshBuilder<V, Triangles> {
    /// Adds a triangle to the mesh.
    pub fn triangle(&mut self, a: MeshIndex, b: MeshIndex, c: MeshIndex) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }
}

#[derive(Copy, Clone)]
pub struct Lines;

impl Primitive for Lines {
    const AS_GL: u32 = WebGl2::LINES;
}

impl<V: Vertex> MeshBuilder<V, Lines> {
    /// Adds a line to the mesh.
    pub fn line(&mut self, a: MeshIndex, b: MeshIndex) {
        self.indices.push(a);
        self.indices.push(b);
    }
}

/// A mesh; built using a `MeshBuilder`.
pub struct Mesh<V: Vertex, U: GlUniforms, P: Primitive> {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    ibo: WebGlBuffer,
    context: GlContext,
    program: GlProgram<V, U>,
    num_indices: i32,
    phantom: PhantomData<P>,
    // TODO: can this be inferred from the vertex/uniforms types?
    draw_mode: DrawMode,
}

impl<V: Vertex, U: GlUniforms, P: Primitive> Drop for Mesh<V, U, P> {
    fn drop(&mut self) {
        self.context.inner.delete_vertex_array(Some(&self.vao));
        self.context.inner.delete_buffer(Some(&self.vbo));
        self.context.inner.delete_buffer(Some(&self.ibo));
    }
}

impl<V: Vertex, U: GlUniforms, P: Primitive> Mesh<V, U, P> {
    /// Creates an empty `Mesh`. It must have data written via `build_from` before it's usable.
    pub fn new(context: &GlContext, program: &GlProgram<V, U>, draw_mode: DrawMode) -> Self {
        let vao = context.inner.create_vertex_array().unwrap();
        context.inner.bind_vertex_array(Some(&vao));

        let vbo = context.inner.create_buffer().unwrap();
        let ibo = context.inner.create_buffer().unwrap();
        context.inner.bind_buffer(WebGl2::ARRAY_BUFFER, Some(&vbo));
        context.inner.bind_buffer(WebGl2::ELEMENT_ARRAY_BUFFER, Some(&ibo));

        Mesh {
            vao,
            vbo,
            ibo,
            context: context.clone(),
            program: program.clone(),
            num_indices: 0,
            phantom: PhantomData,
            draw_mode,
        }
    }

    /// Clears the mesh's current contents and updates it with the contents of the `MeshBuilder`.
    pub fn build_from(&mut self, builder: &MeshBuilder<V, P>, usage: MeshUsage) {
        self.num_indices = builder.indices.len() as i32;
        if self.num_indices == 0 {
            return;
        }

        self.bind();

        // TODO: if this function used buffer_sub_data, this would probably be unnecessary
        // and this method would be more efficient.
        let stride = V::stride();
        let mut offset = 0;
        for (attr, size) in V::ATTRIBUTES.iter() {
            let loc =
                self.context.inner.get_attrib_location(&self.program.inner.program, attr) as u32;
            self.context.inner.enable_vertex_attrib_array(loc);
            self.context.inner.vertex_attrib_pointer_with_i32(
                loc,
                *size,
                WebGl2::FLOAT,
                false,
                stride * 4,
                offset * 4,
            );
            offset += size;
        }

        let memory_buffer = memory().dyn_into::<Memory>().unwrap().buffer();

        let vertex_data_loc = builder.vertex_data.as_ptr() as u32 / 4;
        let vertex_array = Float32Array::new(&memory_buffer)
            .subarray(vertex_data_loc, vertex_data_loc + builder.vertex_data.len() as u32);
        self.context.inner.buffer_data_with_array_buffer_view(
            WebGl2::ARRAY_BUFFER,
            &vertex_array,
            usage.as_gl(),
        );

        let indices_loc = builder.indices.as_ptr() as u32 / 4;
        let index_array = Float32Array::new(&memory_buffer)
            .subarray(indices_loc, indices_loc + builder.indices.len() as u32);
        self.context.inner.buffer_data_with_array_buffer_view(
            WebGl2::ELEMENT_ARRAY_BUFFER,
            &index_array,
            usage.as_gl(),
        );
    }

    fn bind(&self) {
        self.context.inner.bind_vertex_array(Some(&self.vao));
    }

    /// Draws the mesh.
    pub fn draw(&self, surface: &impl Surface, uniforms: &impl Uniforms<GlUniforms = U>) {
        if self.num_indices == 0 {
            return;
        }

        // TODO: state caching
        self.bind();
        self.program.bind(&self.context);
        uniforms.update(&self.context, &self.program.inner.gl_uniforms);
        surface.bind(&self.context);
        self.draw_mode.bind(&self.context);

        self.context.inner.draw_elements_with_i32(
            P::AS_GL,
            self.num_indices,
            WebGl2::UNSIGNED_SHORT,
            0,
        );
    }
}
