mod rasterizer;

use std::path::PathBuf;
use std::error::Error;
use glam::*;
use tobj;
use std::fs::OpenOptions;
use clap::{Parser, Subcommand};
use rasterizer::*;

pub fn to_meshes(models: Vec<tobj::Model>, materials: Vec<tobj::Material>) -> Vec<SimpleMesh> {
    let mut meshes: Vec<SimpleMesh> = vec![];
    for model in models {
        meshes.push(model.mesh.to_simple_mesh_with_materials(&materials));
    }
    meshes
}

#[derive(Subcommand, Debug)]
enum Mode {
    Image {
        #[arg(short, long, required = false)]
        width: usize,

        #[arg(short, long, required = false)]
        height: usize,
    },
    Turntable {
        speed: f32,
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, required = false)]
    file_name: PathBuf,

    #[command(subcommand)]
    mode: Option<Mode>,
}

fn main() -> Result<(), Box<dyn Error>>  {
    let args = Args::parse();

    // TODO: Image + Turntable
    let mut context = Rasterizer::new(40, 40);

    let error = |s: &str, e: &str| -> Result<Vec<SimpleMesh>, Box<dyn Error>> {
        Err(format!("filename: [{}] couldn't load, {}. {}", args.file_name.display(), s, e).into())
    };

    let meshes = match args.file_name.extension() {
        None => error("couldn't determine filename extension", ""),
        Some(ext) => match ext.to_str() {
            None => error("couldn't parse filename extension", ""),
            Some(extstr) => match &*extstr.to_lowercase() {
                "obj" => match tobj::load_obj(&args.file_name, &tobj::GPU_LOAD_OPTIONS) {
                    Err(e) => error("tobj couldnt load/parse OBJ", &e.to_string()),
                    Ok(present) => Ok(to_meshes(
                        present.0,
                        present.1.expect("Expected to have materials."),
                    )),
                },
                "stl" => match OpenOptions::new().read(true).open(&args.file_name) {
                    Err(e) => error("STL load failed", &e.to_string()),
                    Ok(mut file) => match stl_io::read_stl(&mut file) {
                        Err(e) => error("stl_io couldnt parse STL", &e.to_string()),
                        Ok(stlio_mesh) => Ok(vec![stlio_mesh.to_simple_mesh()]),
                    },
                },
                _ => error("unknown filename extension", ""),
            },
        },
    }?;

    context.update(&meshes)?;
    let transform = Mat4::IDENTITY;
    context.draw_all(transform, meshes)?;

    context.flush()?;

    Ok(())
}
