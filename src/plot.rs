use itertools::Itertools;

use crate::Command;

pub type Point = [f64; 3];
pub type Space = [usize; 3];

pub trait Plot {
    fn plot(&self, wait_time: f64, pre_trigger_wait: f64) -> String;
}

pub struct CustomPlot {
    space: Space,
    resolution: f64,
    origin_coordinate: Point,
    layers: Vec<ZLayer>,
}

impl CustomPlot {
    fn new(space: Space, resolution: f64, origin: Point) -> Self {
        Self {
            space,
            resolution,
            origin_coordinate: origin,
            layers: vec![],
        }
    }

    fn add_layer(&mut self, layer: ZLayer) -> Result<(), String> {
        match (
            self.space[0] != layer.x_space,
            self.space[1] != layer.y_space,
            self.space[2] == self.layers.len(),
        ) {
            (true, _, _) => Err("X space does not match.".to_string()),
            (_, true, _) => Err("Y space does not match.".to_string()),
            (_, _, true) => Err("Cannot add another layer, it would exceed Z-space.".to_string()),
            _ => {
                if self.layers.is_empty() && layer.origin.is_none() {
                    return Err("First layer must have an origin.".to_string());
                }
                if self.layers.len() == self.space[2] {
                    return Err("Cannot add another layer, it would exceed Z-space.".to_string());
                }
                self.layers.push(layer);
                Ok(())
            }
        }
    }

    fn origin_point(&self) -> Option<(usize, usize)> {
        if !self.layers.is_empty() {
            self.layers[0].origin
        } else {
            None
        }
    }

    pub fn from_string(string: &str, resolution: f64, origin: Point) -> Result<Self, String> {
        let layers = string
            .replace("\r\n", "\n")
            .split("\n\n")
            .map(ZLayer::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let x_space = layers[0].x_space;
        let y_space = layers[0].y_space;
        let z_space = layers.len();
        let mut plot = Self::new([x_space, y_space, z_space], resolution, origin);
        for layer in layers {
            plot.add_layer(layer)?;
        }
        Ok(plot)
    }

    pub fn from_file(path: &str, resolution: f64, origin: Point) -> Result<Self, String> {
        let file = std::fs::read_to_string(path).map_err(|_| "Failed to read file.")?;
        Self::from_string(&file, resolution, origin)
    }
}

impl Plot for CustomPlot {
    fn plot(&self, wait_time: f64, pre_trigger_wait: f64) -> String {
        let mut commands = vec![];
        let origin = self.origin_point().unwrap();
        for layer in self.layers.iter().enumerate() {
            let (z, layer) = layer;
            for point in layer.points.iter().enumerate() {
                let (i, point) = point;
                if matches!(point, PointType::ProbePoint) || matches!(point, PointType::Origin) {
                    let x = i % layer.x_space;
                    let y = i / layer.x_space + 1;
                    let x = self.origin_coordinate[0]
                        + (x as i32 - origin.0 as i32) as f64 * self.resolution;
                    let y = self.origin_coordinate[1]
                        + (y as i32 - origin.1 as i32) as f64 * self.resolution;
                    let z = self.origin_coordinate[2] + z as f64 * self.resolution;
                    commands.push(Command::Move([x, y, z]));
                    commands.push(Command::Wait(pre_trigger_wait));
                    commands.push(Command::SpindleOn);
                    commands.push(Command::Wait(wait_time));
                    commands.push(Command::SpindleOff);
                }
            }
        }

        create_gcode(commands)
    }
}

pub struct ZLayer {
    x_space: usize,
    y_space: usize,
    points: Vec<PointType>,
    origin: Option<(usize, usize)>,
}

impl ZLayer {
    pub fn construct_layer(
        x_space: usize,
        y_space: usize,
        origin: Option<(usize, usize)>,
        points: Vec<PointType>,
    ) -> Result<Self, String> {
        if x_space * y_space == points.len() {
            return Ok(Self {
                x_space,
                y_space,
                origin,
                points,
            });
        }
        Err("Point count does not match space.".to_string())
    }
}

impl TryFrom<&str> for ZLayer {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lines = value.split('\n').collect::<Vec<_>>();
        let x_space = lines[0].len();
        let y_space = lines.len();
        let points = lines
            .iter()
            .flat_map(|line| line.chars())
            .map(PointType::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let origin_index = points
            .iter()
            .enumerate()
            .filter(|(_, p)| matches!(p, PointType::Origin))
            .at_most_one()
            .map_err(|_| "More than one origin found.".to_string())?;
        let origin = origin_index.map(|(i, _)| (i % x_space, i / x_space + 1));
        Self::construct_layer(x_space, y_space, origin, points)
    }
}

pub enum PointType {
    Origin,
    None,
    ProbePoint,
}

impl TryFrom<char> for PointType {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'o' => Ok(Self::Origin),
            '-' => Ok(Self::None),
            'x' => Ok(Self::ProbePoint),
            _ => Err(format!("Invalid point type: {}", value)),
        }
    }
}

pub struct GridPlot {
    origin: Point,
    resolution: f64,
    space: [f64; 3],
}

impl GridPlot {
    pub fn new(origin: Point, space: [f64; 3], resolution: f64) -> Self {
        Self {
            origin,
            space,
            resolution,
        }
    }
}

impl Plot for GridPlot {
    fn plot(&self, wait_time: f64, pre_trigger_wait: f64) -> String {
        let [space_x, space_y, space_z] = self.space;
        let x_points = (space_x / self.resolution) as usize + 1;
        let y_points = (space_y / self.resolution) as usize + 1;
        let z_points = (space_z / self.resolution) as usize + 1;

        let [origin_x, origin_y, origin_z] = self.origin;
        let [x_start, y_start, z_start] = [
            origin_x - (x_points - 1) as f64 / 2. * self.resolution,
            origin_y - (y_points - 1) as f64 / 2. * self.resolution,
            origin_z,
        ];

        let mut commands = vec![];
        for z in 0..z_points {
            for y in 0..y_points {
                for x in 0..x_points {
                    let x = x_start + x as f64 * self.resolution;
                    let y = y_start + y as f64 * self.resolution;
                    let z = z_start + z as f64 * self.resolution;
                    commands.push(Command::Move([x, y, z]));
                    commands.push(Command::Wait(pre_trigger_wait));
                    commands.push(Command::SpindleOn);
                    commands.push(Command::Wait(wait_time));
                    commands.push(Command::SpindleOff);
                }
            }
        }

        create_gcode(commands)
    }
}

fn create_gcode(commands: Vec<Command>) -> String {
    let mut gcode = String::new();
    for command in commands {
        match command {
            Command::Move(point) => {
                gcode.push_str(&format!(
                    "G0 X{:.1} Y{:.1} Z{:.1}\n",
                    point[0], point[1], point[2]
                ));
            }
            Command::Wait(time) => {
                gcode.push_str(&format!("G4 P{}\n", time));
            }
            Command::SpindleOn => {
                gcode.push_str("M3\n");
            }
            Command::SpindleOff => {
                gcode.push_str("M5\n");
            }
        }
    }
    gcode
}
