use std::path::Path;

use spas_render_gltf::GltfSceneInstance;

const SAMPLE_COUNT: spas_render::types::SampleCount = spas_render::types::SampleCount::One;

#[derive(Default)]
pub struct SkinningExample {
    loaded_scene: Option<spas_render_gltf::LoadedGltfScene>,
    loaded_instance: Option<spas_render_gltf::GltfSceneInstance>,
    directional_light_handle: Option<spas_render::types::DirectionalLightHandle>,
    armature: Option<spas_render_gltf::Armature>,
    elapsed_time: f32,
}

/// Locates an object in the node list that corresponds to an animated mesh
/// and returns its list of skeletons. Note that a gltf object may contain
/// multiple primitives, and there will be one skeleton per primitive.
pub fn find_armature(instance: &GltfSceneInstance) -> Option<spas_render_gltf::Armature> {
    for node in &instance.nodes {
        if let Some(ref obj) = node.inner.object {
            if let Some(ref armature) = obj.inner.armature {
                return Some(armature.clone());
            }
        }
    }
    None
}

impl SkinningExample {
    /// This function gets called every frame. Updates the skeleton's joint
    /// positions
    pub fn update_skeleton(&mut self, renderer: &spas_render::Renderer) {
        let armature = &self.armature.as_ref().expect("Data must be loaded by now");
        let loaded_scene = &self.loaded_scene.as_ref().expect("Data must be loaded by now");
        let inverse_bind_matrices = &loaded_scene.skins[armature.skin_index].inner.inverse_bind_matrices;

        // Compute a very simple animation for the top bone
        let t = self.elapsed_time;
        let rotation_degrees = 30.0 * f32::sin(5.0 * t);

        // An armature contains multiple skeletons, one per mesh primitive being
        // deformed. We need to set the joint matrices per each skeleton.
        for skeleton in &armature.skeletons {
            renderer.set_skeleton_joint_transforms(
                skeleton,
                &[
                    glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -4.18)),
                    glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0))
                        * glam::Mat4::from_rotation_x(rotation_degrees.to_radians()),
                ],
                inverse_bind_matrices,
            );
        }
    }
}

impl spas_render_framework::App for SkinningExample {
    const HANDEDNESS: spas_render::types::Handedness = spas_render::types::Handedness::Left;

    fn sample_count(&self) -> spas_render::types::SampleCount {
        SAMPLE_COUNT
    }

    fn setup(&mut self, context: spas_render_framework::SetupContext<'_>) {
        let view_location = glam::Vec3::new(0.0, 0.0, -10.0);
        let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0);
        let view = view * glam::Mat4::from_translation(-view_location);

        // Set camera's location
        context.renderer.set_camera_data(spas_render::types::Camera {
            projection: spas_render::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view,
        });

        // Load a gltf model with animation data
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/skinning/RiggedSimple.glb"));
        let gltf_data = std::fs::read(path).unwrap();
        let parent_directory = path.parent().unwrap();
        let (loaded_scene, loaded_instance) = pollster::block_on(spas_render_gltf::load_gltf(
            context.renderer,
            &gltf_data,
            &spas_render_gltf::GltfLoadSettings::default(),
            |p| async move { spas_render_gltf::filesystem_io_func(&parent_directory, &p).await },
        ))
        .expect("Loading gltf scene");

        // The returned loaded model contains a node hierarchy with a complete
        // scene. We know in our case there will be a single node in the tree
        // with an armature.
        self.armature = Some(find_armature(&loaded_instance).unwrap());

        // Store the loaded model somewhere, otherwise all the data gets freed.
        self.loaded_scene = Some(loaded_scene);
        self.loaded_instance = Some(loaded_instance);

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        self.directional_light_handle = Some(context.renderer.add_directional_light(spas_render::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 10.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
            resolution: 2048,
        }));
    }

    fn handle_redraw(&mut self, context: spas_render_framework::RedrawContext<'_, ()>) {
        self.update_skeleton(context.renderer);

        // Swap the instruction buffers so that our frame's changes can be processed.
        context.renderer.swap_instruction_buffers();
        // Evaluate our frame's world-change instructions
        let mut eval_output = context.renderer.evaluate_instructions();

        // Lock the routines
        let pbr_routine = spas_render_framework::lock(&context.routines.pbr);
        let tonemapping_routine = spas_render_framework::lock(&context.routines.tonemapping);

        // Build a rendergraph
        let mut graph = spas_render::graph::RenderGraph::new();

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

        // Dispatch a render using the built up rendergraph!
        graph.execute(context.renderer, &mut eval_output);
    }
}

pub fn main() {
    let app = SkinningExample::default();
    spas_render_framework::start(
        app,
        winit::window::WindowAttributes::default().with_title("skinning-example").with_maximized(true),
    );
}

#[cfg(test)]
#[spas_render_test::test_attr]
async fn test() {
    crate::tests::test_app(crate::tests::TestConfiguration {
        app: SkinningExample::default(),
        reference_path: "src/skinning/screenshot.png",
        size: glam::UVec2::new(1280, 720),
        threshold_set: spas_render_test::Threshold::Mean(0.01).into(),
    })
    .await
    .unwrap();
}
