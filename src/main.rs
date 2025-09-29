#![allow(unused)]

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

fn parse_f32(input: &[u8]) -> f32 {
    f32::from_be_bytes([input[0], input[1], input[2], input[3]])
}

fn parse_f64(input: &[u8]) -> f64 {
    f64::from_be_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
    ])
}

fn parse_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes([input[0], input[1], input[2], input[3]])
}

fn parse_i64(input: &[u8]) -> i64 {
    i64::from_be_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
    ])
}

fn parse_u16(input: &[u8]) -> u16 {
    u16::from_be_bytes([input[0], input[1]])
}

fn parse_javaized_utf8(input: &Vec<u8>) -> Option<String> {
    let mut char_data: Vec<char> = Vec::with_capacity(input.len());
    let mut index: usize = 0;
    while index < input.len() {
        let datum: u32 = input[index] as u32;
        // Is the current datum we are working on the first of a multi-byte
        // character
        // This particular sentinel value marks code points above U+FFFF which
        // are encoded using a 6 byte encoding for the two surrogate code
        // points.
        if datum == 0xED {
            // Do we have enough input data?
            if (index + 5) >= input.len() {
                return None;
            }
            let datum_v: u32 = input[index + 1] as u32;
            let datum_w: u32 = input[index + 2] as u32;
            let datum_x: u32 = input[index + 3] as u32;
            let datum_y: u32 = input[index + 4] as u32;
            let datum_z: u32 = input[index + 5] as u32;
            // Do the non code point parts of the bytes match the expected
            // pattern?
            if datum_v & 0xA0 != 0xA0
                || datum_w & 0x80 != 0x80
                || datum_x != 0xED
                || datum_y & 0xB0 != 0xB0
                || datum_z & 0x80 != 0x80
            {
                return None;
            }
            let temp: u32 = 0x10000 as u32
                + ((datum_v & 0x0F) << 16)
                + ((datum_w & 0x3F) << 10)
                + ((datum_y & 0x0F) << 6)
                + (datum_z & 0x3F);
            match char::from_u32(temp) {
                None => {
                    return None;
                }
                Some(a) => {
                    char_data.push(a);
                    index += 6;
                    continue;
                }
            }
        }
        // This particular sentinel value marks code points from U+0800 to
        // U+FFFF. These code points are spread over 3 bytes.
        if datum & 0xE0 == 0xE0 {
            // Do we have enough input data?
            if (index + 2) >= input.len() {
                return None;
            }
            let datum_y: u32 = input[index + 1] as u32;
            let datum_z: u32 = input[index + 2] as u32;
            if (datum_y & datum_z & 0x80) != 0x80 {
                return None;
            }
            let temp: u32 = ((datum & 0x0F) << 12) + ((datum_y & 0x3F) << 6) + (datum_z & 0x3F);
            match char::from_u32(temp) {
                None => {
                    return None;
                }
                Some(a) => {
                    char_data.push(a);
                    index += 3;
                    continue;
                }
            }
        }
        // This particular sentinal value marks code points from U+0080 to
        // U+00FF and uniquely to Java U+0000. These code points are spread over
        // 2 bytes.
        if datum & 0xC0 == 0xC0 {
            // Do we have enough input data?
            if (index + 1) >= input.len() {
                return None;
            }
            let datum_y: u32 = input[index + 1] as u32;
            if datum_y & 0x80 != 0x80 {
                return None;
            }
            let temp: u32 = ((datum & 0x1F) << 6) + (datum_y & 0x3F);
            match char::from_u32(temp) {
                None => {
                    return None;
                }
                Some(a) => {
                    char_data.push(a);
                    index += 2;
                    continue;
                }
            }
        }
        // Everything else is plain old ASCII.
        match char::from_u32(datum) {
            Some(a) => {
                char_data.push(a);
                index += 1;
            }
            None => {
                return None;
            }
        }
    }
    Some(String::from_iter(char_data))
}

#[derive(Clone, Debug)]
enum ConstantPoolItem {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(usize),
    String(usize),
    Fieldref(usize, usize),
    Methodref(usize, usize),
    InterfaceMethodref(usize, usize),
    NameAndType(usize, usize),
    MethodHandle(u8, usize),
    MethodType(usize),
    InvokeDynamic(usize, usize),
}

fn parse_constant_pool_tag(iter: &mut impl Iterator<Item = u8>) -> Option<ConstantPoolItem> {
    let mut temp_storage: Vec<u8>;
    match iter.next() {
        Some(tag) => match tag {
            01 => {
                temp_storage = iter.by_ref().take(2).collect();
                if temp_storage.len() != 2 {
                    return None;
                }
                let length = parse_u16(&temp_storage[..]);
                temp_storage = iter.by_ref().take(length as usize).collect();
                if temp_storage.len() != length as usize {
                    return None;
                }
                match parse_javaized_utf8(&temp_storage) {
                    Some(temp) => Some(ConstantPoolItem::Utf8(temp)),
                    None => None,
                }
            }
            03 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::Integer(parse_i32(&temp_storage[..])))
            }
            04 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::Float(parse_f32(&temp_storage[..])))
            }
            05 => {
                temp_storage = iter.by_ref().take(8).collect();
                if temp_storage.len() != 8 {
                    return None;
                }
                Some(ConstantPoolItem::Long(parse_i64(&temp_storage[..])))
            }
            06 => {
                temp_storage = iter.by_ref().take(8).collect();
                if temp_storage.len() != 8 {
                    return None;
                }
                Some(ConstantPoolItem::Double(parse_f64(&temp_storage[..])))
            }
            07 => {
                temp_storage = iter.by_ref().take(2).collect();
                if temp_storage.len() != 2 {
                    return None;
                }
                Some(ConstantPoolItem::Class(
                    parse_u16(&temp_storage[..]) as usize
                ))
            }
            08 => {
                temp_storage = iter.by_ref().take(2).collect();
                if temp_storage.len() != 2 {
                    return None;
                }
                Some(ConstantPoolItem::String(
                    parse_u16(&temp_storage[..]) as usize
                ))
            }
            09 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::Fieldref(
                    parse_u16(&temp_storage[0..2]) as usize,
                    parse_u16(&temp_storage[2..]) as usize,
                ))
            }
            10 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::Methodref(
                    parse_u16(&temp_storage[0..2]) as usize,
                    parse_u16(&temp_storage[2..]) as usize,
                ))
            }
            11 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::InterfaceMethodref(
                    parse_u16(&temp_storage[0..2]) as usize,
                    parse_u16(&temp_storage[2..]) as usize,
                ))
            }
            12 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::NameAndType(
                    parse_u16(&temp_storage[0..2]) as usize,
                    parse_u16(&temp_storage[2..]) as usize,
                ))
            }
            15 => {
                temp_storage = iter.by_ref().take(3).collect();
                if temp_storage.len() != 3 {
                    return None;
                }
                Some(ConstantPoolItem::MethodHandle(
                    temp_storage[0],
                    parse_u16(&temp_storage[1..]) as usize,
                ))
            }
            16 => {
                temp_storage = iter.by_ref().take(2).collect();
                if temp_storage.len() != 2 {
                    return None;
                }
                Some(ConstantPoolItem::MethodType(
                    parse_u16(&temp_storage[..]) as usize
                ))
            }
            18 => {
                temp_storage = iter.by_ref().take(4).collect();
                if temp_storage.len() != 4 {
                    return None;
                }
                Some(ConstantPoolItem::InvokeDynamic(
                    parse_u16(&temp_storage[0..2]) as usize,
                    parse_u16(&temp_storage[2..]) as usize,
                ))
            }
            _ => None,
        },
        None => None,
    }
}

#[repr(u16)]
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
enum CFAccessFlags {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

fn parse_access_flags(input: u16) -> HashSet<CFAccessFlags> {
    let mut set_flags: HashSet<CFAccessFlags> = HashSet::with_capacity(8);
    if input & CFAccessFlags::Public as u16 == CFAccessFlags::Public as u16 {
        set_flags.insert(CFAccessFlags::Public);
    }
    if input & CFAccessFlags::Final as u16 == CFAccessFlags::Final as u16 {
        set_flags.insert(CFAccessFlags::Final);
    }
    if input & CFAccessFlags::Super as u16 == CFAccessFlags::Final as u16 {
        set_flags.insert(CFAccessFlags::Super);
    }
    if input & CFAccessFlags::Interface as u16 == CFAccessFlags::Interface as u16 {
        set_flags.insert(CFAccessFlags::Interface);
    }
    if input & CFAccessFlags::Abstract as u16 == CFAccessFlags::Abstract as u16 {
        set_flags.insert(CFAccessFlags::Abstract);
    }
    if input & CFAccessFlags::Synthetic as u16 == CFAccessFlags::Synthetic as u16 {
        set_flags.insert(CFAccessFlags::Synthetic);
    }
    if input & CFAccessFlags::Annotation as u16 == CFAccessFlags::Annotation as u16 {
        set_flags.insert(CFAccessFlags::Annotation);
    }
    if input & CFAccessFlags::Enum as u16 == CFAccessFlags::Enum as u16 {
        set_flags.insert(CFAccessFlags::Enum);
    }
    return set_flags;
}

fn parse_constant_pool(constant_pool: &Vec<ConstantPoolItem>) -> bool {
    for t in constant_pool.iter() {
        match t {
            ConstantPoolItem::Utf8(data) => {
                println!("I found the raw Utf data: {data}");
            }
            ConstantPoolItem::Integer(item) => {
                println!("I found the integer {item}.");
            }
            ConstantPoolItem::Float(item) => {
                println!("I found the float {item}.");
            }
            ConstantPoolItem::Long(item) => {
                println!("I found the long {item}.");
            }
            ConstantPoolItem::Double(item) => {
                println!("I found a double {item}.");
            }
            ConstantPoolItem::Class(name_index) => {
                match &constant_pool[*name_index - 1] {
                    ConstantPoolItem::Utf8(name) => {
                        println!("I found a class named: {name}");
                    },
                    _ => { return false; }
                }
            }
            ConstantPoolItem::String(string_index) => {
                println!("I found a string. Its contents are at index {string_index}.");
            }
            ConstantPoolItem::Fieldref(class, name_and_type) => {
                println!(
                    "I found a field belonging to {class} that is named and \
typed at {name_and_type}."
                );
            }
            ConstantPoolItem::Methodref(class, name_and_type) => {
                println!(
                    "I found a method belonging to {class} that is named and \
typed at {name_and_type}."
                );
            }
            ConstantPoolItem::InterfaceMethodref(class, name_and_type) => {
                println!(
                    "I found an interface method belonging to {class} \
that is name and typed at {name_and_type}."
                );
            }
            ConstantPoolItem::NameAndType(name, descriptor) => {
                println!(
                    "I found a member named at {name} and typed at \
{descriptor}."
                );
            }
            ConstantPoolItem::MethodHandle(kind, reference) => {
                println!(
                    "I found a method handle of {kind} referencing \
the item at {reference}."
                );
            }
            ConstantPoolItem::MethodType(description) => {
                println!("I found a method descriped at {description}.");
            }
            ConstantPoolItem::InvokeDynamic(bootstrap, name_and_type) => {
                println!(
                    "I found a dynamic invocation boostrap method \
named and type at {name_and_type},"
                );
                println!(
                    "and descriped further and index {bootstrap} \
of the bootstrap table."
                );
            }
        }
    }
    return true;
}

enum FieldDescriptor {
    Byte,
    Boolean,
    Char,
    Double,
    Float,
    Integer,
    Long,
    Reference(Vec<String>),
    Short,
    Array(u8, Box<FieldDescriptor>),
}

fn parse_field_descriptor(descriptor: &str) ->
Option<FieldDescriptor> {
    if descriptor.starts_with(&['B','C','D','F','I','J','S','Z']) {
        // This is the simple version. This field descriptor should be a single
        // character in length.
        if descriptor.len() != 1 { return None; }
        // Safety: We have already checked the length of the input string.
        match descriptor.chars().next().unwrap() {
            'B' => Some(FieldDescriptor::Byte),
            'C' => Some(FieldDescriptor::Char),
            'D' => Some(FieldDescriptor::Double),
            'F' => Some(FieldDescriptor::Float),
            'I' => Some(FieldDescriptor::Integer),
            'J' => Some(FieldDescriptor::Long),
            'Z' => Some(FieldDescriptor::Boolean),
            _ => None
        }
    } else {
        if descriptor.starts_with(&['L', '[']) {
            // This is the more complex version. This Field descriptor should be
            // more than a single character in length.
            if descriptor.len() < 2 { return None; }
            let mut chars = descriptor.chars();
            let mut temp: Vec<char> = Vec::with_capacity(descriptor.len());
            let mut identifiers: Vec<String> = Vec::with_capacity(8);
            // Safety: We have already checked the length of the input string.
            match chars.next().unwrap() {
                'L' => {
                    while let Some(c) = chars.next() {
                        match c {
                            '.' | '[' => { return None; },
                            '/' => {
                                identifiers.push(temp.iter().collect());
                                temp.clear();
                            }
                            ';' => {
                                identifiers.push(temp.iter().collect());
                                temp.clear();
                                break;
                            }
                            _ => temp.push(c),
                        }
                    }
                    Some(FieldDescriptor::Reference(identifiers))
                },
                '[' => {
                    let mut depth: usize = 1;
                    let mut result: Option<FieldDescriptor> = None;
                    while let Some(c) = chars.next() {
                        match c {
                            '[' => depth += 1,
                            'L' => {
                                while let Some(c) = chars.next() {
                                    match c {
                                        '.' | '[' => { return None; },
                                        '/' => {
                                            identifiers.push(temp.iter().collect());
                                            temp.clear();
                                        }
                                        ';' => {
                                            identifiers.push(temp.iter().collect());
                                            temp.clear();
                                            break;
                                        }
                                        _ => temp.push(c),
                                    }
                                }
                                result =
                                Some(FieldDescriptor::Reference(identifiers));
                                break;
                            },
                            'B' => {
                                result = Some(FieldDescriptor::Byte);
                                break;
                            },
                            'C' => {
                                result = Some(FieldDescriptor::Char);
                                break;
                            },
                            'D' => {
                                result = Some(FieldDescriptor::Double);
                                break;
                            },
                            'F' => {
                                result = Some(FieldDescriptor::Float);
                                break;
                            },
                            'I' => {
                                result = Some(FieldDescriptor::Integer);
                                break;
                            },
                            'J' => {
                                result = Some(FieldDescriptor::Long);
                                break;
                            },
                            'Z' => {
                                result = Some(FieldDescriptor::Boolean);
                                break;
                            },
                            _ => { return None; }
                        }
                    }
                    if depth <= 255 && result.is_some() {
                        Some(FieldDescriptor::Array(depth as u8,
                            Box::new(result.unwrap())))
                    } else {
                        None
                    }
                },
                _ => None
            }
        } else {
        // The descriptor is not a valid field descriptor.
        None
        }
    }
}
