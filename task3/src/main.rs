use std::fs::File;
use std::path::PathBuf;

use anyhow;
use clap::Parser;
use walkdir::WalkDir;
use threadpool::ThreadPool;
use crossbeam_channel::{unbounded, Sender, Receiver};


// Task3. Folder indexing with threads: variants 1) HashMap and 2) BTreeMap
//use std::collections::HashMap;
//type Map = HashMap<String, HashMap<std::path::PathBuf, Vec<usize>>>;

use std::collections::BTreeMap;
type Map = BTreeMap<String, BTreeMap<std::path::PathBuf, Vec<usize>>>;

static SEPARATORS: &[char; 4] = &[' ', ',', ';', '.'];


#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "./result.txt")]
    result: String,
    #[arg(short, long, default_value = ".")]
    folder: String,
    #[arg(short, long, default_value_t = 0)]
    max_depth: usize,
    #[arg(short, long, default_value_t = 0)]
    workers: usize
}

fn main() -> anyhow::Result<()> {
    println!("The task3:");
    /*
        println!("  1) Get path to a folder and max depth, and number of thread workers via application arguments");
        println!("  2) Iterate over the folders and process files in thread workers");
        println!("  3) A worker reads a file content and use indexing function to get word positions in text");
        println!("  4) A worker sends resulting hash map to main");
        println!("  5) The main application merge resulting hash-maps into one"
        println!("  6) The main application saves a final hash-map in a file");
    */

    let args                 = Args::parse();
    let start_path:  PathBuf = args.folder.into();
    let result_path: PathBuf = args.result.into();
    let max_depth:     usize =
        if args.max_depth == 0 {
            usize::MAX
        } else {
            args.max_depth
        };
    let workers:     usize =
        if args.workers == 0 {
            match std::thread::available_parallelism() {
                Ok(e) => { e.get() }
                Err(_err) => 4
            }
        } else {
            args.workers
        };

    let map: Map = index_folder_threaded(&start_path, max_depth, workers);

    let result = save_index_file(&result_path, map);
    match result {
        Ok(_res) => {
            println!("Files in folder {:?} are indexed as {:?}", start_path, result_path);
        }
        Err(err) => {
            eprintln!("Error: {}...", err);
        }
    }

    println!("Good bye!");
    Ok(())
}

fn index_file(file_path: &PathBuf) -> Map {
    let mut fmap = Map::new();
    let path = file_path.as_path();

    let file_content = std::fs::read_to_string(path);
    match file_content {
        Ok(file_cont) => {
            let text_addr: usize = file_cont.as_ptr().addr();

            for word in file_cont.split(SEPARATORS) {
                let word_address = word.as_ptr().addr();
                let offset = word_address - text_addr;

                fmap.entry(String::from(word)).or_default()
                    .entry(path.to_path_buf()).or_default().push(offset);
            }
        }
        Err(_err) => {
            eprintln!("Error: {}...", _err);
        }
    }
    return fmap;
}

fn thread_worker(task_rx: Receiver<PathBuf>, result_tx: Sender<Map>) {
    while let Ok(path) = task_rx.recv() {
        let map = index_file(&path);
        //println!("THR {:?} : file {:?}: map {:?}", std::thread::current().id(), path, map);
        result_tx.send(map).unwrap();
    }
}

fn index_folder_threaded(start_path: &PathBuf, depth: usize, workers: usize) -> Map {
    let mut bigmap = Map::new();

    if !start_path.exists() {
        eprintln!("Error: '{}' does not exist", start_path.display());
        return bigmap;
    }

    let (task_tx, task_rx) = unbounded();
    let (result_tx, result_rx) = unbounded();

    let the_pool = ThreadPool::new(workers);
    for _ in 0..workers {
        let rx = task_rx.clone();
        let tx = result_tx.clone();
        the_pool.execute(move || { thread_worker(rx, tx); });
    }

    for entry_res in WalkDir::new(start_path).max_depth(depth) {
        match entry_res {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    task_tx.send(entry.into_path()).unwrap();
                }
            }
            Err(err) => {
                eprintln!("Error: {}...", err);
                continue;
            }
        }
    }

    drop(task_tx);
    drop(result_tx);

    while let Ok(fmap) = result_rx.recv() {
        merge_maps(&mut bigmap, fmap);
    }

    the_pool.join();

    //println!("Final map {:?}", bigmap);
    return bigmap;
}

fn merge_maps(bigmap: &mut Map, fmap: Map)  {
    for (word, map_path) in fmap {//.iter() -->  expected `String`, found `&String`
        let bigmap_path = bigmap.entry(word).or_default();
        for (path, mut vector) in map_path {
            bigmap_path.entry(path).or_default().append(&mut vector);
        }
    }
}

fn save_index_file(file_name: &PathBuf, map: Map)  -> anyhow::Result<()> {
    let file = File::create(&file_name)?;
    serde_json::to_writer_pretty(file, &map)?;
    Ok(())
}


#[test]
fn test_index_single_file() -> anyhow::Result<()> {
    let Ok(result) = std::fs::create_dir_all("./test/") else {
        return Err(anyhow::anyhow!("Error: './test/'"))
    };
    let Ok(result) = std::fs::copy("./test_data/lorem1", "./test/lorem1") else {
        return Err(anyhow::anyhow!("Error: copy file './test_data/lorem1'"))
    };

    let map = index_folder_threaded(&PathBuf::from("./test/"), 4, 4);
    println!("MAP {:?}", map);
    assert_ne!(map.is_empty(), true);
    assert_eq!(map["Lorem"].len(), 1);
    assert_eq!(map["dolor"].len(), 1);
    assert_eq!(map["ipsum"].len(), 1);
    assert_eq!(map.contains_key("dolore"), false);

    let Ok(result) = std::fs::remove_dir_all("./test") else {
        return Err(anyhow::anyhow!("Error: removing folder './test'"))
    };
    Ok(())
}

#[test]
fn test_index_many_files_with_folder_depth() {
    println!("The task3 test with depth");
    let map = index_folder_threaded(&PathBuf::from("./test_data/"), 1, 4);

    assert_eq!(map["Lorem"].len(), 2);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["ipsum"].len(), 2);
    assert_eq!(map.contains_key("dolore"), false);
}

#[test]
fn test_index_many_files_no_folder_depth() {
    let map = index_folder_threaded(&PathBuf::from("./test_data/"), 4, 4);

    assert_ne!(map.len(), 0);
    assert_eq!(map["Lorem"].len(), 7);
    assert_eq!(map["dolor"].len(), 7);
    assert_eq!(map["ipsum"].len(), 7);
    assert_eq!(map["dolore"].len(), 2);
}
