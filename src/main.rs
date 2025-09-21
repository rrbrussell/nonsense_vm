use std::iter::Iterator;
use std::process::exit;

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
        eprintln!("I was unable to read {class_file_path:?} because of {}.",
            class_file_data.err().unwrap());
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
    println!("This class file uses version {major_version}.{minor_version} of \
the class file format.");
    let constant_pool_count = parse_u16(&class_file_data[8..10]);
    println!("There are {constant_pool_count} items in the contant_pool.");
    let mut probable_constant_pool = &class_file_data[10..].iter();
    match parse_constant_pool_tag(&mut probable_constant_pool.cloned()) {
        Some(t) => {
            println!("I read something");
        }
        None => {}
    }
    exit(0);
}

fn parse_u16(input: &[u8]) -> u16 {
    u16::from_be_bytes([input[0], input[1]])
}

enum ConstantPoolTag{
    Utf8 = 1,
    Integer = 3,
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,
    Fieldref = 9,
    Methodref = 10,
    InterfaceMethodref = 11,
    NameAndType = 12,
    MethodHandle = 15,
    MethodType = 16,
    InvokeDynamic = 18
}

fn parse_constant_pool_tag(iter: &mut impl Iterator<Item = u8>) ->
Option<ConstantPoolTag> {
    match iter.next() {
        Some(tag) => {
            match tag {
                01 => Some(ConstantPoolTag::Utf8),
                03 => Some(ConstantPoolTag::Integer),
                04 => Some(ConstantPoolTag::Float),
                05 => Some(ConstantPoolTag::Long),
                06 => Some(ConstantPoolTag::Double),
                07 => Some(ConstantPoolTag::Class),
                08 => Some(ConstantPoolTag::String),
                09 => Some(ConstantPoolTag::Fieldref),
                10 => Some(ConstantPoolTag::Methodref),
                11 => Some(ConstantPoolTag::InterfaceMethodref),
                12 => Some(ConstantPoolTag::NameAndType),
                15 => Some(ConstantPoolTag::MethodHandle),
                16 => Some(ConstantPoolTag::MethodType),
                18 => Some(ConstantPoolTag::InvokeDynamic),
                _ => None,                
            }
        }
        None => None,
    }
}
