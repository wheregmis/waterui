//! Web platform implementation.
//!
//! Note: running the local web development server requires Python in PATH
//! (`python3` preferred, `python` fallback).

use std::path::{Path, PathBuf};

use color_eyre::eyre;
use smol::{fs, process::Command};
use target_lexicon::Triple;

use crate::{
    build::BuildOptions, device::Artifact, project::Project, utils::command,
    web::toolchain::WebToolchain,
};

/// Default local web development server port.
pub const WEB_DEV_SERVER_PORT: u16 = 8000;

/// Web platform implementation.
#[derive(Debug, Clone, Copy)]
pub struct WebPlatform;

impl WebPlatform {
    /// Create a new web platform instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Build the wasm artifact and generate JS bindings.
    pub async fn build(&self, project: &Project, options: BuildOptions) -> eyre::Result<PathBuf> {
        let target = "wasm32-unknown-unknown";
        let out_dir = project.root().join("web").join("pkg");

        fs::create_dir_all(&out_dir).await?;

        let mut cargo = Command::new("cargo");
        let mut cargo = command(&mut cargo)
            .arg("build")
            .arg("--lib")
            .arg("--target")
            .arg(target)
            .current_dir(project.root());
        if options.is_release() {
            cargo = cargo.arg("--release");
        }
        let status = cargo.status().await?;
        if !status.success() {
            return Err(eyre::eyre!("cargo build failed for web target"));
        }

        let profile = if options.is_release() {
            "release"
        } else {
            "debug"
        };
        let wasm_input = project
            .target_dir()
            .join(target)
            .join(profile)
            .join(format!("{}.wasm", project.crate_name().replace('-', "_")));

        if !wasm_input.exists() {
            return Err(eyre::eyre!(
                "expected wasm artifact not found at {}",
                wasm_input.display()
            ));
        }

        let mut bindgen = Command::new("wasm-bindgen");
        let status = command(&mut bindgen)
            .arg(&wasm_input)
            .arg("--target")
            .arg("web")
            .arg("--out-dir")
            .arg(&out_dir)
            .current_dir(project.root())
            .status()
            .await?;
        if !status.success() {
            return Err(eyre::eyre!("wasm-bindgen failed"));
        }

        ensure_web_assets(project.root(), project.crate_name()).await?;

        Ok(out_dir)
    }

    /// Package web output into a deployable directory.
    pub async fn package(&self, project: &Project) -> eyre::Result<Artifact> {
        let web_dir = project.root().join("web");
        let pkg_dir = web_dir.join("pkg");
        if !pkg_dir.exists() {
            return Err(eyre::eyre!(
                "web package not found; run `water build --platform web` first"
            ));
        }

        ensure_web_assets(project.root(), project.crate_name()).await?;
        Ok(Artifact::new(project.bundle_identifier(), web_dir))
    }

    /// Clean web build artifacts.
    pub async fn clean(&self, project: &Project) -> eyre::Result<()> {
        let web_pkg = project.root().join("web").join("pkg");
        if web_pkg.exists() {
            fs::remove_dir_all(web_pkg).await?;
        }
        Ok(())
    }

    /// Run a simple local web server for the generated web app.
    pub async fn run(&self, project: &Project) -> eyre::Result<()> {
        let web_dir = project.root().join("web");
        if !web_dir.exists() {
            return Err(eyre::eyre!(
                "web directory is missing; run `water build --platform web` first"
            ));
        }

        let mut server = if crate::utils::which("python3").await.is_ok() {
            Command::new("python3")
        } else if crate::utils::which("python").await.is_ok() {
            Command::new("python")
        } else {
            return Err(eyre::eyre!(
                "Python is required to run the web dev server. Install Python 3 and ensure `python3` or `python` is in PATH."
            ));
        };

        let status = command(&mut server)
            .arg("-m")
            .arg("http.server")
            .arg(WEB_DEV_SERVER_PORT.to_string())
            .arg("--directory")
            .arg(&web_dir)
            .status()
            .await?;
        if !status.success() {
            return Err(eyre::eyre!("failed to start web server"));
        }
        Ok(())
    }

    /// Target triple for web builds.
    #[must_use]
    pub fn triple(&self) -> Triple {
        "wasm32-unknown-unknown"
            .parse()
            .expect("valid wasm target triple")
    }

    /// Toolchain for web builds.
    #[must_use]
    pub const fn toolchain(&self) -> WebToolchain {
        WebToolchain
    }
}

impl Default for WebPlatform {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensure required web assets exist under `<project>/web`.
///
/// Creates `web/` if missing and writes a default `index.html` when absent.
async fn ensure_web_assets(root: &Path, crate_name: &str) -> eyre::Result<()> {
    let web_dir = root.join("web");
    fs::create_dir_all(&web_dir).await?;

    let index_html = web_dir.join("index.html");
    if !index_html.exists() {
        let js_module = format!("./pkg/{}.js", crate_name.replace('-', "_"));
        let content = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>WaterUI Web App</title>
  </head>
  <body>
    <script type="module">
      import init from "__WASM_JS_MODULE__";
      init();
    </script>
  </body>
</html>
"#
        .replace("__WASM_JS_MODULE__", &js_module);
        fs::write(index_html, content).await?;
    }

    Ok(())
}
