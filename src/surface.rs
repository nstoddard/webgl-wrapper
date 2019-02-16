use cgmath::*;
use web_sys::*;

use crate::context::*;
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
    fn clear<Color>(&self, context: &GlContext, buffers: &[ClearBuffer<Color>])
    where
        Color: Into<[f32; 4]> + Copy,
    {
        assert!(!buffers.is_empty());
        self.bind(context);

        let mut bits = 0;
        for buffer in buffers {
            bits |= buffer.as_gl();

            if let ClearBuffer::Color(color) = buffer {
                let color: [f32; 4] = (*color).into();
                context.inner.clear_color(color[0], color[1], color[2], color[3]);
            }
        }

        context.inner.clear(bits);
    }

    /// Returns the size of the surface.
    fn size(&self) -> Vector2<u32>;
}

#[derive(Copy, Clone)]
pub enum ClearBuffer<Color>
where
    Color: Into<[f32; 4]>,
{
    Color(Color),
    Depth,
}

impl<Color: Into<[f32; 4]>> ClearBuffer<Color> {
    fn as_gl(self) -> u32 {
        match self {
            ClearBuffer::Color(_) => WebGl2::COLOR_BUFFER_BIT,
            ClearBuffer::Depth => WebGl2::DEPTH_BUFFER_BIT,
        }
    }
}

/// A surface that represents the screen/default framebuffer.
pub struct ScreenSurface {
    viewport: Rect<i32>,
    size: Vector2<u32>,
    canvas: HtmlCanvasElement,
}

impl ScreenSurface {
    pub(crate) fn new(canvas: HtmlCanvasElement) -> Self {
        let viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(canvas.width() as i32, canvas.height() as i32)),
        );
        let size = vec2(canvas.width(), canvas.height());
        ScreenSurface { viewport, size, canvas }
    }

    /// Resizes the canvas.
    pub fn set_size(&mut self, new_size: Vector2<u32>) {
        self.canvas.set_width(new_size.x);
        self.canvas.set_height(new_size.y);
        self.viewport = Rect::new(
            Point2::origin(),
            Point2::from_vec(vec2(new_size.x as i32, new_size.y as i32)),
        );
        self.size = new_size;
    }
}

impl Surface for ScreenSurface {
    #[doc(hidden)]
    fn bind(&self, context: &GlContext) {
        context.inner.bind_framebuffer(WebGl2::DRAW_FRAMEBUFFER, None);
        context.viewport(&self.viewport);
    }

    #[doc(hidden)]
    fn bind_read(&self, context: &GlContext) {
        context.inner.bind_framebuffer(WebGl2::READ_FRAMEBUFFER, None);
    }

    fn size(&self) -> Vector2<u32> {
        self.size
    }
}
