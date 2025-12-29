use crate::reflect::collect_all_types;
use spirv_reflect::ShaderModule;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_gpu(reflection: &ShaderModule, out_dir: &Path) {
    let descriptor_sets = reflection.enumerate_descriptor_sets(None).unwrap();
    let type_info = collect_all_types(&descriptor_sets);
    let mut file = File::create(out_dir.join("gpu.rs")).unwrap();
    for (struct_name, struct_) in &type_info.structs {
        let alignment = struct_.alignment;
        writeln!(
            file,
            r#"#[repr(C, align({alignment}))]
#[derive(Clone, Copy, Debug)]
pub struct {struct_name} {{"#
        )
        .unwrap();
        for (member_name, member_typ) in &struct_.members {
            let member_typ_rust = member_typ.to_rust();
            writeln!(file, "    pub {member_name}: {member_typ_rust},").unwrap();
        }
        writeln!(
            file,
            r#"}}
"#
        )
        .unwrap();
    }
}
