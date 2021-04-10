<h1 align="center">
    Path Tracer
</h1>
<p align="center">
   <a href="https://github.com/flomonster/path-tracer/actions">
      <img src="https://github.com/flomonster/path-tracer/workflows/Build/badge.svg" alt="github">
   </a>
</p>
<hr>

This project is an implementation of Monte Carlo path tracing in **Rust**.

## How to use ?

The renderer takes as input a scene in **glTF** format.

Minimal command line:

```sh
path-tracer --help # Prompt all available options
path-tracer scene.gltf -o my-render.png # Render a scene with default profile
path-tracer -p profile.yml scene.gltf # Render a scene with a custom profile
```

## Profile

Profile files are used to configure the renderer behaviour. 

| Option | Description | Default |
|------------|----------------|-------------------|
| `resolution.width` | Width of the output image | `1920` |
| `resolution.height` | Height of the output image | `1080` |
| `samples` | Number of sample ray throw by pixel | `64` |
| `bounces` | Maximum number of bounces per sample | `4` |
| `brdf` | Which brdf tu use (`COOK_TORRANCE`) | `COOK_TORRANCE` |
| `tonemap` | Which color tone map tu use (`REINHARD`, `FILMIC`, `ACES`) | `FILMIC` |
| `background_color` | Color of the brackground (in RGB [`0.`:`1.`]) | `[0., 0., 0.]` |
| `nb_threads` | Number of threads to use (`0` will use the maximum available threads) | `0` |

Here is a profile example.

```yaml
resolution: # Resolution of the output image
  width: 1920
  height: 1080
samples: 64 # Number of sample ray throw by pixel
bounces: 4 # Maximum number of bounces per sample
brdf: COOK_TORRANCE # Which brdf to use
tonemap: FILMIC # Which color tone map to use
background_color: [0.051, 0.051, 0.051] # Color of the background
nb_threads: 8 # Number of threads to use (0 will use the maximum available threads)
```

## Features

- [x] Parallel computation
- [x] KD Tree
- [x] Unidirectional Monte Carlo path tracing
- [x] BRDF
- [x] Importance sampling
- [x] Various Tone mapping
- [x] Viewer
- [ ] BTDF
- [ ] BSSRDF
- [ ] ...
