use std::env;
use std::io::Read;
use std::io::BufReader;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::process;
use std::path::PathBuf;
use std::io::prelude::*;
use serde_json::Value;
use byteorder::{ReadBytesExt, LittleEndian};


fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut path = PathBuf::new();

    // Check if we have got an argument
    match args.len() {
        1 => panic!("You must specify a file to unpack"),
        2 => path.push("."),
        3 => path.push(&args[2]),
        _ => {
            println!("asar_unpack FILE [DEST]");
            process::exit(1);
        }
    }

    let asar_file = &args[1];
    println!("Going to use {:?}", asar_file);


    let f = File::open(asar_file)?;
    let mut reader = BufReader::new(f);

    // Read info about header
    let mut pre_header = [0u8; 16];
    reader.read_exact(&mut pre_header)?;

    let mut pre_header_cursor = Cursor::new(pre_header);

    // These bytes are unknown to me
    let _ = pre_header_cursor.read_u32::<LittleEndian>().unwrap();
    let _ = pre_header_cursor.read_u32::<LittleEndian>().unwrap();

    // Read usefull info
    let _start_of_files = pre_header_cursor.read_u32::<LittleEndian>().unwrap();
    let header_length = pre_header_cursor.read_u32::<LittleEndian>().unwrap() as usize;

    let real_header_length = pad_fucking_pickle_string(&header_length);

    //println!("start_of_files: {:?}", start_of_files);
    //println!("header_length: {:?} Real: {:?}", header_length, real_header_length);

    // Vector for raw file wihtout header-info and read while file into it
    let mut raw_file = Vec::new();
    reader.read_to_end(&mut raw_file)?;

    // Extract json from raw header
    let meta_raw = &raw_file[0..header_length];
    // Parse the slice as json
    let meta_json: serde_json::Map<String, Value> = serde_json::from_slice(meta_raw).unwrap();

    process_level(meta_json, path, &real_header_length, &raw_file);

    Ok(())
}

fn process_level(
        input_dir: serde_json::Map<String, Value>, 
        level_path: std::path::PathBuf,
        header_len: &usize,
        raw_file: &Vec<u8>) {

    let unpacked_level = unpack_dir(input_dir);

    // Iterate over each item of unpacked level
    for (node_name, value) in unpacked_level.iter() {
        let node_path = level_path.join(node_name);

        let unobed_value = unobjectify(value);


        // Check what time this item is (folder, file ...)
        // If it contains a key named "files" it is a folder
        // also we need to go a level deeper
        if unobed_value.contains_key("files") {
            println!("{}", node_path.display());
            // Create dir 
            match fs::create_dir_all(&node_path){
                Ok(_) => (),
                Err(x) => {
                    println!("Create folder FAILED: {:?}", x)
                }
            }

            // Go level deeper
            process_level(unobed_value, node_path, header_len, raw_file);
        } else if unobed_value.contains_key("size") {
            // Files can be unpacked (-> not included in archive)
            if unobed_value.contains_key("unpacked") {
                println!("{} skipped as marked as unpacked :(", node_path.display());
            } else {
                println!("{}", node_path.display());
                process_file(node_path, unobed_value, header_len, raw_file);
            }
        } else {
            println!("{} is unknown", node_path.display());
        }
    }
}

fn process_file(
    file_path: PathBuf, 
    meta: serde_json::Map<std::string::String, 
    serde_json::Value>, 
    header_len: &usize,
    raw_file: &Vec<u8>) {

    // Offset is specified as a string
    let offset = meta.get("offset").unwrap().as_str().unwrap().parse::<usize>().unwrap();

    // Size is a "number"
    let size = meta.get("size").unwrap().as_u64().unwrap() as usize;

    // TODO: Get hash and check it

    let start_of_file = header_len + offset;
    let file_content = &raw_file[start_of_file..start_of_file+size];

    //println!("Offset: {} Size: {}\nFile_content {:?}", offset, size, file_content);

    let mut buffer = File::create(&file_path).unwrap();
    match buffer.write_all(file_content) {
        Ok(_) => {},
        Err(x) => {
            println!("Can't write file {}: {}", &file_path.display(), x);
        }
    }
}

fn unpack_dir(input: serde_json::Map<String, Value>) -> serde_json::Map<std::string::String, serde_json::Value> {
    let level0 = input.get("files").unwrap_or_else(|| {
        println!("Can't get files from directory");
        process::exit(1)
    });

    return unobjectify(level0);
}

// Remove Object()...
fn unobjectify(input: &Value) -> serde_json::Map<std::string::String, serde_json::Value> {
    match input {
        Value::Object(x) => x.clone(),
        _ => {
            println!("Unkown stuff for key 'files'");
            process::exit(1)
        }
    }
}

// https://github.com/electron/node-chromium-pickle-js/blob/master/lib/pickle.js#L195
fn pad_fucking_pickle_string(input: &usize) -> usize{
    let remaining = input % 4;

    let padding = match remaining {
        0 => 0,
        _ => 4 - remaining
    };

    return input + padding;
}