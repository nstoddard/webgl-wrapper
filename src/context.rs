use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::*;

use crate::framebuffer::*;
use crate::mesh::*;
use crate::program::*;
use crate::rect::*;
use crate::surface::*;
use crate::texture::*;

pub(crate) type WebGl2 = WebGl2RenderingContext;

/// A WebGL context.
#[derive(Clone)]
pub struct GlContext {
    pub(crate) inner: WebGl2RenderingContext,
    pub(crate) cache: Rc<RefCell<GlContextCache>>,
    // A VAO/VBO that is currently used for all instanced rendering
    // TODO: this isn't suitable for all cases of instanced rendering; some apps will want to
    // use static data for the instances rather than recreating them each frame.
    pub(crate) instanced_vao: WebGlVertexArrayObject,
    pub(crate) instanced_vbo: WebGlBuffer,
}

pub(crate) struct GlContextCache {
    pub draw_mode: Option<DrawMode>,
    pub bound_program: Option<ProgramId>,
    pub bound_framebuffer: Option<FramebufferId>,
    pub bound_read_framebuffer: Option<FramebufferId>,
    pub bound_textures: [Option<(u32, TextureId)>; 32],
}

impl GlContextCache {
    fn new() -> Self {
        Self {
            draw_mode: None,
            bound_program: None,
            bound_framebuffer: None,
            bound_read_framebuffer: None,
            bound_textures: [None; 32],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum GlFlag {
    DepthTest,
    CullFace,
}

impl GlFlag {
    fn as_gl(self) -> u32 {
        match self {
            GlFlag::DepthTest => WebGl2::DEPTH_TEST,
            GlFlag::CullFace => WebGl2::CULL_FACE,
        }
    }
}

impl GlContext {
    /// Creates a `GlContext` and associated surface.
    ///
    /// Returns an error if the WebGl 2 context couldn't be created.
    pub fn new(canvas_id: &str) -> Result<(Self, ScreenSurface), Box<&str>> {
        let document = window().unwrap().document().unwrap();
        let canvas =
            document.get_element_by_id(canvas_id).unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context_with_context_options(
                "webgl2",
                WebGlContextAttributes::new().antialias(true).as_ref(),
            )
            .expect("Unable to create canvas")
            .ok_or("Unable to create canvas")?
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        context.enable(WebGl2::BLEND);
        context.blend_func(WebGl2::ONE, WebGl2::ONE_MINUS_SRC_ALPHA);
        context.pixel_storei(WebGl2::UNPACK_ALIGNMENT, 1);

        let instanced_vao = context.create_vertex_array().unwrap();
        context.bind_vertex_array(Some(&instanced_vao));
        let instanced_vbo = context.create_buffer().unwrap();
        context.bind_buffer(WebGl2::ARRAY_BUFFER, Some(&instanced_vbo));

        Ok((
            GlContext {
                inner: context,
                cache: Rc::new(RefCell::new(GlContextCache::new())),
                instanced_vao,
                instanced_vbo,
            },
            ScreenSurface::new(canvas),
        ))
    }

    pub(crate) fn viewport(&self, viewport: &Rect<i32>) {
        self.inner.viewport(
            viewport.start.x,
            viewport.start.y,
            viewport.end.x - viewport.start.x,
            viewport.end.y - viewport.start.y,
        );
    }

    pub(crate) fn enable(&self, flag: GlFlag) {
        self.inner.enable(flag.as_gl());
    }

    pub(crate) fn disable(&self, flag: GlFlag) {
        self.inner.disable(flag.as_gl());
    }
}
