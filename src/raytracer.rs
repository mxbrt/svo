use crate::bvh;
use crate::camera::Camera;
use crate::shader;
use crate::svo::SparseVoxelOctree;

#[derive(Clone, Copy, Default, glsl_layout::AsStd140)]
struct RaytracerShaderUniforms {
    camera_rotation: [[f32; 4]; 4],
    camera_origin: [f32; 4],
    width: u32,
    height: u32,
    aspect_ratio: f32,
    fov: f32,
    bvh_root: u32,
}

unsafe impl bytemuck::Pod for RaytracerShaderUniforms {}
unsafe impl bytemuck::Zeroable for RaytracerShaderUniforms {}

pub struct Raytracer {
    uniform_buffer: wgpu::Buffer,
    _svo_buffer: wgpu::Buffer,
    bvh_node_buffer: wgpu::Buffer,
    bvh_leaf_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Raytracer {
    pub fn new(device: &mut wgpu::Device, svo: &SparseVoxelOctree) -> Raytracer {
        let vs_module = shader::load("shaders/quad.vert", &device);
        let fs_module = shader::load("shaders/raytrace.frag", &device);
        let uniform_size = std::mem::size_of::<RaytracerShaderUniforms>() as u64;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<RaytracerShaderUniforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let svo_bytes = bytemuck::cast_slice(&svo.node_pool[..]);
        let svo_buffer = device.create_buffer_with_data(svo_bytes, wgpu::BufferUsage::STORAGE_READ);

        // for now, we allow a maximum of 16384 rendered objects.
        let bvh_max_leafs = 2u64.pow(16);
        let bvh_max_nodes = bvh_max_leafs * 2 - 1;
        let bvh_node_bufsize = bvh_max_nodes * std::mem::size_of::<bvh::BVHNode>() as u64;
        let bvh_leaf_bufsize = bvh_max_leafs * std::mem::size_of::<bvh::BVHLeaf>() as u64;
        let bvh_node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bvh_node_bufsize,
            usage: wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        });
        let bvh_leaf_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bvh_leaf_bufsize,
            usage: wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        });

        let bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
            ],
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bindgroup_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bindgroup_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..uniform_size as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &svo_buffer,
                        range: 0..svo_bytes.len() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &bvh_node_buffer,
                        range: 0..bvh_node_bufsize,
                    },
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &bvh_leaf_buffer,
                        range: 0..bvh_leaf_bufsize,
                    },
                },
            ],
            label: None,
        });

        Raytracer {
            uniform_buffer,
            _svo_buffer: svo_buffer,
            bvh_node_buffer,
            bvh_leaf_buffer,
            render_pipeline,
            bind_group,
        }
    }

    pub fn render(
        &self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        camera: &Camera,
        bvh: &bvh::BoundingVolumeHierarchy,
    ) {
        {
            let bvh_node_bytes = bytemuck::cast_slice(&bvh.nodes[..]);
            let bvh_leaf_bytes = bytemuck::cast_slice(&bvh.leafs[..]);
            let tmp_bvh_node_buffer =
                device.create_buffer_with_data(bvh_node_bytes, wgpu::BufferUsage::COPY_SRC);
            let tmp_bvh_leaf_buffer =
                device.create_buffer_with_data(bvh_leaf_bytes, wgpu::BufferUsage::COPY_SRC);

            encoder.copy_buffer_to_buffer(
                &tmp_bvh_node_buffer,
                0,
                &self.bvh_node_buffer,
                0,
                bvh_node_bytes.len() as wgpu::BufferAddress,
            );
            encoder.copy_buffer_to_buffer(
                &tmp_bvh_leaf_buffer,
                0,
                &self.bvh_leaf_buffer,
                0,
                bvh_leaf_bytes.len() as wgpu::BufferAddress,
            );

            let raytracer_uniform = RaytracerShaderUniforms {
                camera_rotation: *camera.rotation.as_ref(),
                camera_origin: *camera.position.coords.as_ref(),
                width,
                height,
                aspect_ratio: width as f32 / height as f32,
                fov: f32::tan(camera.fov.to_radians() / 2.0),
                bvh_root: bvh.root,
            };

            let uniform_size = std::mem::size_of::<RaytracerShaderUniforms>() as u64;
            let tmp_uniform_buffer = device.create_buffer_with_data(
                bytemuck::bytes_of(&raytracer_uniform),
                wgpu::BufferUsage::COPY_SRC,
            );

            encoder.copy_buffer_to_buffer(
                &tmp_uniform_buffer,
                0,
                &self.uniform_buffer,
                0,
                uniform_size as wgpu::BufferAddress,
            );
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: frame_view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..4, 0..1);
        }
    }
}
