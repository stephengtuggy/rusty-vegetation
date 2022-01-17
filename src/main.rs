#[macro_use] extern crate random_number;

use std::iter;
use std::env;
use std::str::FromStr;
use std::vec::Vec;
use cgmath::num_traits::Pow;
use is_odd::IsOdd;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTEX_BUFFER_LAYOUT_ATTRIBUTES: &[wgpu::VertexAttribute; 2] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: VERTEX_BUFFER_LAYOUT_ATTRIBUTES,
        }
    }
}

const GREEN: [f32; 3] = [0.0, 1.0, 0.0];
const BROWN: [f32; 3] = [0.25, 0.25, 0.0];

const X_SCALE: f32 = 1.0f32 / 256.0f32;
const Y_SCALE: f32 = 1.0f32 / 256.0f32;

#[derive(Clone, Copy)]
enum GrowthDirection {
    Left = 0,
    UpperLeft = 1,
    Up = 2,
    UpperRight = 3,
    Right = 4,
    LowerRight = 5,
    Down = 6,
    LowerLeft = 7,
}

const GROWTH_DIRECTIONS: [GrowthDirection; 8] = [
    GrowthDirection::Left,
    GrowthDirection::UpperLeft,
    GrowthDirection::Up,
    GrowthDirection::UpperRight,
    GrowthDirection::Right,
    GrowthDirection::LowerRight,
    GrowthDirection::Down,
    GrowthDirection::LowerLeft,
];

struct Tree {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

struct Forest {
    trees: Vec<Tree>,
}

impl Forest {
    pub fn new() -> Self {
        Self {
            trees: Vec::new(),
        }
    }
}

struct TreeGenerator {
    fractal_level: u8,
    completeness_factor: u8,
    num_trees: u16,
}

impl TreeGenerator {
    pub fn new (fractal_level: u8, completeness_factor: u8, num_trees: u16) -> Self {
        Self {
            fractal_level,
            completeness_factor,
            num_trees,
        }
    }

    pub fn generate_tree(tree: &mut Tree,
                     fractal_level: u32,
                     start_x: f32,
                     start_y: f32,
                     completeness_factor: u8,
                     direction: GrowthDirection) {
        let end_x: f32;
        let end_y: f32;

        match direction {
            GrowthDirection::Left => {
                end_x = start_x - X_SCALE * 2f32.pow(fractal_level);
                end_y = start_y;
            }
            GrowthDirection::UpperLeft => {
                end_x = start_x - X_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
                end_y = start_y - Y_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
            }
            GrowthDirection::Up => {
                end_x = start_x;
                end_y = start_y - Y_SCALE * 2f32.pow(fractal_level);
            }
            GrowthDirection::UpperRight => {
                end_x = start_x + X_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
                end_y = start_y - Y_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
            }
            GrowthDirection::Right => {
                end_x = start_x + X_SCALE * 2f32.pow(fractal_level);
                end_y = start_y;
            }
            GrowthDirection::LowerRight => {
                end_x = start_x + X_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
                end_y = start_y + Y_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
            }
            GrowthDirection::Down => {
                end_x = start_x;
                end_y = start_y + Y_SCALE * 2f32.pow(fractal_level);
            }
            GrowthDirection::LowerLeft => {
                end_x = start_x - X_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
                end_y = start_y + Y_SCALE * f32::sqrt(2f32.pow(fractal_level * 2) / 2f32);
            }
        }

        let mut color: [f32; 3] = BROWN;
        if fractal_level == 0 {
            color = GREEN;
        }

        let vertex_start: Vertex = Vertex { position: [start_x, start_y, 0.0f32], color };
        tree.vertices.push(vertex_start);
        tree.indices.push((tree.vertices.len() - 1) as u32);
        let vertex_end: Vertex = Vertex { position: [end_x, end_y, 0.0f32], color };
        tree.vertices.push(vertex_end);
        tree.indices.push((tree.vertices.len() - 1) as u32);

        if fractal_level > 0 {
            for i in GROWTH_DIRECTIONS {
                let n: u8 = random!();
                if (i as u32 != direction as u32) && (n < completeness_factor) {
                    TreeGenerator::generate_tree(tree, fractal_level - 1, end_x, end_y, completeness_factor, i);
                }
            }
        }
    }

    pub fn generate_forest(&mut self) -> Forest {
        let mut forest = Forest::new();

        for i in 0u16..self.num_trees {
            let x: f32 = 0.0f32; //random!(-1.0..=1.0);
            let mut tree = Tree::new();
            TreeGenerator::generate_tree(&mut tree, self.fractal_level as u32, x, 0.0f32, self.completeness_factor, GrowthDirection::Up);

            // if (tree.indices.len() as u64).is_odd() {
            //     tree.indices.push(*tree.indices.last().unwrap());
            // }

            forest.trees.push(tree);
        }

        forest
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    async fn new(window: &Window, tree_generator: &mut TreeGenerator) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,   // Trace path
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // let mut tree_generator = TreeGenerator::new(8u8, 192u8, 1u16);
        let forest = tree_generator.generate_forest();
        let trees = forest.trees;

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&(trees[0].vertices)),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&(trees[0].indices)),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = trees[0].indices.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let num_trees = u16::from_str(&args[1]).unwrap();
    let completeness_factor = u8::from_str(&args[2]).unwrap();
    let fractal_level = u8::from_str(&args[3]).unwrap();
    let mut tree_generator = TreeGenerator::new(fractal_level, completeness_factor, num_trees);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state: State = pollster::block_on(State::new(&window, &mut tree_generator));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory; we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, timeout) should b resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
