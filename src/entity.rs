use gamefox::physical::Vector2;
use gamefox::renderable::Renderable;
use gamefox::camera::Camera;
use gamefox::animation::Animation;
use std::str::FromStr;
use sdl2::render::{RenderTarget, Canvas};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::RwLock;
use std::fmt;

lazy_static! {
    static ref ENTITY_DATA: RwLock<HashMap<String, Vec<Animation>>> = {
        let data = std::fs::read_to_string("./assets/EntityData.txt").unwrap();
        let mut result = HashMap::new();

        for line in data.lines() {
            let mut parts = line.split(' ').filter(|s| !s.is_empty());
            assert_eq!(parts.next(), Some("OBJ"));
            let kind = parts.next().unwrap().to_string();
            let mut parts = parts.skip(7);

            let mut anims = Vec::new();
            let mut n = parts.next().unwrap();
            while n != "EA" {
                // Trim off the .png extention
                let image = n[..n.len() - 4].to_string();

                let duration = parts.next().unwrap().parse::<i32>().unwrap();
                let duration = if duration == -1 {
                    Duration::new(0, 0)
                } else {
                    Duration::from_millis(duration as u64)
                };

                let frames = parts.next().unwrap().parse::<usize>().unwrap();

                anims.push(Animation::new(image, frames, duration));

                n = parts.next().unwrap();
            }

            result.insert(kind, anims);
        }

        RwLock::new(result)
    };
}

#[derive(PartialEq, Debug)]
pub struct Entity {
    position: Vector2,
    kind: String,
    flags: Vec<String>,
}

impl Entity {
    pub fn update(&mut self) {
        let mut data = ENTITY_DATA.write().unwrap();
        let anims = data.get_mut(&self.kind).unwrap();
        if let Some(anim) = anims.get_mut(0) {
            anim.update();
        }
    }

    pub fn new(position: Vector2, kind: String, flags: Vec<String>) -> Entity {
        if ENTITY_DATA.read().unwrap().get(&kind).is_none() {
            panic!("Invalid entity kind {:?}", kind);
        }

        Entity {
            position,
            kind,
            flags,
        }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} ", self.position.x, self.position.y, self.kind)?;
        for flag in self.flags.iter() {
            write!(f, "{} ", flag)?;
        }
        Ok(())
    }
}

impl FromStr for Entity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split(' ').filter(|s| !s.is_empty());
        let x = s
            .next()
            .ok_or_else(|| "Expected x".to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())?;

        let y = s
            .next()
            .ok_or_else(|| "Expected y".to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())?;

        let kind = s
            .next()
            .ok_or_else(|| "Expected name".to_string())?
            .to_string();

        let flags = s.map(String::from).collect::<Vec<String>>();

        Ok(Entity { position: Vector2 { x, y }, kind, flags })
    }
}

impl Renderable for Entity {
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, camera: &Camera) -> Result<(), String> {
        let data = ENTITY_DATA.read().unwrap();
        let anims = data.get(&self.kind).unwrap();

        if let Some(anim) = anims.get(0) {
            (self.position, anim).render(canvas, camera)
        } else {
            Ok(())
        }
    }
}