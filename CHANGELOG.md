# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to cargo's version of [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per Keep a Changelog there are 6 main categories of changes:

- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security

## Versions

- [Unreleased](#unreleased)
- [v0.3.0](#v030)
- [v0.2.0](#v020)
- [v0.1.1](#v011)
- [v0.1.0](#v010)
- [v0.0.6](#v006)
- [v0.0.5](#v005)
- [v0.0.4](#v004)
- [v0.0.3](#v003)
- [v0.0.2](#v002)
- [v0.0.1](#v001)
- [Diffs](#diffs)

## Unreleased

### Major Changes
- spas_render: `add_mesh`, `add_skeleton` and `add_texture_*` now return Results with fully typed errors. This will catch all errors on all platforms except for web, where wgpu allocation errors will not be caught. @cwfitzgerald
- spas_render-routine: Argument structs broken up into multiple sub-structs for better ergonomics. @cwfitzgerald

### Added
- spas_render-egui: Added the ability to create egui textures (egui::TextureId) with the wgpu backend @AlbinSjoegren
  - Added function for creating egui textures (egui::TextureId) @AlbinSjoegren
  - Added function for writing previous wgpu textures with new rgba data and exporting to egui::TextureId @AlbinSjoegren
- spas_render-routine: Added the option to set a custom primitive topology value when building a forward routine. @setzer22
- spas_render-routine: Added a resolution field to the per-frame uniforms. @setzer22
- spas_render-routine: Added add_clear_to_graph to make clears explicit and add `clear_color` argument to base rendergraph.
- spas_render: Added basic (no shadow maps, no clustering) point light support to the renderer API. @marceline-cramer

### Changes
- spas_render: Update to wgpu 0.19, naga 0.14 @garyttierney @kpreid
- spas_render: Convert all shaders to WGSL using a custom preprocessing solution @cwfitzgerald
- spas_render: Update to winit 0.29.4 @pillowtrucker
- spas_render-framework: Consolidate many arguments into single `SetupContext`, `EventContext`, and `RedrawContext` structs. @cwfitzgerald
- spas_render-framework: Surfaces are now handled amost entirely by the framework, including acquiring frames and presenting. Redraws now happen in a dedicated `handle_redraw` callback. @cwfitzgerald
- spas_render-egui: Update to egui 0.26. @Elabajaba

### Fixes
- Fixed renderpass compatibility checks to avoid issues when RODS is used. @OptimisticPeach
- Fixed mismatched BGLs when using a custom material with no cutout specification
- Fixed PbrMaterial instead of generic parameter M being used in forward and depth routines. @setzer22
- Fixes loading of gltf with embedded base64 binary data.
- Fixed building with `profiling/profile-with-tracing`. @SparkyPotato
- Fixed panic when a mesh object with a skeleton was despawned. @setzer22
- Fixed forward pass not writing to depth when no depth prepass was used. @IsseW
- Fixed panic in IAD creation when system has more than 4 wgpu adapters. @marceline-cramer
- Fixed multiple issues with mesh management. @cwfitzgerald
- Fixed hang when adding a large amount of meshes in a single frame on MacOS. @cwfitzgerald

## v0.3.0

Released 2022-02-11

This is by far the largest release of spas_render since 0.1. I have done my best to document everything that has changed.

### Major Changes
- spas_render-pbr got renamed to spas_render-routine and will host all render routines, not just PBR related ones.

### Added
- spas_render-egui: An integration with the immediate mode GUI [egui](https://github.com/emilk/egui) @MindSwipe
- spas_render-textured-quad: Add example of simple 2D rendering.
- spas_render: Allow duplicating objects overriding some of their properties. @setzer22
- spas_render: Implement mesh skinning. @setzer22
- spas_render: Added `CameraProjection::Raw`.
- spas_render: The renderer now has a `handedness` value that determines the handedness of its coordinate system.
  This allows the renderer to deal with both DX-convention coordinate systems and OpenGL convention coordinate systems.
- spas_render-gltf: Load gltf animation data @setzer22

### Changes
- spas_render: Instead of passing a render routine to the render function, 
  you now add them to a rendergraph, then pass that rendergraph into the renderer.
  - spas_render-routine:
    - The old `PbrRoutine` has been replaced with the more vercitile `BaseRenderGraph`.
    - This base rendergraph will put all the parts together through it's `add_to_graph` function.
    - Split the PbrRoutine into two parts `add_prepass_to_graph` and `add_forward_to_graph`.
    - Split out the skybox renderer into `SkyboxRoutine`.
    - Split out tonemapping into the `TonemappingRoutine`.
- spas_render: Renamed the old `RendererMode` to `RendererProfile` and adjusted verbiage to refer to them as profiles.
  - "CpuPowered" => "CpuDriven".
  - "GpuPowered" => "GpuDriven".
- spas_render: All meshes now require validation.
  - `MeshBuilder::new()` now takes a handedness of the mesh. If it doesn't match the handedness of the renderer, it flips the winding order.
  - `MeshBuilder::build()` now returns a `Result<Mesh, MeshValidationError>`. This validation can be unsafely omitted.
  - The implementation functions `Mesh::calculate_normals{,_for_buffers}` and `Mesh::calculate_tangents{,_for_buffers}` are now unsafe.
- spas_render: Renamed `CameraProjection::Projection` to `CameraProjection::Perspective`.
- spas_render: Allow requesting device features explicitly in `spas_render::create_iad`. @setzer22
- spas_render: Objects now take a `mesk_kind` allowind definition of static or animated meshes. @setzer22
- spas_render-routine: All transparency now has backface culling enabled. Use `Mesh::double_side` or `MeshBuilder::with_double_side` to re-enable double sided transparency.
- spas_render-gltf: Split return value of `load_gltf` into per-scene data and per-instance data. @setzer22
- spas_render-gltf: The `nodes` vector is now flat instead of nested. Hierarchy is represented using indices. @setzer22

### Fixes
- spas_render: Fixed objects becoming invalid after a vertex buffer resize occured.
- spas_render: Get vertex/index counts from RangeAllocator. @jamen
- spas_render: Fix compatibility comparison for RenderPassTargets. @setzer22

## v0.2.0

Released 2021-10-09

This release saw a signifigant amount of cpu side optimization.
In a 50k object scene, the render loop went from taking 16ms to taking 1.75ms, a 9x speedup

### Added
- spas_render: Added an explicit dependency on wgpu-core and wgpu-hal to ensure bug-fixes are included.
- spas_render-gltf: Add the ability to turn off image's default features.
- spas_render-gltf: Add support for ktx2 and dds images.
- spas_render-gltf: Expose implementation functions
- spas_render-pbr-bake: Added crate for automatic light baking to a texture.

### Changed
- spas_render: IMPORTANT: You now must call output.present() in order for things to show up on screen.
- spas_render: `Material` is now a trait and render routines can specify their own material.
  - `spas_render::types::Material` is now `spas_render_pbr::material::PbrMaterial`
  - `Renderer::update_material` no longer takes a `MaterialChange`, it takes a completely new material.
- spas_render: renderlists refactored to have a generic Input and Output.
  - `Renderer::renderer` passes through the Input and Output.
  - OutputFrame is now a user-side only utility.
- spas_render: AddTexture* will now create/upload the texture before the call returns.
- spas_render: ResourceHandle<T> now prints the actual reference count while debug printing.
- spas_render: `CameraProjection` now deals with view matrix, instead of location/yaw/pitch @scoopr
- spas_render: Update to glam 0.19 @scoopr
- spas_render & spas_render-gltf: update to wgpu 0.11
- spas_render-pbr: Input is of type `()` and Output is type `&TextureView`.
- spas_render-gltf: Added labels to all the different data types.
- spas_render-gltf: Errors now use a `SsoString` instead of a `String` for the URI.
- spas_render-gltf: All implementation functions no longer write into a single `&mut LoadedGltfScene`, but return their results directly.

### Fixed
- spas_render: No longer require pipeline rebuilds when bind group length updates.
- spas_render-pbr: fix rendering of cutout objects in shadow passes.
- spas_render-pbr: remove redundant material changes in cpu mode.

## v0.1.1

Released 2021-09-13

### Added
- spas_render-pbr: `set_ambient_lighting` sets the ambient light value, making sure no lighting result is less than `ambient * color`.

### Fixed
- spas_render: properly exported `ExtendedAdapterInfo`

## v0.1.0

Released 2021-09-11

### Added
- spas_render: Materials now have a `Transparency` field that supports Opaque, Cutout, and Blend transparency modes.
- spas_render: `AlbedoComponent::TextureVertexValue` to represent `texture * vertex * constant`
- spas_render: Mipmaps can be generated automatically on the gpu without the user needing to upload them.
- spas_render: `Renderer::add_texture_2d_from_texture` which allows you to make a new texture from a set another texture's mipmaps.
- spas_render & spas_render-pbr: Use `wgpu-profiler` to generate GPU timings that show up as `RendererStatistics`.
- spas_render & spas_render-pbr: Annotate most code with `profiling` spans.
- spas_render & spas_render-pbr: Add a `distance` field that signifies how much space shadows should take up.
- spas_render-pbr: All major rendering spans are labeled and show up in renderdoc
- spas_render-pbr: Multisampling support
- spas_render-pbr: Support for transparency as well as stable gpu-culling to preserve transparency sort order.

### Changed
- spas_render: **SUBTLE** All handles are now refcounted.
  - Handles are now `!Copy`. Functions taking handles now accept a reference to a handle.
  - If you want to keep something alive, you need to keep the handle to it alive.
  - `Object`s will keep `Material`s/`Mesh`s alive.
  - `Material`s will keep `Texture`s alive.
  - All resources are removed the `render()` after they are deleted.
- spas_render: Externalize all surfaces, adapters, devices, etc.
  - Instead of using a `RendererBuilder`, construct an Instance/Adapter/Device with `rend::create_iad` and pass that to `Renderer::new`.
  - Surfaces are now controlled by the user. There is a convinence function `spas_render::configure_surface` to make this smoother.
- spas_render: `Texture::width` and `Texture::height` replaced with `Texture::size`
- spas_render: `RendererStatistics` is now an alias for `Vec<wgpu_profiler::GpuTimerScopeResult>`
- spas_render: `Texture::mip_levels` was split into `mip_count` and `mip_source` allowing you to easily auto-generate mipmaps.
- spas_render: Changed limits such that intel gets CPU mode until [wgpu#1111](https://github.com/gfx-rs/wgpu/issues/1111) gets resolved.
- spas_render-pbr: creation and resizing's `resolution` argument replaced with options containing resolution and sample count.

### Updated
- Dependencies:
  - `glam` 0.17 -> 0.18

### Fixed
- spas_render-pbr: Shadow artifacting due to incorrect face culling when rendering shadow passes
- spas_render-pbr: CPU mode drawing failed to account for proper vertex offsets
- spas_render-pbr: Non-normalized normal maps now work correctly.
- spas_render-pbr: Growing the GPU mode texture descriptor list no longer causes panic
- spas_render-gltf: albedo-texture UV transform is now respected
- spas_render-gltf: image loading now properly caches images

### Removed
- spas_render: `RendererBuilder` replaced with explicit calls to `Renderer::new`.
- spas_render: `Renderer::delete_*` functions were removed in favor of refcounting.

## v0.0.6

Released 2021-08-22

### Added
- `spas_render_types` crate with all datatypes.

### Changed
- `spas_render::datatypes` is now renamed to `spas_render::types`. It is a reexport of `spas_render_types`.
- `spas_render::types::TextureFormat` is a reexport of `wgpu_types::TextureFormat`.
- Replaced Renderlists with Render Routines
  - `spas_render_list` crate is now `spas_render_pbr`.
  - `Swapchain` mentions are now `Surface`.
  - `set_options` is now `set_internal_surface_options`
  - The following are now functions of the render routine:
    - `resize` is on both.
    - `set_background_texture` now takes an `Option<TextureHandle>` and there is no `clear_background_texture`.
- `log` is now used for logging as opposed to `tracing`, so `env_logger` should be used over `wgpu_subscriber`.

### Updated
- `wgpu` 0.7 -> 0.10
- `glam` 0.13 -> 0.17

### Removed
- `span` and `span_transfer`, due to `tracing`'s removal.
- All ties to `switchyard`.
- Shader compiling infrastructure is gone, shaders must be wgsl or pre-compiled to spirv.

## v0.0.5

Released 2021-03-10

### Fixed
- Fixed silly math error when converting to `glam` to `v0.13.0`.

## v0.0.4

Yanked 2021-03-10

Released 2021-03-08

#### Updated
- `glam` to `v0.13.0`

## v0.0.3

Released 2021-03-06

#### Added
- Internal: use cargo-release for all releases

## v0.0.2

Released 2021-03-06

#### Changes
- Update documentation.

## v0.0.1

Released 2021-03-06

#### Added
- First release of `spas_render`.

## Diffs

- [Unreleased](https://github.com/CVALENDB/spas_render/compare/v0.3.0...HEAD)
- [v0.3.0](https://github.com/CVALENDB/spas_render/compare/v0.2.0...v0.3.0)
- [v0.2.0](https://github.com/CVALENDB/spas_render/compare/v0.1.1...v0.2.0)
- [v0.1.1](https://github.com/CVALENDB/spas_render/compare/v0.1.0...v0.1.1)
- [v0.1.0](https://github.com/CVALENDB/spas_render/compare/v0.0.6...v0.1.0)
- [v0.0.6](https://github.com/CVALENDB/spas_render/compare/v0.0.5...v0.0.6)
- [v0.0.5](https://github.com/CVALENDB/spas_render/compare/v0.0.4...v0.0.5)
- [v0.0.4](https://github.com/CVALENDB/spas_render/compare/v0.0.3...v0.0.4)
- [v0.0.3](https://github.com/CVALENDB/spas_render/compare/v0.0.2...v0.0.3)
- [v0.0.2](https://github.com/CVALENDB/spas_render/compare/v0.0.1...v0.0.2)
