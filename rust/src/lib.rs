// Time to build a program that parses Java Class files.

#![allow(unused)]

mod constantpool;

use std::collections::HashSet;

fn parse_f32(input: &[u8]) -> f32 {
    f32::from_be_bytes([input[0], input[1], input[2], input[3]])
}

fn parse_f64(input: &[u8]) -> f64 {
    f64::from_be_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6],
input[7],
    ])
}

fn parse_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes([input[0], input[1], input[2], input[3]])
}

fn parse_i64(input: &[u8]) -> i64 {
    i64::from_be_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6],
input[7],
    ])
}

pub fn parse_u16(input: &[u8]) -> u16 {
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
            let temp: u32 = ((datum & 0x0F) << 12) + ((datum_y & 0x3F) << 6) +
(datum_z & 0x3F);
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
        // U+00FF and uniquely to Java U+0000. These code points are spread
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

#[repr(u16)]
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum CFAccessFlags {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

pub fn parse_access_flags(input: u16) -> HashSet<CFAccessFlags> {
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
    if input & CFAccessFlags::Interface as u16 == CFAccessFlags::Interface as
u16 {
        set_flags.insert(CFAccessFlags::Interface);
    }
    if input & CFAccessFlags::Abstract as u16 == CFAccessFlags::Abstract as u16
{
        set_flags.insert(CFAccessFlags::Abstract);
    }
    if input & CFAccessFlags::Synthetic as u16 == CFAccessFlags::Synthetic as
u16 {
        set_flags.insert(CFAccessFlags::Synthetic);
    }
    if input & CFAccessFlags::Annotation as u16 == CFAccessFlags::Annotation as
u16 {
        set_flags.insert(CFAccessFlags::Annotation);
    }
    if input & CFAccessFlags::Enum as u16 == CFAccessFlags::Enum as u16 {
        set_flags.insert(CFAccessFlags::Enum);
    }
    return set_flags;
}

#[derive(Clone, Debug)]
pub enum ConstantPoolItem {
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

impl ConstantPoolItem {
    pub fn get_string(&self) -> &String {
        match self {
            ConstantPoolItem::Utf8(data) => {
                return data;
            }
            _ => {
                panic!(
                    "The ConstantPoolItem you called this on doesn't have\
getString support."
                );
            }
        }
    }
}

pub fn parse_constant_pool_tag(iter: &mut impl Iterator<Item = u8>) ->
Option<ConstantPoolItem> {
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

pub fn parse_constant_pool(constant_pool: &Vec<ConstantPoolItem>) -> bool {
    for (ind, t) in constant_pool.iter().enumerate() {
        match t {
            ConstantPoolItem::Utf8(data) => {
                println!("{ind}: I found the raw Utf data: {data}");
            }
            ConstantPoolItem::Integer(item) => {
                println!("{ind}: I found the integer {item}.");
            }
            ConstantPoolItem::Float(item) => {
                println!("{ind}: I found the float {item}.");
            }
            ConstantPoolItem::Long(item) => {
                println!("{ind}: I found the long {item}.");
            }
            ConstantPoolItem::Double(item) => {
                println!("{ind}: I found a double {item}.");
            }
            ConstantPoolItem::Class(name_index) => match
&constant_pool[*name_index - 1] {
                ConstantPoolItem::Utf8(name) => {
                    println!("{ind}: I found a class named: {name}");
                }
                _ => {
                    return false;
                }
            },
            ConstantPoolItem::String(string_index) => {
                println!(
                    "{ind}: I found a string. Its contents are at index
{string_index}."
                );
            }
            ConstantPoolItem::Fieldref(class, name_and_type) => {
                println!(
                    "{ind}: I found a field belonging to {class} that is named
and \
typed at {name_and_type}."
                );
            }
            ConstantPoolItem::Methodref(class, name_and_type) => {
                println!(
                    "{ind}: I found a method belonging to {class} that is named
and \
typed at {name_and_type}."
                );
            }
            ConstantPoolItem::InterfaceMethodref(class, name_and_type) => {
                println!(
                    "{ind}: I found an interface method belonging to {class} \
that is name and typed at {name_and_type}."
                );
            }
            ConstantPoolItem::NameAndType(name, descriptor) => {
                println!(
                    "{ind}: I found a member named at {name} and typed at \
{descriptor}."
                );
            }
            ConstantPoolItem::MethodHandle(kind, reference) => {
                println!(
                    "{ind}: I found a method handle of {kind} referencing \
the item at {reference}."
                );
            }
            ConstantPoolItem::MethodType(description) => {
                println!("{ind}: I found a method descriped at {description}.");
            }
            ConstantPoolItem::InvokeDynamic(bootstrap, name_and_type) => {
                println!(
                    "{ind}: I found a dynamic invocation boostrap method named
and \
typed at {name_and_type},"
                );
                println!(
                    "and descriped further at index {bootstrap} of the \
bootstrap table."
                );
            }
        }
    }
    return true;
}

pub struct ReferenceClassDescriptor {
    /// The constant pool index of the class in the Constant Pool.
    index: usize,
    /// The Decoded reference information.
    thing: FieldDescriptor,
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

fn parse_field_descriptor(descriptor: &str) -> Option<FieldDescriptor> {
    if descriptor.starts_with(&['B', 'C', 'D', 'F', 'I', 'J', 'S', 'Z']) {
        // This is the simple version. This field descriptor should be a single
        // character in length.
        if descriptor.len() != 1 {
            return None;
        }
        // Safety: We have already checked the length of the input string.
        match descriptor.chars().next().unwrap() {
            'B' => Some(FieldDescriptor::Byte),
            'C' => Some(FieldDescriptor::Char),
            'D' => Some(FieldDescriptor::Double),
            'F' => Some(FieldDescriptor::Float),
            'I' => Some(FieldDescriptor::Integer),
            'J' => Some(FieldDescriptor::Long),
            'Z' => Some(FieldDescriptor::Boolean),
            _ => None,
        }
    } else {
        if descriptor.starts_with(&['L', '[']) {
            // This is the more complex version. This Field descriptor should
            // be more than a single character in length.
            if descriptor.len() < 2 {
                return None;
            }
            let mut chars = descriptor.chars();
            let mut temp: Vec<char> = Vec::with_capacity(descriptor.len());
            let mut identifiers: Vec<String> = Vec::with_capacity(8);
            // Safety: We have already checked the length of the input string.
            match chars.next().unwrap() {
                'L' => {
                    while let Some(c) = chars.next() {
                        match c {
                            '.' | '[' => {
                                return None;
                            }
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
                }
                '[' => {
                    let mut depth: usize = 1;
                    let mut result: Option<FieldDescriptor> = None;
                    while let Some(c) = chars.next() {
                        match c {
                            '[' => depth += 1,
                            'L' => {
                                while let Some(c) = chars.next() {
                                    match c {
                                        '.' | '[' => {
                                            return None;
                                        }
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
                            }
                            'B' => {
                                result = Some(FieldDescriptor::Byte);
                                break;
                            }
                            'C' => {
                                result = Some(FieldDescriptor::Char);
                                break;
                            }
                            'D' => {
                                result = Some(FieldDescriptor::Double);
                                break;
                            }
                            'F' => {
                                result = Some(FieldDescriptor::Float);
                                break;
                            }
                            'I' => {
                                result = Some(FieldDescriptor::Integer);
                                break;
                            }
                            'J' => {
                                result = Some(FieldDescriptor::Long);
                                break;
                            }
                            'Z' => {
                                result = Some(FieldDescriptor::Boolean);
                                break;
                            }
                            _ => {
                                return None;
                            }
                        }
                    }
                    if depth <= 255 && result.is_some() {
                        Some(FieldDescriptor::Array(
                            depth as u8,
                            Box::new(result.unwrap()),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            // The descriptor is not a valid field descriptor.
            None
        }
    }
}
