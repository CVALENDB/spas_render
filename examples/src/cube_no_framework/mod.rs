#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

use std::sync::Arc;

fn vertex(pos: [f32; 3]) -> glam::Vec3 {
    glam::Vec3::from(pos)
}

fn create_mesh() -> spas_render::types::Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 1.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        // near side (0.0, 0.0, -1.0)
        vertex([-1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // right side (1.0, 0.0, 0.0)
        vertex([1.0, -1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        // left side (-1.0, 0.0, 0.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // top (0.0, 1.0, 0.0)
        vertex([1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        // bottom (0.0, -1.0, 0.0)
        vertex([1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // far
        4, 5, 6, 6, 7, 4, // near
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    spas_render::types::MeshBuilder::new(vertex_positions.to_vec(), spas_render::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()
        .unwrap()
}

#[allow(deprecated)]
pub fn main() {
    // Setup logging
    env_logger::init();

    // Create event loop and window
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = {
        let builder = winit::window::WindowAttributes::default().with_title("spas_render cube");
        event_loop.create_window(builder).expect("Could not build window")
    };

    let window_size = window.inner_size();

    // Create the Instance, Adapter, and Device. We can specify preferred backend,
    // device name, or rendering profile. In this case we let spas_render choose for us.
    let iad = pollster::block_on(spas_render::create_iad(None, None, None, None)).unwrap();

    // The one line of unsafe needed. We just need to guarentee that the window
    // outlives the use of the surface.
    //
    // SAFETY: this surface _must_ not be used after the `window` dies. Both the
    // event loop and the renderer are owned by the `run` closure passed to winit,
    // so rendering work will stop after the window dies.
    let surface = Arc::new(iad.instance.create_surface(&window).unwrap());
    // Get the preferred format for the surface.
    let caps = surface.get_capabilities(&iad.adapter);
    let preferred_format = caps.formats[0];

    // Configure the surface to be ready for rendering.
    spas_render::configure_surface(
        &surface,
        &iad.device,
        preferred_format,
        glam::UVec2::new(window_size.width, window_size.height),
        spas_render::types::PresentMode::Fifo,
    );

    // Make us a renderer.
    let renderer = spas_render::Renderer::new(
        iad,
        spas_render::types::Handedness::Left,
        Some(window_size.width as f32 / window_size.height as f32),
    )
    .unwrap();

    // Create the shader preprocessor with all the default shaders added.
    let mut spp = spas_render::ShaderPreProcessor::new();
    spas_render_routine::builtin_shaders(&mut spp);

    // Create the base rendergraph.
    let base_rendergraph = spas_render_routine::base::BaseRenderGraph::new(&renderer, &spp);

    let mut data_core = renderer.data_core.lock();
    let pbr_routine =
        spas_render_routine::pbr::PbrRoutine::new(&renderer, &mut data_core, &spp, &base_rendergraph.interfaces);
    drop(data_core);
    let tonemapping_routine = spas_render_routine::tonemapping::TonemappingRoutine::new(
        &renderer,
        &spp,
        &base_rendergraph.interfaces,
        preferred_format,
    );

    // Create mesh and calculate smooth normals based on vertices
    let mesh = create_mesh();

    // Add mesh to renderer's world.
    //
    // All handles are refcounted, so we only need to hang onto the handle until we
    // make an object.
    let mesh_handle = renderer.add_mesh(mesh).unwrap();

    // Add PBR material with all defaults except a single color.
    let material = spas_render_routine::pbr::PbrMaterial {
        albedo: spas_render_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
        ..spas_render_routine::pbr::PbrMaterial::default()
    };
    let material_handle = renderer.add_material(material);

    // Combine the mesh and the material with a location to give an object.
    let object = spas_render::types::Object {
        mesh_kind: spas_render::types::ObjectMeshKind::Static(mesh_handle),
        material: material_handle,
        transform: glam::Mat4::IDENTITY,
    };
    // Creating an object will hold onto both the mesh and the material
    // even if they are deleted.
    //
    // We need to keep the object handle alive.
    let _object_handle = renderer.add_object(object);

    let view_location = glam::Vec3::new(3.0, 3.0, -5.0);
    let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
    let view = view * glam::Mat4::from_translation(-view_location);

    // Set camera's location
    renderer.set_camera_data(spas_render::types::Camera {
        projection: spas_render::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
        view,
    });

    // Create a single directional light
    //
    // We need to keep the directional light handle alive.
    let _directional_handle = renderer.add_directional_light(spas_render::types::DirectionalLight {
        color: glam::Vec3::ONE,
        intensity: 10.0,
        // Direction will be normalized
        direction: glam::Vec3::new(-1.0, -4.0, 2.0),
        distance: 400.0,
        resolution: 2048,
    });

    let mut resolution = glam::UVec2::new(window_size.width, window_size.height);

    let _ = spas_render_framework::run_event_loop(event_loop, move |event, _event_loop_window_target| match event {
        // Window was resized, need to resize renderer.
        winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(physical_size), .. } => {
            resolution = glam::UVec2::new(physical_size.width, physical_size.height);
            // Reconfigure the surface for the new size.
            spas_render::configure_surface(
                &surface,
                &renderer.device,
                preferred_format,
                glam::UVec2::new(resolution.x, resolution.y),
                spas_render::types::PresentMode::Fifo,
            );
            // Tell the renderer about the new aspect ratio.
            renderer.set_aspect_ratio(resolution.x as f32 / resolution.y as f32);
        }
        // Render!
        winit::event::Event::WindowEvent { event: winit::event::WindowEvent::RedrawRequested, .. } => {
            // Get a frame
            let frame = match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(texture) => texture,
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                wgpu::CurrentSurfaceTexture::Outdated
                | wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded => return,
                wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Validation => {
                    panic!("Surface lost")
                }
            };

            // Swap the instruction buffers so that our frame's changes can be processed.
            renderer.swap_instruction_buffers();
            // Evaluate our frame's world-change instructions
            let mut eval_output = renderer.evaluate_instructions();

            // Build a rendergraph
            let mut graph = spas_render::graph::RenderGraph::new();

            // Import the surface texture into the render graph.
            let frame_handle =
                graph.add_imported_render_target(&frame, 0..1, 0..1, spas_render::graph::ViewportRect::from_size(resolution));
            // Add the default rendergraph without a skybox
            base_rendergraph.add_to_graph(
                &mut graph,
                spas_render_routine::base::BaseRenderGraphInputs {
                    eval_output: &eval_output,
                    routines: spas_render_routine::base::BaseRenderGraphRoutines {
                        pbr: &pbr_routine,
                        skybox: None,
                        tonemapping: &tonemapping_routine,
                    },
                    target: spas_render_routine::base::OutputRenderTarget {
                        handle: frame_handle,
                        resolution,
                        samples: spas_render::types::SampleCount::One,
                    },
                },
                spas_render_routine::base::BaseRenderGraphSettings {
                    ambient_color: glam::Vec4::ZERO,
                    clear_color: glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
                },
            );

            // Dispatch a render using the built up rendergraph!
            graph.execute(&renderer, &mut eval_output);

            // Present the frame
            renderer.queue.present(frame);
        }
        // Other events we don't care about
        _ => {}
    });
}
