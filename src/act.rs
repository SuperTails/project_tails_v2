use std::str::FromStr;

pub struct Entity {
    x: f64,
    y: f64,
    kind: String,
    flags: Vec<String>,
}

impl FromStr for Entity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split(' ');
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

        Ok(Entity { x, y, kind, flags })
    }
}

pub struct ActFile {
    pub version: String,
    pub name: String,
    pub entities: Vec<Entity>,
    pub width: usize,
    pub tiles: Vec<Option<(usize, u32)>>,
}

impl FromStr for ActFile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let version = lines
            .next()
            .ok_or_else(|| "Expected version".to_string())?
            .to_string();

        let name = lines
            .next()
            .ok_or_else(|| "Expected name".to_string())?
            .to_string();

        let entities = lines
            .by_ref()
            .take_while(|l| l != &"E")
            .map(<Entity as FromStr>::from_str)
            .collect::<Result<Vec<Entity>, <Entity as FromStr>::Err>>()?;

        // TODO: Maybe use the act type?
        assert_eq!(lines.next(), Some("NORMAL"));
        // TODO: Maybe use the filename for the tileset?
        assert_eq!(lines.next(), Some("EmeraldHillZone.png"));
        // TODO: Maybe use the path for the blocks?
        assert_eq!(lines.next(), Some("EmeraldHillZone/Block"));
        // TODO: Maybe use the path for the background?
        assert_eq!(lines.next(), Some("EmeraldHillZone/Background/"));

        let tile_count = lines
            .next()
            .ok_or_else(|| "Expected tile count".to_string())?
            .parse::<usize>()
            .map_err(|e| format!("Parsing error: {}", e))?;

        let (width, height) = {
            let mut numbers = lines
                .next()
                .ok_or_else(|| "Expected width and height".to_string())?
                .split(' ');

            let width = numbers
                .next()
                .ok_or_else(|| "Expected width".to_string())?
                .parse::<usize>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            let height = numbers
                .next()
                .ok_or_else(|| "Expected height".to_string())?
                .parse::<usize>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            if numbers.next() != None {
                return Err("Trailing data after width and height".to_string());
            }

            (width, height)
        };

        let mut tiles: Vec<Option<(usize, u32)>> = Vec::new();
        tiles.resize(width * height, None);
        for (_idx, line) in (0..tile_count).zip(lines.by_ref()) {
            let mut numbers = line.split(' ');

            let x = numbers
                .next()
                .ok_or_else(|| "Expected x".to_string())?
                .parse::<usize>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            let y = numbers
                .next()
                .ok_or_else(|| "Expected y".to_string())?
                .parse::<usize>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            let tile_idx = numbers
                .next()
                .ok_or_else(|| "Expected index".to_string())?
                .parse::<usize>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            let tile_flags = numbers
                .next()
                .unwrap_or("0")
                .parse::<u32>()
                .map_err(|e| format!("Parsing error: {}", e))?;

            if x % 128 != 0 || y % 128 != 0 {
                return Err(format!("Invalid tile position {}, {}", x, y));
            }

            let x = x / 128;
            let y = y / 128;

            if x >= width || y >= height {
                return Err(format!("Out of range tile position {}, {}", x, y));
            }

            tiles[x + y * width] = Some((tile_idx, tile_flags));

            if numbers.next() != None {
                return Err("Trailing data after end of tile".to_string());
            }
        }

        if lines.next() != None {
            return Err("Trailing data after end of tile list".to_string());
        }

        Ok(ActFile {
            version,
            name,
            entities,
            width,
            tiles,
        })
    }
}
