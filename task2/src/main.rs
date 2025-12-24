use std::fs::File;
use std::path::PathBuf;

use anyhow;
use clap::Parser;
use walkdir::WalkDir;

// Task2. Folder indexing: variants 1) HashMap and 2) BTreeMap
// -----------------------------------------------------------
//use std::collections::HashMap;
//type Map = HashMap<String, HashMap<std::path::PathBuf, Vec<usize>>>;

use std::collections::BTreeMap;
type Map = BTreeMap<String, BTreeMap<std::path::PathBuf, Vec<usize>>>;


static SEPARATORS: &[char; 4] = &[' ', ',', ';', '.'];
fn index_folder(start_path: &PathBuf, depth: usize) -> Map {
    let mut bigmap = Map::new();

    if !start_path.exists() {
        eprintln!("Error: '{}' does not exist", start_path.display());
        return bigmap;
    }

    for entry_res in WalkDir::new(start_path).max_depth(depth) {
        match entry_res {
            Ok(entry) => {
                let path = entry.path();
                let mut text_addr: usize = 0;

                if entry.file_type().is_file() {
                    let file_content = std::fs::read_to_string(path);
                    match file_content {
                        Ok(file_cont) => {
                            text_addr = file_cont.as_ptr().addr();

                            for word in file_cont.split(SEPARATORS) {
                                let word_address = word.as_ptr().addr();
                                let offset = word_address - text_addr;

                                bigmap.entry(String::from(word)).or_default()
                                    .entry(path.to_path_buf()).or_default().push(offset);
                            }
                        }
                        Err(_err) => {
                            eprintln!("Error: {}...", _err);
                            continue;
                        }
                    }
                }
            }
            Err(_err) => {
                eprintln!("Error: {}...", _err);
                continue;
            }
        }
    }
    return bigmap;
}

fn save_index_file(file_name: &PathBuf, map: Map)  -> anyhow::Result<()> {
    let file = File::create(&file_name)?;
    serde_json::to_writer_pretty(file, &map)?;
    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "./result.txt")]
    result: String,
    #[arg(short, long, default_value = ".")]
    folder: String,
    #[arg(short, long, default_value_t = 0)]
    max_depth: usize
}

fn main() -> anyhow::Result<()> {
    println!("The task2:");
    /*
        println!("  1) Get path to a folder and max depth via application arguments");
        println!("  2) Iterate over the folders and open each file");
        println!("  3) Read a file content and use index_string function to get word positions in text");
        println!("  4) Save results in a resulting hash-map under key of file name");
        println!("  5) Save hash-map in a file");
    */

    let args                 = Args::parse();
    let start_path:  PathBuf = args.folder.clone().into();
    let result_path: PathBuf = args.result.clone().into();
    let max_depth:     usize =
        if args.max_depth == 0 {
            usize::MAX
        } else {
            args.max_depth
        };

    let map: Map = index_folder(&start_path, max_depth);
    let result = save_index_file(&result_path, map);
    match result {
        Ok(res) => {
            println!("Files in folder {:?} are indexed as {:?}", start_path, result_path);
        }
        Err(_err) => {
            eprintln!("Error: {}...", _err);
        }
    }

    println!("Good bye!");
    Ok(())
}

/*
#[test]
fn test_index_single_file() {
    match std::fs::create_dir("./test/") {
        Err(_err) => { eprintln!("Error: {}...", _err);
                         assert!(false, "Error: create folder") }
         Ok(_res) => {}
    }
    match std::fs::copy("./test_data/lorem1", "./test/file") {
        Err(_err) => { eprintln!("Error: {}...", _err);
                         assert!(false, "Error: copy file") }
         Ok(_res) => {}
    }

    let map = index_folder(&PathBuf::from("./test/"), 0);
    assert_ne!(map.len(), 0);
    assert_eq!(map["Lorem"].len(), 2);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["ipsum"].len(), 2);
    assert_eq!(map["dolore"].len(), 0);

    match std::fs::remove_file("./test/lorem1") {
        Err(_err) => { eprintln!("Error: {}...", _err);
                         assert!(false, "Error: remove file") }
         Ok(_res) => {}
    }
    match std::fs::remove_dir("./test/") {
        Err(_err) => { eprintln!("Error: {}...", _err);
                         assert!(false, "Error: remove folder") }
         Ok(_res) => {}
    }
}*/

#[test]
fn test_index_many_files_no_folder_depth() {
    let map = index_folder(&PathBuf::from("./test_data/"), 1);
    println!("map {:#?}", map);

    assert_eq!(map["Lorem"].len(), 2);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["ipsum"].len(), 2);
    assert_eq!(map.contains_key("dolore"), false);
}

#[test]
fn test_index_many_files_with_folder_depth() {
    let map = index_folder(&PathBuf::from("./test_data/"), 4);
    println!("map {:#?}", map);

    assert_eq!(map["Lorem"].len(), 7);
    assert_eq!(map["dolor"].len(), 7);
    assert_eq!(map["ipsum"].len(), 7);
    assert_eq!(map["dolore"].len(), 2);
}
