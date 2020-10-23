use std::error::Error;

use shaderc::{
    CompilationArtifact, CompileOptions, Compiler, IncludeType, ResolvedInclude, ShaderKind,
};

pub fn load(path: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    let spirv = match compile_glsl(path) {
        Ok(spirv) => spirv,
        Err(e) => panic!("Failed to load shader {}\n{}", path, e),
    };
    device.create_shader_module(wgpu::ShaderModuleSource::SpirV(spirv.as_binary().into()))
}

fn include_callback(
    path: &str,
    _: IncludeType,
    _: &str,
    _: usize,
) -> Result<ResolvedInclude, String> {
    match std::fs::read_to_string(path) {
        Ok(source) => Ok(ResolvedInclude {
            resolved_name: String::from(path),
            content: source,
        }),
        Err(e) => Err(format!("Failed to load included file {}\n{}", path, e)),
    }
}

fn compile_glsl(path: &str) -> Result<CompilationArtifact, Box<dyn Error>> {
    let mut compiler = Compiler::new().unwrap();
    let mut options = CompileOptions::new().unwrap();
    options.set_include_callback(include_callback);
    let _path = std::path::Path::new(path);
    let extension = _path.extension().unwrap().to_str().unwrap();
    let shader_kind = match extension {
        "vert" => ShaderKind::Vertex,
        "frag" => ShaderKind::Fragment,
        "comp" => ShaderKind::Compute,
        _ => return Err(format!("Invalid shader extension {}", extension).into()),
    };
    let source = std::fs::read_to_string(path)?;
    let spirv = compiler.compile_into_spirv(&source, shader_kind, path, "main", Some(&options))?;
    Ok(spirv)
}
