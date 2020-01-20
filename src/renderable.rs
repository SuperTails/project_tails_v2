use sdl2::render::{Canvas, RenderTarget};

pub trait Renderable {
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), String>;
}
