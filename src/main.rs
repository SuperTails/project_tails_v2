mod block;
mod player;
mod entity;
mod act;

use gamefox::renderable::Renderable;
use gamefox::camera::Camera;
use gamefox::physical::{Physical, Vector2};
use gamefox::input;
use gamefox::asset_mgr::GraphicsHolder;

use act::ActFile;
use player::Player;
use entity::Entity;
use block::{Block, Tileset};

use std::path::Path;
use std::time::Duration;

use sdl2::rect::Point;
use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::WindowContext;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use clap::{Arg, App};

struct SdlSystem {
    pub sdl_context: Sdl,
    pub video_subsystem: VideoSubsystem,
    pub canvas: WindowCanvas,
    pub creator: &'static TextureCreator<WindowContext>,
}

impl SdlSystem {
    pub fn new() -> SdlSystem {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Project Tails v2", 800, 600)
            .position_centered()
            .build()
            .unwrap();
    
        let canvas = window.into_canvas().build().unwrap();
        let creator = Box::leak(Box::new(canvas.texture_creator()));
    
        SdlSystem {
            sdl_context,
            video_subsystem,
            canvas,
            creator,
        }
    }
}

fn main() {
    let matches = App::new("Project Tails v2")
        .version("0.0")
        .author("Salix")
        .about("A simple game")
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Run in debug mode"))
        .get_matches();

    if matches.is_present("debug") {
        todo!("Debug mode");
    }    
    
    let mut sdl_system = SdlSystem::new();

    GraphicsHolder::load(Path::new("./assets/"), sdl_system.creator).ok().unwrap();

    let blocks = block::load_blocks(Path::new("./assets/EmeraldHillZone/"), "Block").unwrap();
    println!("Loaded {} blocks", blocks.len());

    let tileset = Tileset { tiles_per_row: 20, image: "EmeraldHillZone".to_string() };

    let mut act_file = std::fs::read_to_string("./assets/Act1Data.txt")
        .unwrap()
        .parse::<ActFile>()
        .unwrap();

    let act_file2 = format!("{}", act_file).parse::<ActFile>().unwrap();
    assert_eq!(act_file, act_file2);

    act_file.entities = vec![
        Entity::new(Vector2 { x: 100.0, y: 300.0, }, "BEEBADNIK".to_string(), Vec::new()),
        Entity::new(Vector2 { x: 100.0, y: 400.0, }, "BEEBADNIK".to_string(), Vec::new()),
    ];
    
    println!("Loaded act with {} entities and a width of {}", act_file.entities.len(), act_file.width);

    let mut camera = Camera { position: Vector2 { x: 0.0, y: 0.0 }};

    Block::add_graphics_multi(
        blocks.iter().enumerate().map(|(i, block)| (format!("BLOCK{}", i), block)),
        &tileset,
        sdl_system.creator,
    );

    let collision_map = block::parse_collision_map("CollisionTiles");

    let getter = block::TerrainGetter::new(
        act_file.width,
        &act_file.tiles,
        &blocks,
        &collision_map,
    );

    let mut player = Player::new();

    let mut event_pump = sdl_system.sdl_context.event_pump().unwrap();
    'running: loop {
        for entity in act_file.entities.iter_mut() {
            entity.update();
        }
        player.update(&getter);
        camera.position = player.get_position();
        camera.position.x -= 200.0;
        camera.position.y -= 300.0;

        let events = event_pump.poll_iter().collect::<Vec<_>>();
        input::update(&events);
        for event in events.clone() {
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

        sdl_system.canvas.set_draw_color(Color::RGB(20, 20, 20));
        sdl_system.canvas.clear();
        for r in 0..act_file.tiles.len() / act_file.width {
            for c in 0..act_file.width {
                let x = c * 128;
                let y = r * 128;

                let x = x as f64 - camera.position.x;
                let y = y as f64 - camera.position.y;

                sdl_system.canvas.set_draw_color(Color::RGB(255, 255, 255));
                sdl_system.canvas.draw_line(
                    Point::new(x as i32, y as i32),
                    Point::new(x as i32, y as i32 + 600),
                ).unwrap();
                sdl_system.canvas.draw_line(
                    Point::new(x as i32, y as i32),
                    Point::new(x as i32 + 600, y as i32),
                ).unwrap();

                if let Some((block_idx, block_flags)) = act_file.tiles[r * act_file.width + c] {
                    sdl_system.canvas.copy_ex(
                        &GraphicsHolder::get().get_pair(&format!("BLOCK{}", block_idx)).unwrap().1,
                        None,
                        sdl2::rect::Rect::new(x as i32, y as i32, 128, 128),
                        0.0,
                        None,
                        // TODO: Support more flags
                        block_flags != 0,
                        false,
                    ).unwrap();
                }
            }
        }
        for entity in act_file.entities.iter() {
            entity.render(&mut sdl_system.canvas, &camera).unwrap();
        }
        player.render(&mut sdl_system.canvas, &camera).unwrap();
        sdl_system.canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
