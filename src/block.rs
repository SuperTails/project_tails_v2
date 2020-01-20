use crate::asset_mgr::GraphicsHolder;
use sdl2::render::{WindowCanvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use serde_json::de::from_reader;
use serde_json::Result as JsonResult;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    rot: u32,
    #[serde(rename = "flipX")]
    flip: bool,
    tile: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layer {
    name: String,
    tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    layers: Vec<Layer>,
}

impl Block {
    pub fn add_graphics<T>(
        name: String,
        block: &Block,
        tileset: &Tileset,
        canvas: &WindowCanvas,
        creator: &'static TextureCreator<T>,
    ) {
        let mut holder = GraphicsHolder::get();
        let tileset_gfx = holder.get_pair(&tileset.image).unwrap();

        let mut target = Surface::new(128, 128, canvas.default_pixel_format()).unwrap();
        
        for layer in block.layers.iter() {
            for (idx, tile) in layer.tiles.iter().enumerate() {
                let row = idx / 16;
                let col = idx % 16;

                let src = Rect::new(
                    (tile.tile % tileset.tiles_per_row) as i32 * 16,
                    (tile.tile / tileset.tiles_per_row) as i32 * 16,
                    16,
                    16
                );

                if src.y() as u32 >= tileset_gfx.0.height() {
                    continue;
                }

                let dst = Rect::new(
                    col as i32 * 16,
                    row as i32 * 16,
                    16,
                    16
                );

                tileset_gfx.0.blit(src, &mut target, dst).unwrap();
            }
        }

        let tex = creator.create_texture_from_surface(&target).unwrap();

        holder.0.insert(name, (target, tex));
    }
}

pub struct Tileset {
    pub tiles_per_row: usize,
    pub image: String,
}

// TODO: Use actual proper asset path
pub fn load_blocks(directory: &Path, prefix: &str) -> JsonResult<Vec<Block>> {
    let mut result = Vec::new();

    for i in 1.. {
        if let Ok(f) = File::open(directory.join(format!("{}{}.json", prefix, i))) {
            result.push(from_reader(f)?);
        } else {
            break;
        }
    }

    Ok(result)
}