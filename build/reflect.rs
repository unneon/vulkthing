use spirv_reflect::types::{ReflectDescriptorSet, ReflectTypeDescription, ReflectTypeFlags};
use std::{borrow::Cow, collections::HashMap};

pub struct TypeInfo<'a> {
    pub structs: HashMap<(&'a str, Layout), Struct<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Layout {
    Std140,
    Std430,
}

pub enum Type<'a> {
    Struct(&'a str, Layout),
    Array(Box<Type<'a>>, u32),
    Vector(Scalar, u32),
    Scalar(Scalar),
    Matrix4F32,
}

pub struct Struct<'a> {
    pub alignment: usize,
    pub members: Vec<(&'a str, Type<'a>)>,
}

pub enum Scalar {
    F32,
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
}

impl Layout {
    pub fn lowercase(&self) -> &'static str {
        match self {
            Layout::Std140 => "std140",
            Layout::Std430 => "std430",
        }
    }
}

impl<'a> Type<'a> {
    pub fn to_rust(&self) -> Cow<'a, str> {
        match self {
            Type::Struct(name, _) => Cow::Borrowed(*name),
            Type::Array(subtyp, count) => Cow::Owned(format!("[{}; {count}]", subtyp.to_rust())),
            Type::Vector(scalar, components) => Cow::Owned(format!(
                "nalgebra::Vector{components}<{}>",
                scalar.to_rust()
            )),
            Type::Scalar(scalar) => Cow::Borrowed(scalar.to_rust()),
            Type::Matrix4F32 => Cow::Borrowed("nalgebra::Matrix4<f32>"),
        }
    }
}

impl Scalar {
    fn to_rust(&self) -> &'static str {
        match self {
            Scalar::F32 => "f32",
            Scalar::U8 => "u8",
            Scalar::U16 => "u16",
            Scalar::U32 => "u32",
            Scalar::I8 => "i8",
            Scalar::I16 => "i16",
            Scalar::I32 => "i32",
        }
    }

    fn alignment(&self) -> usize {
        match self {
            Scalar::F32 => 4,
            Scalar::U8 => 1,
            Scalar::U16 => 2,
            Scalar::U32 => 4,
            Scalar::I8 => 1,
            Scalar::I16 => 2,
            Scalar::I32 => 4,
        }
    }
}

pub fn reflect(descriptor_sets: &[ReflectDescriptorSet]) -> TypeInfo<'_> {
    let mut structs = HashMap::new();
    for set in descriptor_sets {
        for binding in &set.bindings {
            collect_types(binding.type_description.as_ref().unwrap(), &mut structs);
        }
    }
    TypeInfo { structs }
}

fn collect_types<'a>(
    typ: &'a ReflectTypeDescription,
    structs: &mut HashMap<(&'a str, Layout), Struct<'a>>,
) {
    for member in &typ.members {
        collect_types(member, structs);
    }
    if is_struct(typ) {
        let (name, layout) = parse_struct_name(typ);
        if !structs.contains_key(&(name, layout)) {
            let mut alignment: usize = 1;
            let mut members = Vec::new();

            for member in &typ.members {
                let member_typ = parse_type(member);
                let member_alignment = get_alignment(&member_typ, layout, structs);
                alignment = alignment.max(member_alignment);
                members.push((member.struct_member_name.as_str(), member_typ));
            }

            if layout == Layout::Std140 {
                alignment = alignment.next_multiple_of(16);
            }

            let struct_ = Struct { alignment, members };
            structs.insert((name, layout), struct_);
        }
    }
}

fn is_struct(typ: &ReflectTypeDescription) -> bool {
    typ.type_flags.contains(ReflectTypeFlags::STRUCT)
        && !typ.type_name.starts_with('_')
        && typ.type_name != "StructuredBuffer"
}

fn parse_struct_name(typ: &ReflectTypeDescription) -> (&str, Layout) {
    if let Some(name) = typ.type_name.strip_suffix("_std140") {
        (name, Layout::Std140)
    } else if let Some(name) = typ.type_name.strip_suffix("_std430") {
        (name, Layout::Std430)
    } else {
        unreachable!()
    }
}

fn parse_type(typ: &ReflectTypeDescription) -> Type<'_> {
    if let Some(name) = typ.type_name.strip_suffix("_std140") {
        Type::Struct(name, Layout::Std140)
    } else if let Some(name) = typ.type_name.strip_suffix("_std430") {
        Type::Struct(name, Layout::Std430)
    } else if typ.type_name.starts_with("_Array") {
        let typ = &typ.members[0];
        let subtyp = parse_type(typ);
        assert_eq!(typ.traits.array.dims.len(), 1);
        let count = typ.traits.array.dims[0];
        Type::Array(Box::new(subtyp), count)
    } else if typ
        .type_name
        .starts_with("_MatrixStorage_float4x4_ColMajor")
    {
        // TODO: Is there a difference between std140 and std340 matrices?
        Type::Matrix4F32
    } else if typ.type_flags.contains(ReflectTypeFlags::VECTOR) {
        let subtyp = parse_scalar(typ);
        let components = typ.traits.numeric.vector.component_count;
        Type::Vector(subtyp, components)
    } else {
        Type::Scalar(parse_scalar(typ))
    }
}

fn parse_scalar(typ: &ReflectTypeDescription) -> Scalar {
    if typ.type_flags.contains(ReflectTypeFlags::FLOAT) && typ.traits.numeric.scalar.width == 32 {
        Scalar::F32
    } else if typ.type_flags.contains(ReflectTypeFlags::INT) {
        match (
            typ.traits.numeric.scalar.width,
            typ.traits.numeric.scalar.signedness,
        ) {
            (8, 0) => Scalar::U8,
            (16, 0) => Scalar::U16,
            (32, 0) => Scalar::U32,
            (8, 1) => Scalar::I8,
            (16, 1) => Scalar::I16,
            (32, 1) => Scalar::I32,
            _ => todo!("{typ:?}"),
        }
    } else {
        todo!()
    }
}

fn get_alignment(typ: &Type, layout: Layout, structs: &HashMap<(&str, Layout), Struct>) -> usize {
    match typ {
        Type::Struct(name, layout) => structs[&(*name, *layout)].alignment,
        Type::Array(subtyp, _) => match layout {
            Layout::Std140 => get_alignment(subtyp, layout, structs).next_multiple_of(16),
            Layout::Std430 => get_alignment(subtyp, layout, structs),
        },
        Type::Vector(scalar, components) => match components {
            1 => scalar.alignment(),
            2 => 2 * scalar.alignment(),
            3..=4 => 4 * scalar.alignment(),
            _ => unreachable!(),
        },
        Type::Scalar(scalar) => scalar.alignment(),
        Type::Matrix4F32 => 16,
    }
}
