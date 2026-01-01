use spirv_reflect::types::{ReflectDescriptorSet, ReflectTypeDescription, ReflectTypeFlags};
use spirv_reflect::ShaderModule;
use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap};

pub struct TypeInfo<'a> {
    pub structs: HashMap<&'a str, Struct<'a>>,
}

pub enum Type<'a> {
    Struct(&'a str),
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

impl<'a> Type<'a> {
    pub fn to_rust(&self) -> Cow<'a, str> {
        match self {
            Type::Struct(name) => Cow::Borrowed(*name),
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

pub fn reflect_shaders() -> ShaderModule {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let spirv_path = out_dir.join("shaders").join("voxel.mesh.spv");
    let spirv = std::fs::read(&spirv_path).unwrap();
    spirv_reflect::create_shader_module(&spirv).unwrap()
}

pub fn collect_all_types(descriptor_sets: &[ReflectDescriptorSet]) -> TypeInfo<'_> {
    let mut structs = HashMap::new();
    for set in descriptor_sets {
        for binding in &set.bindings {
            collect_types(binding.type_description.as_ref().unwrap(), &mut structs);
        }
    }
    TypeInfo { structs }
}

fn collect_types<'a>(typ: &'a ReflectTypeDescription, structs: &mut HashMap<&'a str, Struct<'a>>) {
    for member in &typ.members {
        collect_types(member, structs);
    }
    if is_struct(typ) && !structs.contains_key(typ.type_name.as_str()) {
        let mut alignment: usize = 1;
        let mut members = Vec::new();

        for member in &typ.members {
            let member_typ = parse_type(member);
            let member_alignment = get_alignment(&member_typ, structs);
            alignment = alignment.max(member_alignment);
            members.push((member.struct_member_name.as_str(), member_typ));
        }

        if let [(_, Type::Array(_, 0))] = members.as_slice() {
            return;
        }

        let struct_ = Struct { alignment, members };
        structs.insert(&typ.type_name, struct_);
    }
}

fn is_struct(typ: &ReflectTypeDescription) -> bool {
    typ.type_flags.contains(ReflectTypeFlags::STRUCT)
        && !typ.type_name.starts_with('_')
        && typ.type_name != "StructuredBuffer"
}

fn parse_type(typ: &ReflectTypeDescription) -> Type<'_> {
    if typ.type_flags.contains(ReflectTypeFlags::ARRAY) {
        assert_eq!(typ.traits.array.dims.len(), 1);
        let inner = parse_type_nonarray(typ);
        let count = typ.traits.array.dims[0];
        Type::Array(Box::new(inner), count)
    } else {
        parse_type_nonarray(typ)
    }
}

fn parse_type_nonarray(typ: &ReflectTypeDescription) -> Type<'_> {
    if typ.type_flags.contains(ReflectTypeFlags::STRUCT) {
        assert!(!typ.type_name.is_empty());
        Type::Struct(&typ.type_name)
    } else if typ.type_flags.contains(ReflectTypeFlags::MATRIX) {
        assert!(typ.type_flags.contains(ReflectTypeFlags::FLOAT));
        assert_eq!(typ.traits.numeric.scalar.width, 32);
        assert_eq!(typ.traits.numeric.matrix.row_count, 4);
        assert_eq!(typ.traits.numeric.matrix.column_count, 4);
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
        todo!("typ={typ:?}")
    }
}

fn get_alignment(typ: &Type, structs: &HashMap<&str, Struct>) -> usize {
    match typ {
        Type::Struct(name) => structs[name].alignment,
        Type::Array(subtyp, _) => get_alignment(subtyp, structs),
        Type::Vector(scalar, _) => scalar.alignment(),
        Type::Scalar(scalar) => scalar.alignment(),
        Type::Matrix4F32 => Scalar::F32.alignment(),
    }
}
