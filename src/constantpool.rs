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
