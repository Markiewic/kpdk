use std::path::PathBuf;

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    schemars, tool, tool_router,
    transport::stdio,
    ServiceExt,
};
use serde::{Deserialize, Serialize};

use crate::commands::build;
use crate::device;
use crate::error::{Error, Result};

#[derive(Clone)]
struct KpdkMcp;

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct DeviceInfo {
    name: String,
    architecture: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct SupportedDevicesResult {
    devices: Vec<DeviceInfo>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuildProjectArgs {
    /// Path to a kpdk project directory containing pdk.toml.
    project: String,
    /// Build with SDCC size optimizations enabled.
    #[serde(default)]
    release: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct BuildArtifacts {
    ihx: String,
    bin: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct BuildProjectResult {
    success: bool,
    device: Option<String>,
    artifacts: Option<BuildArtifacts>,
    diagnostics: Vec<String>,
    error: Option<String>,
}

#[tool_router(server_handler)]
impl KpdkMcp {
    #[tool(description = "List the exact Padauk devices supported by this kpdk build")]
    fn get_supported_devices(&self) -> Json<SupportedDevicesResult> {
        Json(SupportedDevicesResult {
            devices: device::supported_devices()
                .map(|(name, architecture)| DeviceInfo {
                    name: name.to_owned(),
                    architecture: architecture.sdcc_target().to_owned(),
                })
                .collect(),
        })
    }

    #[tool(description = "Build an existing kpdk firmware project with the installed toolchain")]
    fn build_project(
        &self,
        Parameters(args): Parameters<BuildProjectArgs>,
    ) -> Json<BuildProjectResult> {
        match build::run_silent(&PathBuf::from(args.project), args.release) {
            Ok(artifacts) => Json(BuildProjectResult {
                success: true,
                device: Some(artifacts.device),
                artifacts: Some(BuildArtifacts {
                    ihx: artifacts.ihx.display().to_string(),
                    bin: artifacts.bin.display().to_string(),
                }),
                diagnostics: artifacts.diagnostics,
                error: None,
            }),
            Err(error) => Json(BuildProjectResult {
                success: false,
                device: None,
                artifacts: None,
                diagnostics: Vec::new(),
                error: Some(error.to_string()),
            }),
        }
    }
}

pub fn run() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|error| Error::Message(format!("failed to start MCP runtime: {error}")))?;

    runtime.block_on(async {
        let service = KpdkMcp
            .serve(stdio())
            .await
            .map_err(|error| Error::Message(format!("failed to start MCP server: {error}")))?;
        service.waiting().await.map_err(|error| {
            Error::Message(format!("MCP server stopped with an error: {error}"))
        })?;
        Ok(())
    })
}
