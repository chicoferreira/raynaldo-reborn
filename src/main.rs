use std::path::PathBuf;

use crate::raytracer::environment::Environment;
use crate::raytracer::loader::CameraSettings;
use crate::raytracer::world::World;
use clap::Parser;

mod app;
mod raytracer;

#[derive(Parser, Debug)]
#[command(name = "raynaldo-reborn")]
#[command(about = "A ray tracer with multiple backend options")]
struct Args {
    /// Path to the world configuration file
    world: PathBuf,
    /// Tracer type to use for ray tracing
    #[arg(short, long, value_enum, default_value_t = TracerType::Embree)]
    tracer: TracerType,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum TracerType {
    /// Use the naive ray tracer implementation
    Naive,
    /// Use the Embree ray tracer implementation
    Embree,
}

fn main() {
    let args = Args::parse();

    #[derive(serde::Deserialize)]
    struct WorldConfig {
        camera: CameraSettings,
        #[serde(default)]
        environment: Environment,
        #[serde(flatten)]
        world: World,
    }

    let world = std::fs::read_to_string(args.world).unwrap();
    let world: WorldConfig = toml::from_str(&world).unwrap();

    app::run(world.world, world.environment, world.camera, args.tracer);
}
