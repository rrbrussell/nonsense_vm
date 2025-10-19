#![allow(unused)]

use nonesense::ConstantPoolItem as ConstantPoolItem;
use nonesense::parse_access_flags as parse_access_flags;
use nonesense::parse_constant_pool as parse_constant_pool;
use nonesense::parse_constant_pool_tag as parse_constant_pool_tag;
use nonesense::parse_u16 as parse_u16;

use std::collections::HashSet;
use std::iter::Iterator;
use std::process::exit;
use std::vec::Vec;

fn main() {
    // Let's handle basic command line arguments and reading the file.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("I need a least 2 arguments.");
        exit(1);
    }
    let working_directory = std::env::current_dir();
    if working_directory.is_err() {
        eprintln!("Please fix {}", working_directory.err().unwrap());
        exit(2);
    }
    let working_directory = working_directory.unwrap();
    let class_file_path = working_directory.join(args[1].to_owned());
    let class_file_data = std::fs::read(&class_file_path);
    if class_file_data.is_err() {
        eprintln!(
            "I was unable to read {class_file_path:?} because of {}.",
            class_file_data.err().unwrap()
        );
        exit(3);
    }
    let class_file_data = class_file_data.unwrap();
    // Now we can get around to parsing a class file.
    // The first 4 bytes are going to be the magic value.
    let magic = &class_file_data[0..4];
    if magic != [0xCA, 0xFE, 0xBA, 0xBE] {
        eprintln!("Invalid magic value:");
        exit(4);
    }
    println!("I read the expected magic value of {:X?}", &magic);
    let minor_version = parse_u16(&class_file_data[4..6]);
    let major_version = parse_u16(&class_file_data[6..8]);
    println!(
        "This class file uses version {major_version}.{minor_version} of \
the class file format."
    );
    let constant_pool_count = parse_u16(&class_file_data[8..10]) - 1;
    let mut constant_pool: Vec<ConstantPoolItem> = Vec::with_capacity(constant_pool_count as usize);
    println!("There are {constant_pool_count} items in the contant_pool.");
    let mut raw_class_file_data = class_file_data[10..].iter();
    for _ in 1..=constant_pool_count {
        match parse_constant_pool_tag(&mut raw_class_file_data.by_ref().copied()) {
            Some(t) => {
                constant_pool.push(t.to_owned());
            }
            None => {
                break;
            }
        }
    }
    println!("The constant pool has been read.");
    if constant_pool.len() != constant_pool_count as usize {
        eprintln!("Unable to read the constant_pool correctly.");
        dbg!(constant_pool);
        exit(5);
    }
    if parse_constant_pool(&constant_pool) {
        println!("Parsed constant pool correctly.");
    } else {
        eprintln!("Unable to parse constant pool correctly.");
        dbg!(constant_pool);
        exit(5);
    }
    let mut temp: Vec<u8> = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("Access Flags are missing from the class file.");
        exit(6);
    }
    let access_flags = parse_access_flags(parse_u16(&temp[..]));
    println!("The following Access Flags were set: {access_flags:?}");
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("The 'this_class' item is missing from the class file.");
        exit(7);
    }
    let this_class: u16 = parse_u16(&temp[..]);
    print!("This file defines the class described in Constant Pool Entry");
    println!(" {this_class}.");
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("The 'super_class' item is missing from the class file.");
        exit(7);
    }
    let super_class: u16 = parse_u16(&temp[..]);
    print!("This class has the superclass described in Constant Pool Entry");
    println!(" {super_class}.");
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("The 'interfaces_count' item is missing from the class file.");
        exit(7);
    }
    let interfaces_count: u16 = parse_u16(&temp[..]);
    println!("This class implements {interfaces_count} interfaces.");
    if interfaces_count != 0 {
        eprintln!("I cannot parse the interfaces currently.");
        exit(0);
    }
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("The 'fields_count' item is missing from the class file.");
        exit(7);
    }
    let fields_count: u16 = parse_u16(&temp[..]);
    println!("This class has {fields_count} fields.");
    if fields_count != 0 {
        eprintln!("I cannot parse the fields currently.");
        exit(0);
    }
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!("The 'methods_count' item is missing from the class file.");
        exit(7);
    }
    let methods_count: u16 = parse_u16(&temp[..]);
    println!("This class implements {methods_count} methods.");
    if methods_count != 0 {
        todo!("Parse the methods.");
    }
    temp = raw_class_file_data.by_ref().take(2).copied().collect();
    if temp.len() != 2 {
        eprintln!(
            "The 'attributes_count' item is missing from the class \
file."
        );
        exit(7);
    }
    let attributes_count: u16 = parse_u16(&temp[..]);
    println!("There are {attributes_count} attributes in this class.");
    println!(
        "There are {} bytes of unprocessed input left.",
        raw_class_file_data.count()
    );
    exit(0);
}

