use std::{env, fs::File, io::{self, Read}};

fn load_rom(path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn handle_rom() -> Vec<u8> {
    let mut rom_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match &arg[..] {
            "-r" | "--rom_path" => {
                if let Some(arg_rom_path) = args.next() {
                    rom_path = Some(arg_rom_path)
                }
            },
            _ => println!("Unknown argument {}. Skipping...", arg)
        }
    }

    let rom = match rom_path {
        Some(value) => value,
        None => panic!("required argument '--rom_path <rom_path>' not found.")
    };

    match load_rom(rom.as_str()) {
        Ok(value) => value,
        Err(err) => panic!("Couldn't load rom: {}", err)
    }
}