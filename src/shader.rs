use std::error::Error;

use glsl_to_spirv::ShaderType;

pub fn load(path: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    let spirv = match compile_glsl(path) {
        Ok(spirv) => spirv,
        Err(e) => panic!("Failed to load shader {}\n{}", path, e),
    };
    device.create_shader_module(&spirv)
}

fn compile_glsl(path: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    let _path = std::path::Path::new(path);
    let extension = _path.extension().unwrap().to_str().unwrap();
    let shader_type = match extension {
        "vert" => ShaderType::Vertex,
        "frag" => ShaderType::Fragment,
        "comp" => ShaderType::Compute,
        _ => return Err(format!("Unknown shader extension {}", extension).into()),
    };
    let code = std::fs::read_to_string(path)?;
    let spirv = glsl_to_spirv::compile(&code, shader_type)?;
    let wgpu_spirv = wgpu::read_spirv(spirv)?;
    Ok(wgpu_spirv)
}
