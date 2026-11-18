# Testing native OpenGL on macOS

This branch makes Ruffle's desktop app able to render through OpenGL on macOS. It uses a patched wgpu (`SuchAFuriousDeath/wgpu`, branch `macos-cgl`) that adds a native CGL backend. The goal is to support Macs without a Metal-capable GPU. Any current Mac can test it, because macOS still ships OpenGL 4.1.

None of this has run on a Mac before, so problems are expected. Logs and screenshots of whatever happens are the useful part.

## Setup

1. Install the Xcode command line tools: `xcode-select --install`
2. Install Rust from https://rustup.rs (the default options are fine).
3. Clone both branches:

   ```sh
   git clone -b macos-cgl https://github.com/SuchAFuriousDeath/wgpu.git
   git clone -b macos-gl-experiment https://github.com/SuchAFuriousDeath/ruffle.git
   ```

## 1. wgpu on its own

In the `wgpu` folder:

```sh
RUST_LOG=wgpu_hal=debug cargo run -p wgpu-info --features wgpu/cgl
```

The output should list an adapter with backend `Gl` and an OpenGL version of 4.1 or 3.3.

Then run a few examples through OpenGL:

```sh
WGPU_BACKEND=gl RUST_LOG=wgpu_hal=debug cargo run -p wgpu-examples --features wgpu/cgl -- hello_triangle
```

Repeat with `cube`, `msaa_line`, `texture_arrays`, `srgb_blend` and `hello_windows` instead of `hello_triangle`.

For each one, please check:
- **First frame:** is something drawn as soon as the window appears, without resizing it?
- **Resize:** does the picture follow when you resize the window?
- **Other display:** if there is a second monitor, does it still work after dragging the window there?
- **Fullscreen:** does it still draw?
- **Minimised:** when minimised, does the CPU usage of the process stay low?
- **Colours:** does `srgb_blend` look right, and not too dark?

## 2. Ruffle

In the `ruffle` folder:

```sh
RUST_LOG=info,wgpu_hal=debug cargo run --release -p ruffle_desktop -- -g gl --volume 0 path/to/some.swf
```

`-g gl` forces OpenGL. Without it, a Mac with Metal uses Metal as usual. Try a few SWFs if you can: games, animations, and anything with filters or blur.

The log should say it is using the OpenGL backend. Please check the same things as above, plus:
- **Startup without a SWF:** does the start screen appear when Ruffle is run without one?
- **Rendering:** does the content look the same as with Metal (run without `-g gl` to compare)?
- **Preferences dialog:** does it open and work?

## What to send back

- **Terminal output:** the full output of each command, especially any panic or repeating warnings.
- **Visuals:** screenshots of anything that looks wrong: blank or black windows, wrong colours, flicker.
- **About This Mac:** the model, macOS version and graphics.
