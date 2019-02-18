use wasm_bindgen::JsCast;
use web_sys::*;

use crate::rect::*;
use crate::surface::*;

pub(crate) type WebGl2 = WebGl2RenderingContext;

/// A WebGL context.
#[derive(Clone)]
pub struct GlContext {
    pub(crate) inner: WebGl2RenderingContext,
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

        Ok((GlContext { inner: context }, ScreenSurface::new(canvas)))
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
