use cgmath::*;
use web_sys::*;

use crate::context::*;
use crate::framebuffer::*;
use crate::rect::*;

/// A trait for things that can be rendered to.
pub trait Surface {
    /// Binds the `Surface` and sets the appropriate viewport.
    #[doc(hidden)]
    fn bind(&self, context: &GlContext);

    /// Binds the `Surface` for reading. Doesn't modify the viewport.
    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext);

    /// Clears one or more buffers.
    ///
    /// Example usage:
    /// ```
    /// surface.clear(&context, &[ClearBuffer::Color([0.0, 0.0, 0.0, 0.0])]);
    /// ```
    fn clear(&self, context: &GlContext, buffers: &[ClearBuffer]) {
        assert!(!buffers.is_empty());
        self.bind(context);

        let mut bits = 0;
        for buffer in buffers {
            bits |= buffer.as_gl();

            if let Some(color) = buffer.color() {
                context.inner.clear_color(color[0], color[1], color[2], color[3]);
            }
        }

        context.inner.clear(bits);
    }

    /// Returns the size of the surface.
    fn size(&self) -> Vector2<u32>;
}

pub trait ClearColor {
    #[doc(hidden)]
    fn color(self) -> [f32; 4];
}

impl ClearColor for [f32; 4] {
    fn color(self) -> [f32; 4] {
        self
    }
}

#[derive(Copy, Clone)]
pub enum ClearBuffer {
    Color([f32; 4]),
    Depth,
}

impl ClearBuffer {
    fn as_gl(&self) -> u32 {
        match self {
            ClearBuffer::Color(_) => WebGl2::COLOR_BUFFER_BIT,
            ClearBuffer::Depth => WebGl2::DEPTH_BUFFER_BIT,
        }
    }

    fn color(&self) -> Option<[f32; 4]> {
        match self {
            ClearBuffer::Color(color) => Some(*color),
            _ => None,
        }
    }
}

/// A surface that represents the screen/default framebuffer.
pub struct ScreenSurface {
    viewport: Rect<i32>,
    size: Vector2<u32>,
    canvas: HtmlCanvasElement,
    id: FramebufferId,
}

impl ScreenSurface {
    pub(crate) fn new(canvas: HtmlCanvasElement) -> Self {
        let viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(canvas.width() as i32, canvas.height() as i32)),
        );
        let size = vec2(canvas.width(), canvas.height());
        ScreenSurface { viewport, size, canvas, id: FramebufferId::new() }
    }

    /// Resizes the canvas.
    pub fn set_size(&mut self, context: &GlContext, new_size: Vector2<u32>) {
        self.canvas.set_width(new_size.x);
        self.canvas.set_height(new_size.y);
        self.viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(new_size.x as i32, new_size.y as i32)),
        );
        self.size = new_size;
        // Resizing requires that we also change the viewport to match
        let cache = context.cache.borrow();
        if cache.bound_framebuffer == Some(self.id) {
            context.viewport(&self.viewport);
        }
    }

    /// Returns the canvas corresponding to this surface.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
}

impl Surface for ScreenSurface {
    #[doc(hidden)]
    fn bind(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_framebuffer != Some(self.id) {
            cache.bound_framebuffer = Some(self.id);
            context.inner.bind_framebuffer(WebGl2::DRAW_FRAMEBUFFER, None);
            context.viewport(&self.viewport);
        }
    }

    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext) {
        let mut cache = context.cache.borrow_mut();
        if cache.bound_read_framebuffer != Some(self.id) {
            cache.bound_read_framebuffer = Some(self.id);
            context.inner.bind_framebuffer(WebGl2::READ_FRAMEBUFFER, None);
        }
    }

    fn size(&self) -> Vector2<u32> {
        self.size
    }
}
