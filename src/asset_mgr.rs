use once_cell::sync::OnceCell;
use sdl2::image::LoadSurface;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

pub struct GraphicsHolder<'a>(pub HashMap<String, (Surface<'a>, Texture<'a>)>);

impl<'a> GraphicsHolder<'a> {
    pub fn get() -> MutexGuard<'static, GraphicsHolder<'static>> {
        GRAPHICS_HOLDER.get().unwrap().lock().unwrap()
    }

    pub fn get_pair(&self, id: &str) -> Option<&(Surface<'a>, Texture<'a>)> {
        self.0.get(id)
    }

    pub fn load_all<T>(dir: &Path, creator: &'a TextureCreator<T>) -> GraphicsHolder<'a> {
        let mut holder = GraphicsHolder(HashMap::new());
        let mut queue: Vec<DirEntry> = dir.read_dir().unwrap().map(Result::unwrap).collect();

        while let Some(file) = queue.pop() {
            let file_type = file.file_type().unwrap();
            if file_type.is_dir() {
                queue.extend(file.path().read_dir().unwrap().map(Result::unwrap));
            } else if file_type.is_file() && file.path().extension() == Some(OsStr::new("png")) {
                let surface = Surface::from_file(file.path()).unwrap();
                let texture = creator.create_texture_from_surface(&surface).unwrap();

                let prefix = dir.to_string_lossy().len();
                let name = file.path().to_string_lossy()[prefix..].to_string();
                let name = name[..name.len() - ".png".len()].to_string();

                holder.0.insert(name, (surface, texture));
            }
        }

        holder
    }
}

unsafe impl Send for GraphicsHolder<'static> {}

pub static GRAPHICS_HOLDER: OnceCell<Mutex<GraphicsHolder<'static>>> = OnceCell::new();
