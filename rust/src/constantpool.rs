/// The constant pool is the primary source for symbolic information about the
/// Class or Interface contained in the class file.
pub struct ConstantPool {
    classes: Vec<()>,
    fields: Vec<()>,
    class_methods: Vec<()>,
    interface_methods: Vec<()>,
    strings: Vec<()>,
    integers: Vec<()>,
    floats: Vec<()>,
    longs: Vec<()>,
    doubles: Vec<()>,
    utf8: (),
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
