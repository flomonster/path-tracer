<h1 align="center">
    Path Tracer
</h1>
<p align="center">
   <a href="https://github.com/flomonster/path-tracer/actions">
      <img src="https://github.com/flomonster/path-tracer/workflows/Build/badge.svg" alt="github">
   </a>
</p>
<hr>

This project is an implementation of path tracer in **Rust**.

## How to use ?

The renderer takes as input a scenes in **Gltf** format.

Minimal command line:

```sh
path-tracer --help # Prompt all available options
path-tracer scene.gltf -o my-render.png # Render a scene with default profile
path-tracer -p profile.yml scene.gltf # Render a scene with a custom profile
```

Profile files are used to configure the renderer behaviour. Here is a profile
example:

```yaml
resolution: # Resolution of the output image
  width: 800
  height: 800
samples: 8 # Number of sample ray throw by pixel
bounces: 4 # Maximum number of bounces per sample
brdf: COOK_TORRANCE # Which brdf to use
tonemap: FILMIC # Which color tone map to use
```

## Features

- [x] Camera transform
- [x] Parallel computation
- [x] Multiple Tone mapping
- [x] Importance sampling
- [x] BRDF
- [ ] BTDF
- [ ] Bloom
- [ ] ...
