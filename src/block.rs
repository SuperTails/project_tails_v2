use gamefox::asset_mgr::GraphicsHolder;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::rect::{Rect, Point};
use serde::{Deserialize, Serialize};
use serde_json::de::from_reader;
use serde_json::Result as JsonResult;
use std::fs::File;
use std::path::Path;
use sdl2::pixels::PixelFormatEnum;

const BLOCK_TILE_LENGTH: usize = 8;
const TILE_PIXEL_LENGTH: usize = 16;
const BLOCK_PIXEL_LENGTH: usize = BLOCK_TILE_LENGTH * TILE_PIXEL_LENGTH;

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
struct RawBlock {
    layers: Vec<Layer>
}

pub struct Block {
    pub graphics_layers: Vec<Layer>,
    pub collision_layers: Vec<Layer>,
}

impl From<RawBlock> for Block {
    fn from(raw: RawBlock) -> Block {
        let mut graphics = Vec::new();
        let mut collision = Vec::new();

        for layer in raw.layers {
            if layer.name.starts_with("Graphics") {
                graphics.push(layer);
            } else if layer.name.starts_with("Collision") {
                collision.push(layer);
            } else {
                println!("Warning: Ambiguous layer name {}", layer.name);
                if layer.tiles.iter().any(|t| t.tile >= 340) {
                    collision.push(layer);
                } else {
                    graphics.push(layer);
                }
            }
        }

        assert!(graphics.len() == 1 || graphics.len() == 2, "Len: {}", graphics.len());
        assert!(collision.len() == 1 || collision.len() == 2, "Len: {}", collision.len());

        for layer in graphics.iter() {
            assert_eq!(layer.tiles.len(), 64);
        }

        for layer in collision.iter() {
            assert_eq!(layer.tiles.len(), 64);
        }

        Block {
            graphics_layers: graphics,
            collision_layers: collision,
        }
    }
}

type CollisionTile = ([u32; 16], u8);

pub struct TerrainGetter<'a> {
    width: usize,
    block_map: &'a [Option<(usize, u32)>],
    blocks: &'a [Block],
    tiles: &'a [CollisionTile]
}

impl<'a> TerrainGetter<'a> {
    pub fn new(
        width: usize,
        block_map: &'a [Option<(usize, u32)>],
        blocks: &'a [Block],
        tiles: &'a [CollisionTile],
    ) -> TerrainGetter<'a> {
        TerrainGetter {
            width,
            block_map,
            blocks,
            tiles,
        }
    }

    pub fn is_occupied(&self, x: i32, y: i32, layer_idx: usize) -> bool {
        if let Some(tile) = self.tile_at(x, y, layer_idx) {
            let pixel_x = x as usize % TILE_PIXEL_LENGTH;
            let pixel_y = y as usize % TILE_PIXEL_LENGTH;

            let column = tile.0[pixel_x];
            // This conveniently works
            pixel_y + column as usize >= TILE_PIXEL_LENGTH
        } else {
            false
        }
    }

    pub fn tile_at(&self, x: i32, y: i32, layer_idx: usize) -> Option<CollisionTile> {
        if x < 0 || y < 0 || x >= (self.width * BLOCK_PIXEL_LENGTH) as i32 || y >= (self.block_map.len() / self.width) as i32 * 128 {
            return None;
        }

        let block_x = x as usize / BLOCK_PIXEL_LENGTH;
        let block_y = y as usize / BLOCK_PIXEL_LENGTH;
        let mut tile_x = (x as usize % BLOCK_PIXEL_LENGTH) / TILE_PIXEL_LENGTH;
        let tile_y = (y as usize % BLOCK_PIXEL_LENGTH) / TILE_PIXEL_LENGTH;

        let (block_idx, block_flags) = self.block_map[block_y * self.width + block_x]?;
        let block = &self.blocks[block_idx];

        // TODO: Allow for more flags
        if block_flags != 0 {
            tile_x = BLOCK_TILE_LENGTH - 1 - tile_x;
        }

        let layer = &block.collision_layers[layer_idx];

        let tile_entry = &layer.tiles[tile_y * 8 + tile_x];
        
        let mut tile = self.tiles[tile_entry.tile - 340];

        // TODO: Adjust angle, allow for vertical flip
        if (block_flags != 0) ^ tile_entry.flip {
            tile.0.reverse();
        }
        
        Some(tile)
    }
}

impl Block {
    fn add_graphics_cached<T>(
        name: String,
        block: &Block,
        creator: &'static TextureCreator<T>,
        tileset: &Tileset,
        horiz_flip: &Surface,
        vert_flip: &Surface,
        both_flip: &Surface,
    ) {
        let mut holder = GraphicsHolder::get();
        let tileset_gfx = holder.get_pair(&tileset.image).unwrap();

        let mut target = Surface::new(BLOCK_PIXEL_LENGTH as u32, BLOCK_PIXEL_LENGTH as u32, PixelFormatEnum::RGBA8888).unwrap();
        
        for layer in block.graphics_layers.iter().rev() {
            for (idx, tile) in layer.tiles.iter().enumerate() {
                let row = idx / BLOCK_TILE_LENGTH;
                let col = idx % BLOCK_TILE_LENGTH;

                let mut src = Rect::new(
                    ((tile.tile % tileset.tiles_per_row) * TILE_PIXEL_LENGTH) as i32,
                    ((tile.tile / tileset.tiles_per_row) * TILE_PIXEL_LENGTH) as i32,
                    TILE_PIXEL_LENGTH as u32,
                    TILE_PIXEL_LENGTH as u32,
                );

                let dst = Rect::new(
                    (col * TILE_PIXEL_LENGTH) as i32,
                    (row * TILE_PIXEL_LENGTH) as i32,
                    TILE_PIXEL_LENGTH as u32,
                    TILE_PIXEL_LENGTH as u32,
                );

                let src_map = if tile.rot == 2 && tile.flip {
                    src.set_y(tileset_gfx.0.height() as i32 - TILE_PIXEL_LENGTH as i32 - src.y());
                    &vert_flip
                } else if tile.flip {
                    src.set_x(tileset_gfx.0.width() as i32 - TILE_PIXEL_LENGTH as i32 - src.x());
                    &horiz_flip
                } else if tile.rot == 2 {
                    src.set_x(tileset_gfx.0.width() as i32 - TILE_PIXEL_LENGTH as i32 - src.x());
                    src.set_y(tileset_gfx.0.height() as i32 - TILE_PIXEL_LENGTH as i32 - src.y());
                    &both_flip
                } else {
                    &tileset_gfx.0
                };

                src_map.blit(src, &mut target, dst).unwrap();
            }
        }

        let tex = creator.create_texture_from_surface(&target).unwrap();

        holder.0.insert(name, (target, tex));
    }

    pub fn add_graphics_multi<'a, T, U>(
        input: U,
        tileset: &Tileset,
        creator: &'static TextureCreator<T>,
    ) where U: Iterator<Item = (String, &'a Block)> {
        let holder = GraphicsHolder::get();

        let tileset_gfx = holder.get_pair(&tileset.image).unwrap();
        let horiz_flip = flip(&tileset_gfx.0, true, false);
        let vert_flip = flip(&tileset_gfx.0, false, true);
        let both_flip = flip(&tileset_gfx.0, true, true);
        std::mem::drop(holder);

        for (name, block) in input {
            Block::add_graphics_cached(
                name,
                block,
                creator,
                tileset,
                &horiz_flip,
                &vert_flip,
                &both_flip,
            )
        }
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
            let raw: RawBlock = from_reader(f)?;
            result.push(raw.into());
        } else {
            break;
        }
    }

    Ok(result)
}

pub fn flip(surface: &Surface, horiz: bool, vertical: bool) -> Surface<'static> {
    let mut dest = Surface::new(surface.width(), surface.height(), surface.pixel_format_enum()).unwrap();

    for r in 0..surface.height() {
        for c in 0..surface.width() {
            let new_r = if vertical {
                surface.height() - 1 - r
            } else {
                r
            };

            let new_c = if horiz {
                surface.width() - 1 - c
            } else {
                c
            };

            let color = get_pixel(surface, Point::new(c as i32, r as i32));
            set_pixel(&mut dest, Point::new(new_c as i32, new_r as i32), color);
        }
    }

    dest
}

pub fn parse_collision_map(map_name: &str) -> Vec<([u32; 16], u8)> {
    read_collision_map(&GraphicsHolder::get().get_pair(map_name).unwrap().0)
}

fn read_collision_map(map: &Surface) -> Vec<([u32; 16], u8)> {
    let mut result = Vec::new();

    for y in (0..map.height() / 16).map(|y| y * 16) {
        for x in (0..map.width() / 16).map(|x| x * 16) {
            result.push(read_collision_tile(map, Point::new(x as i32, y as i32)));
            if result.len() == 256 {
                return result;
            }
        }
    }

    result
}

fn get_pixel(map: &Surface, point: Point) -> u32 {
    let bpp = map.pixel_format_enum().into_masks().unwrap().bpp;
    map.with_lock(|pixels| {
        let idx = (point.y() as u32 * map.pitch() + point.x() as u32 * bpp as u32 / 8) as usize;
        let pixel = &pixels[idx..][..4];
        let mut temp = [0; 4];
        temp.copy_from_slice(pixel);
        u32::from_ne_bytes(temp)
    })
}

fn set_pixel(map: &mut Surface, point: Point, color: u32) {
    let bpp = map.pixel_format_enum().into_masks().unwrap().bpp;
    let pitch = map.pitch();
    map.with_lock_mut(|pixels| {
        let idx = (point.y() as u32 * pitch + point.x() as u32 * bpp as u32 / 8) as usize;
        let pixel = &mut pixels[idx..][..4];
        let result = color.to_ne_bytes();
        pixel.copy_from_slice(&result);
    })
}

fn read_collision_tile(map: &Surface, top_left: Point) -> ([u32; 16], u8) {
    let masks = map.pixel_format_enum().into_masks().unwrap();
    let color_mask = masks.rmask | masks.gmask | masks.bmask;
    let g_shift = masks.gmask.trailing_zeros();

    let mut result = [0; 16];
    let mut angle = 0;
    for (col, x) in (top_left.x()..top_left.x() + 16).enumerate() {
        for y in top_left.y()..top_left.y() + 16 {
            let color = get_pixel(map, Point::new(x, y));
            if color & color_mask != 0 {
                angle = ((color & masks.gmask) >> g_shift) as u8;
                result[col] = 16 - (y - top_left.y()) as u32;
                break;
            }
        }
    }

    (result, angle)
}