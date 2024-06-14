#![allow(clippy::vec_init_then_push)]

fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    #[cfg(debug_assertions)]
    const LOG_FILTER: &str = "build_script_build=debug";
    #[cfg(not(debug_assertions))]
    const LOG_FILTER: &str = "build_script_build=info";

    /* Initialize logger */
    env_logger::try_init_from_env(env_logger::Env::new().default_filter_or(LOG_FILTER))
        .context("Failed to initialize env_logger")?;
    #[cfg(not(debug_assertions))]
    log::info!("Using info LOG_FILTER. To see debug messages either define your own env LOG_FILTER or compile as debug");

    if let Err(error) = try_main() {
        log::error!("Error: {error:?}");
        anyhow::bail!("try_main() failed")
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum CompileFileType {
    Header { should_bindgen: bool },
    Source,
}

#[derive(Debug, Clone)]
pub struct CompileFile {
    pub r#type: CompileFileType,
}

fn try_main() -> anyhow::Result<()> {
    use anyhow::Context;

    let manifest_directory = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").context("Failed to get CARGO_MANIFEST_DIR")?,
    );
    log::debug!("manifest_directory {}", manifest_directory.display());

    let out_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").context("Failed to get OUT_DIR")?);
    log::debug!("out_path {}", out_path.display());

    let target_arch = build_target::target_arch().context("Failed to get target architecture")?;
    log::debug!("target_arch {target_arch}");
    let target_os = build_target::target_os().context("Failed to get target OS")?;
    log::debug!("target_os {target_os}");

    #[cfg(feature = "docking")]
    let imgui_path = manifest_directory.join("imgui_docking");
    #[cfg(not(feature = "docking"))]
    let imgui_path = manifest_directory.join("imgui");
    log::debug!("imgui_path {}", imgui_path.display());

    let imgui_backends = imgui_path.join("backends");
    log::debug!("imgui_backends {}", imgui_backends.display());
    let imgui_misc = imgui_path.join("misc");
    log::debug!("imgui_misc {}", imgui_misc.display());
    #[cfg(feature = "freetype")]
    let (imgui_freetype, freetype) = {
        let imgui_freetype = imgui_misc.join("freetype");
        log::debug!("imgui_freetype {}", imgui_freetype.display());
        let freetype =
            pkg_config::probe_library("freetype2").context("Failed to probe freetype2 library!")?;
        (imgui_freetype, freetype)
    };
    #[cfg(feature = "sdl2")]
    let sdl2 = pkg_config::probe_library("sdl2").context("Failed to probe sdl2 library!")?;
    #[cfg(feature = "sdl3")]
    let sdl3 = pkg_config::probe_library("sdl3").context("Failed to probe sdl3 library!")?;

    /* All used imgui files */
    let mut imgui_files: Vec<(&std::path::PathBuf, &CompileFile)> = Vec::new();

    /* Core imgui files */
    let mut imgui_core_files: Vec<(std::path::PathBuf, CompileFile)> = Vec::new();
    imgui_core_files.push((
        imgui_path.join("imgui.cpp"),
        CompileFile {
            r#type: CompileFileType::Source,
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui.h"),
        CompileFile {
            r#type: CompileFileType::Header {
                should_bindgen: true,
            },
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui_internal.h"),
        CompileFile {
            r#type: CompileFileType::Header {
                should_bindgen: true,
            },
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui_widgets.cpp"),
        CompileFile {
            r#type: CompileFileType::Source,
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui_draw.cpp"),
        CompileFile {
            r#type: CompileFileType::Source,
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui_tables.cpp"),
        CompileFile {
            r#type: CompileFileType::Source,
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imgui_demo.cpp"),
        CompileFile {
            r#type: CompileFileType::Source,
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imstb_textedit.h"),
        CompileFile {
            r#type: CompileFileType::Header {
                should_bindgen: false,
            },
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imstb_rectpack.h"),
        CompileFile {
            r#type: CompileFileType::Header {
                should_bindgen: false,
            },
        },
    ));
    imgui_core_files.push((
        imgui_path.join("imstb_truetype.h"),
        CompileFile {
            r#type: CompileFileType::Header {
                should_bindgen: false,
            },
        },
    ));

    for (file_path, file) in &imgui_core_files {
        log::debug!("Core file {}", file_path.display());
        println!("cargo:rerun-if-changed={}", file_path.display());
        imgui_files.push((file_path, file));
    }

    /* Backends */
    let mut imgui_backend_files: Vec<(std::path::PathBuf, CompileFile)> = Vec::new();
    #[cfg(feature = "win32")]
    if target_os == build_target::Os::Windows {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_win32.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_win32.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
        imgui_backend_files.push((
            manifest_directory.join("imgui_impl_win32_wrapper.hpp"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    } else {
        log::info!("Target OS is not windows, skipping win32 feature!");
    }
    #[cfg(feature = "dx9")]
    if target_os == build_target::Os::Windows {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx9.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx9.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    } else {
        log::info!("Target OS is not windows, skipping dx9 feature!");
    }
    #[cfg(feature = "dx10")]
    if target_os == build_target::Os::Windows {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx10.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx10.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    } else {
        log::info!("Target OS is not windows, skipping dx10 feature!");
    }
    #[cfg(feature = "dx11")]
    if target_os == build_target::Os::Windows {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx11.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx11.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    } else {
        log::info!("Target OS is not windows, skipping dx11 feature!");
    }
    #[cfg(feature = "dx12")]
    if target_os == build_target::Os::Windows {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx12.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_dx12.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    } else {
        log::info!("Target OS is not windows, skipping dx12 feature!");
    }
    #[cfg(feature = "sdl2")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdl2.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdl2.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    #[cfg(feature = "sdl2_renderer")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdlrenderer2.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdlrenderer2.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    #[cfg(feature = "sdl3")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdl3.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdl3.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    #[cfg(feature = "sdl3_renderer")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdlrenderer3.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_sdlrenderer3.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    #[cfg(feature = "gl2")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_opengl2.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_opengl2.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    #[cfg(feature = "gl3")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_opengl3.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_opengl3.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_opengl3_loader.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: false,
                },
            },
        ));
    }
    #[cfg(feature = "vulkan")]
    {
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_vulkan.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_backend_files.push((
            imgui_backends.join("imgui_impl_vulkan.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: true,
                },
            },
        ));
    }
    for (file_path, file) in &imgui_backend_files {
        log::debug!("Backend file {}", file_path.display());
        println!("cargo:rerun-if-changed={}", file_path.display());
        imgui_files.push((file_path, file));
    }

    /* Other */
    let mut imgui_other_files: Vec<(std::path::PathBuf, CompileFile)> = Vec::new();
    #[cfg(feature = "freetype")]
    {
        imgui_other_files.push((
            imgui_freetype.join("imgui_freetype.cpp"),
            CompileFile {
                r#type: CompileFileType::Source,
            },
        ));
        imgui_other_files.push((
            imgui_freetype.join("imgui_freetype.h"),
            CompileFile {
                r#type: CompileFileType::Header {
                    should_bindgen: false,
                },
            },
        ));
    }
    for (file_path, file) in &imgui_other_files {
        log::debug!("Other file {}", file_path.display());
        println!("cargo:rerun-if-changed={}", file_path.display());
        imgui_files.push((file_path, file));
    }

    /* Bindgen */
    let mut builder = bindgen::Builder::default();
    builder = builder
        .clang_args(["-I", &imgui_path.to_string_lossy()])
        .clang_args(["-x", "c++"])
        .clang_arg("-std=c++20")
        .clang_args(["-D", "IMGUI_DISABLE_SSE"]); // that is only for inline functions

    /* Add bindgen header files */
    for (file_path, file) in &imgui_files {
        if let CompileFileType::Header { should_bindgen } = file.r#type {
            if should_bindgen {
                log::debug!("Bindgen file {}", file_path.display());
                let file_path_lossy = file_path.to_string_lossy();
                builder = builder
                    .header(file_path_lossy.clone())
                    .allowlist_file(file_path_lossy);
            } else {
                continue;
            }
        }
    }

    builder = builder
        /* C++ Namespaces */
        .enable_cxx_namespaces()
        /* Enum */
        .prepend_enum_name(false)
        .bitfield_enum(".*Flags_")
        .newtype_enum(".*")
        /* Rust target */
        .rust_target(bindgen::RustTarget::Nightly);
    let bindings = builder.generate().context("Failed to generate bindings")?;
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .context("Couldn't write bindings")?;
    log::debug!("Bindings file path {}", bindings_path.display());

    /* Build imgui library */
    let mut build = cc::Build::new();
    /* C++ 20 */
    build.cpp(true).std("c++20");
    /* Include imgui dir */
    build.include(imgui_path);
    /* Add our c++ wrapper */
    build.file("wrapper.cpp");
    /* Add all source files */
    for (file_path, file) in &imgui_files {
        if let CompileFileType::Source = file.r#type {
            build.file(file_path);
        }
    }
    /* Freetype */
    #[cfg(feature = "freetype")]
    {
        build.define("IMGUI_ENABLE_FREETYPE", "1");
        for include in &freetype.include_paths {
            build.include(include);
        }
    }
    /* SDL2 */
    #[cfg(feature = "sdl2")]
    {
        for include in &sdl2.include_paths {
            build.include(&include.display().to_string());
        }
    }
    /* SDL3 */
    #[cfg(feature = "sdl3")]
    {
        for include in &sdl3.include_paths {
            build.include(&include.display().to_string());
        }
    }
    /* Remove assertions from release build */
    #[cfg(not(debug_assertions))]
    build.define("NDEBUG", None);
    /* Compile ImGui */
    build
        .try_compile("dear_imgui")
        .context("Failed to compile Dear ImGui!")?;

    Ok(())
}
