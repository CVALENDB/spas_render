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

const SAMPLE_COUNT: spas_render::types::SampleCount = spas_render::types::SampleCount::One;

#[derive(Default)]
pub struct CubeExample {
    object_handle: Option<spas_render::types::ObjectHandle>,
    directional_light_handle: Option<spas_render::types::DirectionalLightHandle>,
    point_lights: Vec<spas_render::types::PointLightHandle>,
}

impl spas_render_framework::App for CubeExample {
    const HANDEDNESS: spas_render::types::Handedness = spas_render::types::Handedness::Left;

    fn sample_count(&self) -> spas_render::types::SampleCount {
        SAMPLE_COUNT
    }

    fn setup(&mut self, context: spas_render_framework::SetupContext<'_>) {
        // Create mesh and calculate smooth normals based on vertices
        let mesh = create_mesh();

        // Add mesh to renderer's world.
        //
        // All handles are refcounted, so we only need to hang onto the handle until we
        // make an object.
        let mesh_handle = context.renderer.add_mesh(mesh).unwrap();

        // Add PBR material with all defaults except a single color.
        let material = spas_render_routine::pbr::PbrMaterial {
            albedo: spas_render_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.5, 0.5, 0.5, 1.0)),
            ..spas_render_routine::pbr::PbrMaterial::default()
        };
        let material_handle = context.renderer.add_material(material);

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
        self.object_handle = Some(context.renderer.add_object(object));

        let view_location = glam::Vec3::new(3.0, 3.0, -5.0);
        let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
        let view = view * glam::Mat4::from_translation(-view_location);

        // Set camera's location
        context.renderer.set_camera_data(spas_render::types::Camera {
            projection: spas_render::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view,
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        self.directional_light_handle = Some(context.renderer.add_directional_light(spas_render::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 1.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
            resolution: 2048,
        }));

        let lights = [
            // position, color
            (glam::vec3(0.1, 1.2, -1.5), glam::vec3(1.0, 0.0, 0.0)),
            (glam::vec3(1.5, 1.2, -0.1), glam::vec3(0.0, 1.0, 0.0)),
        ];

        for (position, color) in lights {
            self.point_lights.push(context.renderer.add_point_light(spas_render::types::PointLight {
                position,
                color,
                radius: 2.0,
                intensity: 4.0,
            }));
        }
    }

    fn handle_redraw(&mut self, context: spas_render_framework::RedrawContext<'_, ()>) {
        // Swap the instruction buffers so that our frame's changes can be processed.
        context.renderer.swap_instruction_buffers();
        // Evaluate our frame's world-change instructions
        let mut eval_output = context.renderer.evaluate_instructions();

        // Lock the routines
        let pbr_routine = spas_render_framework::lock(&context.routines.pbr);
        let tonemapping_routine = spas_render_framework::lock(&context.routines.tonemapping);

        // Build a rendergraph
        let mut graph = spas_render::graph::RenderGraph::new();

        // Import the surface texture into the render graph.
        let frame_handle = graph.add_imported_render_target(
            context.surface_texture,
            0..1,
            0..1,
            spas_render::graph::ViewportRect::from_size(context.resolution),
        );
        // Add the default rendergraph without a skybox
        context.base_rendergraph.add_to_graph(
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
                    resolution: context.resolution,
                    samples: SAMPLE_COUNT,
                },
            },
            spas_render_routine::base::BaseRenderGraphSettings {
                ambient_color: glam::Vec4::ZERO,
                clear_color: glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
            },
        );

        // Dispatch a render using the built-up rendergraph!
        graph.execute(context.renderer, &mut eval_output);
    }
}

pub fn main() {
    let app = CubeExample::default();
    spas_render_framework::start(
        app,
        winit::window::WindowAttributes::default().with_title("cube-example").with_maximized(true),
    );
}

#[cfg(test)]
#[spas_render_test::test_attr]
async fn test() {
    crate::tests::test_app(crate::tests::TestConfiguration {
        app: CubeExample::default(),
        reference_path: "src/cube/screenshot.png",
        size: glam::UVec2::new(1280, 720),
        threshold_set: spas_render_test::Threshold::Mean(0.01).into(),
    })
    .await
    .unwrap();
}
