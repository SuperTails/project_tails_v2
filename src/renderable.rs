use sdl2::render::{Canvas, RenderTarget};
use crate::camera::Camera;

pub trait Renderable {
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, camera: &Camera) -> Result<(), String>;
}
