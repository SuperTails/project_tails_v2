use crate::asset_mgr::GraphicsHolder;
use crate::physical::Vector2;
use crate::renderable::Renderable;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use std::time::{Duration, Instant};
use crate::camera::Camera;

/// The animation class itself does not implement renderable,
/// only for `(Vector2, &Animation)`. This is because
/// an animation requires a position.
pub struct Animation {
    spritesheet: String,
    frame_count: usize,
    delay: Duration,
    frame: usize,
    last_change: Instant,
}

impl Animation {
    /// Constructs a new animation
    pub fn new(spritesheet: String, frame_count: usize, delay: Duration) -> Animation {
        Animation {
            spritesheet,
            frame_count,
            delay,
            frame: 0,
            last_change: Instant::now(),
        }
    }

    /// Possibly advances which frame the animation is on,
    /// depending on whether enough time has passed.
    pub fn update(&mut self) {
        if self.last_change.elapsed() > self.delay {
            self.frame += 1;
            self.frame %= self.frame_count;
            self.last_change = Instant::now();
        }
    }

    fn frame_window(&self) -> Rect {
        let guard = GraphicsHolder::get();
        let pair = guard.get_pair(&self.spritesheet).unwrap();

        let full_width = pair.0.width();
        let height = pair.0.height();

        let width = full_width / self.frame_count as u32;

        let x = width as i32 * self.frame as i32;
        let y = 0;

        Rect::new(x, y, width, height)
    }
}

impl Renderable for (Vector2, &Animation) {
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, camera: &Camera) -> Result<(), String> {
        let pos = Vector2 { x: self.0.x - camera.position.x, y: self.0.y - camera.position.y };
        let pos = pos.into_point();
        let anim = self.1;

        let source = anim.frame_window();
        let dest = Rect::new(pos.x(), pos.y(), source.width(), source.height());

        let guard = GraphicsHolder::get();
        let pair = guard.get_pair(&anim.spritesheet).unwrap();

        canvas.copy(&pair.1, Some(source), Some(dest))
    }
}
