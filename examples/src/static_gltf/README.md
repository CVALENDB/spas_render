# static-gltf

Quite similar to the cube example, but the geometry to render is pulled from `data.glb` using the gltf crate.

Note that only a small small portion of the gltf spec is used here; you could pull out and render a lot more data with spas_render.
Materials in particular are largely ignored.

If you want a full fledged gltf viewer, look at [scene-viewer](../scene-viewer).

![](screenshot.png)
