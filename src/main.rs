use clap::Parser;
use serde::Deserialize;

fn main() {
    let args = Args::parse();
    let [origin_x, origin_y, origin_z]: [f64; 3] = args.origin.try_into().expect("Invalid origin.");
    let [space_x, space_y, space_z]: [f64; 3] = args.space.try_into().expect("Invalid space.");

    let config = ron::from_str::<Configuration>(
        &std::fs::read_to_string("config.ron").expect("Failed to read config file."),
    )
    .expect("Failed to parse config file.");

    let x_points = (space_x / args.resolution) as usize + 1;
    let y_points = (space_y / args.resolution) as usize + 1;
    let z_points = (space_z / args.resolution) as usize + 1;

    println!(
        "Grid size: {} x {} x {}. {} points.",
        x_points,
        y_points,
        z_points,
        x_points * y_points * z_points
    );

    let [x_start, y_start, z_start] = [
        origin_x - (x_points - 1) as f64 / 2. * args.resolution,
        origin_y - (y_points - 1) as f64 / 2. * args.resolution,
        origin_z,
    ];

    let mut commands = vec![];
    for z in 0..x_points {
        for y in 0..y_points {
            for x in 0..x_points {
                let x = x_start + x as f64 * args.resolution;
                let y = y_start + y as f64 * args.resolution;
                let z = z_start + z as f64 * args.resolution;
                commands.push(Command::Move([x, y, z]));
                commands.push(Command::Wait(config.pre_trigger_wait));
                commands.push(Command::SpindleOn);
                commands.push(Command::Wait(config.wait_time));
                commands.push(Command::SpindleOff);
            }
        }
    }

    let mut gcode = String::new();
    for command in commands {
        match command {
            Command::Move(point) => {
                gcode.push_str(&format!(
                    "G0 X{:.1} Y{:.1} Z{:.1}\n",
                    point[0], point[1], point[2]
                ));
            }
            Command::Wait(seconds) => {
                gcode.push_str(&format!("G4 P{}\n", seconds));
            }
            Command::SpindleOn => {
                gcode.push_str("M3\n");
            }
            Command::SpindleOff => {
                gcode.push_str("M5\n");
            }
        }
    }

    // Save the GCode to a file
    std::fs::write(args.file_name.clone(), gcode).expect("Failed to write GCode file.");
    println!("GCode saved to {}.", args.file_name)
}

#[derive(Parser, Debug)]
#[command(
    author = "Samuel James",
    long_about = "Creates a grid of points in 3D space. The grid is defined by the origin, the spacing between points in each dimension, and the resolution of the grid. After moving to each point the spindle will be activated and then a wait for a defined time in the configuration before deactivating the spindle."
)]
struct Args {
    #[arg(
        num_args = 3,
        required = true,
        value_delimiter = ',',
        short,
        long,
        help = "The volume of the grid given as X Y Z."
    )]
    space: Vec<f64>,
    #[arg(short, long, help = "The resolution of the grid (mm).")]
    resolution: f64,
    #[arg(
        num_args = 3,
        required = true,
        allow_hyphen_values = true,
        value_delimiter = ',',
        short,
        long,
        help = "The origin of the grid position at the center of the bottom of the volume as X Y Z."
    )]
    origin: Vec<f64>,
    #[arg(
        short = 'f',
        long,
        help = "The name of the file to save the GCode to. Defaults to protocol.nc.",
        default_value = "protocol.nc"
    )]
    file_name: String,
}

enum Command {
    Move([f64; 3]),
    Wait(f64),
    SpindleOn,
    SpindleOff,
}

#[derive(Deserialize, Debug)]
struct Configuration {
    wait_time: f64,
    pre_trigger_wait: f64,
}
