use std::io;
use std::collections::HashMap;

fn index_string(data: &str) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();

    let mut parts = data
        .split(|c| c == ';' || c == '.' || c == ',' || c == ' ')
        .filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    parts.sort();
    parts.dedup();

    for part in parts {
        if ! map.contains_key(part) {
            //println!("  NEW {:?}", part);
            let mut values : Vec<usize> = Vec::new();

            for (index, matched_str) in data.match_indices(part) {
                //println!("   Found '{}' at byte index: {}", matched_str, index);
                values.push(index);
            }
            map.insert(part, values);
        }
        else {
            println!("  Update {part} {:?}", map[part]);
        }
    }

    return map;
}
fn main() {
    println!("The task1:");
    //println!("  1) Get a string from a user input or use a pre-defined one");
    //println!("  2) Write function to parse this string and store in a hash-map:");
    //println!("  3) Split to sub-strings and use it as a key in hash-map");
    //println!("  4) Calculate position form the beginning of the string and store in a hash-map as a value for the key");
    //println!("  5) Print the hash-map");

    let predef_string = "this is a string. And this is a string too";

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("error: unable to read user input");
    println!("{}",user_input);

    if user_input == "\n" {
        println!("Using the pre-defined string as data: '{}'", predef_string);
        let pre_map = index_string(predef_string);
        println!("The data_map is '{:#?}'", pre_map);
    }
    else {
        println!("Hello, please, type your sentece... Press Enter in the end.", predef_string);
        let data_map = index_string(user_input.as_str());
        println!("The data_map is '{:#?}'", data_map);
    }
    println!("Good bye!")
}

#[test]
fn test_short() {
    let map = index_string("a b a");
    println!("{:?}", map);
    assert_eq!(map["a"], vec![0, 4]);
    assert_eq!(map["b"], vec![2]);
}

#[test]
fn test_long() {
    let map = index_string("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");
    println!("{:?}", map);
    assert_eq!(map["Lorem"].len(), 1);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["dolore"].len(), 2);
}

#[test]
fn test_cyrillic() {
    let map = index_string("привіт привіт");
    println!("{:?}", map);
    assert_eq!(map["привіт"], vec![0, 13]);
}
