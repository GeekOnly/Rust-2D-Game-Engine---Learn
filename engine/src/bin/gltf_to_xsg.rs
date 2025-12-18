use std::env;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.glb> <output.xsg>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    println!("Converting {} to {}", input_path.display(), output_path.display());

    // Use the XSG importer to convert GLTF to XSG
    let xsg = engine::assets::xsg_importer::XsgImporter::import_from_gltf(input_path)?;
    engine::assets::xsg_importer::XsgImporter::save_to_file(&xsg, output_path)?;

    println!("Conversion completed successfully!");
    Ok(())
}