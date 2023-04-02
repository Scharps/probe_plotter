use clap::{Parser, Subcommand};
use plot::CustomPlot;
use serde::Deserialize;

use crate::plot::{GridPlot, Plot};

pub mod plot;
mod tests;

fn main() {
    let app = App::parse();
    let config: Configuration = ron::from_str(
        std::fs::read_to_string("config.ron")
            .expect("Error reading config.ron file.")
            .as_str(),
    )
    .expect("Error parsing config.ron file.");
    let (wait_time, pre_trigger_wait) = (config.wait_time, config.pre_trigger_wait);
    match app.command {
        SubCommand::Grid {
            space,
            resolution,
            origin,
            file_name,
        } => {
            let grid = GridPlot::new(
                origin
                    .try_into()
                    .expect("Incorrect number of arguments for origin."),
                space
                    .try_into()
                    .expect("Incorrect number of arguments for space."),
                resolution,
            );
            let plot = grid.plot(wait_time, pre_trigger_wait);
            std::fs::write(file_name, plot).expect("Error writing to file.");
        }
        SubCommand::Custom {
            origin,
            resolution,
            input,
            file_name,
        } => {
            let custom = CustomPlot::from_file(
                input.as_str(),
                resolution,
                origin
                    .try_into()
                    .expect("Incorrect number of arguments for origin."),
            );
            match custom {
                Ok(plot) => {
                    let gcode = plot.plot(wait_time, pre_trigger_wait);
                    std::fs::write(file_name, gcode).expect("Error writing to file.");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "grid", author = "Samuel James")]
struct App {
    #[clap(subcommand)]
    command: SubCommand,
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

#[derive(Subcommand)]
enum SubCommand {
    #[clap()]
    Grid {
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
            help = "The name of the file to save the GCode to.",
            default_value = "protocol.nc"
        )]
        file_name: String,
    },
    #[clap()]
    Custom {
        #[arg(
            num_args = 3,
            required = true,
            value_delimiter = ',',
            short,
            long,
            help = "The origin coordinate as X Y Z."
        )]
        origin: Vec<f64>,
        #[arg(short, long, help = "The resolution of the plot (mm).")]
        resolution: f64,
        #[arg(short, long, help = "The path of the input file.")]
        input: String,
        #[arg(
            short = 'f',
            long,
            help = "The name of the file to save the GCode to.",
            default_value = "protocol.nc"
        )]
        file_name: String,
    },
}
