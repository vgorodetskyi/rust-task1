use std::fs::File;
use std::collections::HashMap;

use anyhow;
use clap::Parser;
use walkdir::WalkDir;
use std::fmt::Write as FmtWrite;
use std::io::BufWriter;

type Map = HashMap<String, HashMap<std::path::PathBuf, Vec<usize>>>;

static SEPARATORS: &[char; 4] = &[' ', ',', ';', '.'];
fn index_folder(start_path: &String, depth: usize) -> Map {
    let mut bigmap: Map = HashMap::new();

    for entry_res in WalkDir::new(start_path).max_depth(depth) {
        match entry_res {
            Ok(entry) => {
                let path = entry.path();
                let mut text_addr: usize = 0;
                
                if entry.file_type().is_file() {
                    let file_conent = std::fs::read_to_string(path);
                    match file_conent {
                        Ok(file_cont) => {
                            text_addr = file_cont.as_ptr().addr();

                            for word in file_cont.split(SEPARATORS) {
                                let word_address = word.as_ptr().addr();
                                let offset = word_address - text_addr;

                                bigmap
                                .entry(String::from(word)).or_default()
                                .entry(path.to_path_buf()).or_default().push(offset);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {}...", err);
                            continue;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {}...", err);
                continue;
            }
        }
    }
    return bigmap;
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = ".")]
    folder: String,
    #[arg(short, long)] //, default_value = 0)]
    max_depth: Option<usize>,
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
    //arguments -- folder path, max depth
    let args = Args::parse();
    let depth: usize;
    let start_path = args.folder.clone();

    match args.max_depth {
        Some(max_depth) => depth = max_depth,
        None => depth = 0,
    }

    let  map: Map = index_folder(&start_path, depth);
    let mut file = File::create("result.txt")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &map)?;

    println!("Folder {start_path} is indexed");
    println!("Good bye!");
    Ok(())
}

#[test]
fn test_long() {
    let map = index_string(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    );
    assert_eq!(map["Lorem"].len(), 1);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["dolore"].len(), 2);
}

#[test]
fn test_long_from_file() {
    let file_content = std::fs::read_to_string("test_data/lorem1");
    let data = file_content.unwrap();
    let map = index_string(data.as_str());
    assert_eq!(map["Lorem"].len(), 1);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["dolore"].len(), 2);
}

#[test]
fn test_long_from_file2() {
    let file_content1 = std::fs::read_to_string("test_data/lorem1");
    let data1 = file_content1.unwrap();
    let map1 = index_string(data1.as_str());
    let file_content2 = std::fs::read_to_string("test_data/lorem2");
    let data2 = file_content2.unwrap();
    let map2 = index_string(data2.as_str());
    assert_eq!(map1.len(), map2.len());
    assert_eq!(map1.contains_key("dolor"), true);
    assert_eq!(map2.contains_key("dolor"), true);
    assert_eq!(map1["dolor"].len(), 2);
    assert_eq!(map2["dolor"].len(), 2);
}
