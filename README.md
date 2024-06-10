# ImGui-rs
Raw bindings for the Dear ImGui library for Rust.

# Installing
Simply clone this repo and include it in your project.
```bash
$ git clone --recurse-submodules https://github.com/nepcat/imgui_rs
```
### Cargo.toml
```toml
...

[dependencies]
imgui_rs = { path = "path/to/imgui_rs/", features = ... }
```

# Dependencies
This crate depends on **pkg-config**, **clang** (for bindgen).

# Rust toolchain
Nightly only

# Supported platforms
Both Windows (Only GNU, MSVC is [broken](https://github.com/nepcat/imgui_rs/issues/1)) and Linux should work as long as you meet all dependency criteria.

# Available features
* **freetype** - Freetype support
* **docking** - ImGui docking branch
* **sdl2** - Platform Backend for SDL2
* **sdl2_renderer** - Renderer Backend for SDL_Renderer for SDL2 (Will enable **sdl2** feature)
* **sdl3** - Platform Backend for SDL3
* **sdl3_renderer** - Renderer Backend for SDL3 (Will enable **sdl3** feature)
* **gl2** - Renderer Backend for OpenGL2 (legacy OpenGL, fixed pipeline)
* **gl3** - Renderer Backend for modern OpenGL with shaders / programmatic pipeline
* **vulkan** - Renderer Backend for Vulkan
* **win32** - Platform Backend for Windows
* **dx9** - Renderer Backend for DirectX9
* **dx10** - Renderer Backend for DirectX10
* **dx11** - Renderer Backend for DirectX11
* **dx12** - Renderer Backend for DirectX12

# Example usage
```rust
#![allow(clippy::never_loop)]
use imgui_rs::root as imgui_rs;

pub fn main() {
    /* Initialize SDL2, OpenGL3, ... */
    let sdl_window = core::ptr::null_mut();
    let sdl_gl_context = core::ptr::null_mut();

    unsafe {
        let imgui_context = imgui_rs::ImGui::CreateContext(core::ptr::null_mut());
        if imgui_context.is_null() {
            panic!("Bad imgui context");
        }

        if !imgui_rs::ImGui_ImplSDL2_InitForOpenGL(sdl_window as _, sdl_gl_context as _) {
            panic!("ImGui_ImplSDL2_InitForOpenGL() failed!");
        }
        if !imgui_rs::ImGui_ImplOpenGL3_Init(core::ptr::null_mut()) {
            panic!("ImGui_ImplOpenGL3_Init() failed!");
        }

        loop {
            /* do your loop here... */
            break;
        }

        imgui_rs::ImGui_ImplOpenGL3_Shutdown();
        imgui_rs::ImGui_ImplSDL2_Shutdown();

        imgui_rs::ImGui::DestroyContext(imgui_context);
    }

    /* Destroy SDL2, OpenGL3, ... */
}
```