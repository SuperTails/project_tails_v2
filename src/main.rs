mod act;
mod animation;
mod asset_mgr;
mod block;
mod physical;
mod player;
mod renderable;

use act::ActFile;
use asset_mgr::{GraphicsHolder, GRAPHICS_HOLDER};
use player::Player;
use renderable::Renderable;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use block::{Block, Tileset};

fn main() {
    let blocks = block::load_blocks(Path::new("./assets/EmeraldHillZone/"), "Block").unwrap();
    println!("Loaded {} blocks", blocks.len());

    let tileset = Tileset { tiles_per_row: 20, image: "EmeraldHillZone".to_string() };

    let act_file = std::fs::read_to_string("./assets/Act1Data.txt")
        .unwrap()
        .parse::<ActFile>()
        .unwrap();
    println!("Loaded act with {} entities and a width of {}", act_file.entities.len(), act_file.width);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Project Tails v2", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let creator = Box::leak(Box::new(canvas.texture_creator()));

    let holder = GraphicsHolder::load_all(Path::new("./assets/"), creator);
    println!("Loaded {} textures", holder.0.len());
    GRAPHICS_HOLDER.set(Mutex::new(holder)).ok().unwrap();

    for (idx, block) in blocks.iter().enumerate() {
        Block::add_graphics(format!("BLOCK{}", idx), block, &tileset, &canvas, creator);
    }

    let mut player = Player::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        player.update();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }


        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.clear();
        for r in 0..act_file.tiles.len() / act_file.width {
            for c in 0..act_file.width {
                let x = c * 128;
                let y = r * 128;

                if let Some((block_idx, _block_flags)) = act_file.tiles[r * act_file.width + c] {
                    canvas.copy(
                        &GraphicsHolder::get().get_pair(&format!("BLOCK{}", block_idx)).unwrap().1,
                        None,
                        Some(sdl2::rect::Rect::new(x as i32, y as i32, 128, 128)),
                    ).unwrap();
                }
            }
        }
        player.render(&mut canvas).unwrap();
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
