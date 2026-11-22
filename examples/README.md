# spas_render Examples

The examples are separate crates, so to run an example run `cargo run <name>`.

If you want to demo running spas_render, [`scene-viewer`](scene-viewer) is what you should use. It is a full fledged gltf viewer.

If you want to look at how the basics of the code works, look at either the [`cube`](cube) or [`cube-no-framework`](cube-no-framework) example.

| Name                | Description | Image |
|:-------------------:|:------------|-------|
| scene-viewer        | Advanced gltf model and scene viewer. Used to test all of spas_render's complex features. | ![](src/scene_viewer/scifi-base.jpg) ![](src/scene_viewer/screenshot.jpg) ![](src/scene_viewer/bistro.jpg) ![](src/scene_viewer/emerald-square.jpg) |
| cube                | Basic example. Shows how to get started with the api, both with and without the use of spas_render-framework. | ![](src/cube/screenshot.png) |
| egui                | Shows integration with egui and spas_render-egui | ![](src/egui/screenshot.png) |
| skinning            | Basic bone deformation of a loaded gltf model | ![](src/skinning/screenshot.png) |
| static-gltf         | Similar to cube, but geometry is pulled from a simple gltf file | ![](src/static_gltf/screenshot.png) |
| textured-quad       | Basic 2D rendering with an orthographic camera | ![](src/textured_quad/screenshot.png) |
