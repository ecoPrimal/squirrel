// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Core coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test code needs direct assertions"
    )
)]
#![expect(
    clippy::option_if_let_else,
    clippy::cast_possible_truncation,
    reason = "Main entry point; legacy patterns under progressive refactor"
)]

//! Squirrel AI Coordinator Main Entry Point
//!
//! `UniBin` Architecture v1.0.0 compliant entry point.
//! Modern, idiomatic async Rust with clap-based CLI.

mod cli;
mod doctor;

use anyhow::{Context, Result};
use clap::Parser;
#[cfg(feature = "monitoring")]
use squirrel::MetricsCollector;
use squirrel::shutdown::ShutdownManager;
use std::process;
use std::sync::Arc;
use tracing::{error, info, warn};

use cli::{Cli, Commands, exit_codes};

#[tokio::main]
async fn main() {
    let code = run().await;
    process::exit(code);
}

/// Returns exit code (`UniBin`: 0=success, 1=error, 2=config, 3=network, 130=interrupted)
async fn run() -> i32 {
    // Parse CLI arguments using clap
    let cli = Cli::parse();

    // Initialize tracing for all commands (verbose = debug for server)
    let log_level = match &cli.command {
        Commands::Server { verbose: true, .. } => "debug",
        _ => "info",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(format!("squirrel={log_level},warn"))
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .try_init();

    // Route to appropriate handler based on subcommand
    match cli.command {
        Commands::Server {
            port,
            bind,
            daemon,
            socket,
            verbose,
        } => {
            if let Err(e) = run_server(port, bind, daemon, socket, verbose).await {
                error!("Error: {e:?}");
                return exit_codes::ERROR;
            }
        }
        Commands::Client {
            socket,
            method,
            params,
            timeout,
        } => return run_client(socket, method, params, timeout).await,
        Commands::Doctor {
            comprehensive,
            format,
            subsystem,
        } => {
            if let Err(e) = doctor::run_doctor(comprehensive, format, subsystem) {
                error!("Error: {e:?}");
                return exit_codes::ERROR;
            }
        }
        Commands::Version { verbose } => {
            print_version(verbose);
        }
    }

    exit_codes::SUCCESS
}

/// Run client mode: connect to socket, send JSON-RPC, print response
async fn run_client(
    socket: Option<String>,
    method: String,
    params: String,
    timeout_ms: u64,
) -> i32 {
    use squirrel::rpc::unix_socket;
    use universal_patterns::IpcClient;

    let socket_path = socket.unwrap_or_else(|| {
        let node_id = unix_socket::get_node_id();
        unix_socket::get_socket_path(&node_id)
    });

    let params_value: serde_json::Value = match serde_json::from_str(&params) {
        Ok(v) => v,
        Err(e) => {
            error!("Invalid --params JSON: {e}");
            return exit_codes::CONFIG_ERROR;
        }
    };

    let timeout = std::time::Duration::from_millis(timeout_ms);
    let client = IpcClient::new(&socket_path)
        .with_request_timeout(timeout)
        .with_connection_timeout(timeout);

    let call_fut = client.call(&method, &params_value);
    tokio::select! {
        result = call_fut => match result {
            Ok(result) => {
                println!("{}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()));
                exit_codes::SUCCESS
            }
            Err(e) => {
                error!("Error: {e:?}");
                if let Some(ipc_err) = e.downcast_ref::<universal_patterns::IpcClientError>() {
                    match ipc_err {
                        universal_patterns::IpcClientError::Connection { .. }
                        | universal_patterns::IpcClientError::NotFound(_)
                        | universal_patterns::IpcClientError::Timeout { .. } => exit_codes::NETWORK_ERROR,
                        _ => exit_codes::ERROR,
                    }
                } else {
                    exit_codes::ERROR
                }
            }
        },
        _ = tokio::signal::ctrl_c() => {
            error!("Interrupted");
            exit_codes::INTERRUPTED
        }
    }
}

/// Run server mode
#[expect(
    clippy::too_many_lines,
    reason = "Server orchestration; refactor planned"
)]
async fn run_server(
    port: Option<u16>,
    bind: Option<String>,
    daemon: bool,
    socket: Option<String>,
    _verbose: bool,
) -> Result<()> {
    use squirrel::rpc::JsonRpcServer;
    use squirrel::rpc::unix_socket;

    // BTSP §Security Model (GAP-MATRIX-12): refuse to start when both
    // FAMILY_ID and BIOMEOS_INSECURE are set.
    unix_socket::validate_insecure_guard().map_err(|msg| anyhow::anyhow!("{msg}"))?;

    // Load configuration (file + env vars)
    let mut config =
        squirrel::config::ConfigLoader::load(None).context("Failed to load configuration")?;

    // CLI arguments override config file
    if let Some(ref s) = socket {
        config.server.socket = Some(s.clone());
    }
    if let Some(ref b) = bind {
        config.server.bind = b.clone();
    }
    if daemon {
        config.server.daemon = true;
    }
    if let Some(p) = port {
        config.server.port = p;
    }

    // Tracing already initialized in run() with verbose-based level

    info!("Squirrel AI/MCP Primal Starting");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Mode: Server");
    info!("UniBin Architecture v1.0.0");
    info!("Zero-HTTP Production Mode (v1.1.0)");

    #[cfg(feature = "monitoring")]
    let metrics_collector = Arc::new(MetricsCollector::new());
    #[cfg(not(feature = "monitoring"))]
    let metrics_collector = Arc::new(squirrel::monitoring::metrics::MetricsCollector::new());

    let shutdown_manager = Arc::new(ShutdownManager::new());

    info!("Metrics Collector initialized");
    info!("Shutdown Manager initialized");

    // Legacy HTTP API server REMOVED - Squirrel uses Unix sockets + JSON-RPC + tarpc!
    info!("Modern architecture: Unix sockets + JSON-RPC + tarpc");
    info!("No HTTP server - TRUE PRIMAL!");

    // Determine socket path using priority:
    // 0. TRANSPORT_ENDPOINT env (launcher-injected, sourDough standard)
    // 1. CLI --socket argument
    // 2. config.server.socket (from config file or env)
    // 3. Environment variables (SQUIRREL_SOCKET, BIOMEOS_SOCKET_PATH)
    // 4. Default fallback (XDG or /tmp)
    let socket_path = if let Ok(endpoint_json) =
        std::env::var(universal_constants::env_vars::ecosystem::TRANSPORT_ENDPOINT)
    {
        if let Some(path) = parse_transport_endpoint_socket(&endpoint_json) {
            info!("Socket path from TRANSPORT_ENDPOINT: {path}");
            path
        } else {
            warn!("TRANSPORT_ENDPOINT set but not a UDS endpoint — ignoring");
            resolve_socket_fallback(socket.clone(), &config)
        }
    } else if let Some(path) = socket.clone() {
        info!("Socket path from CLI argument: {path}");
        path
    } else if let Some(ref path) = config.server.socket {
        info!("Socket path from config: {path}");
        path.clone()
    } else {
        let node_id = unix_socket::get_node_id();
        let path = unix_socket::get_socket_path(&node_id);
        info!("Socket path from auto-detection: {path}");
        path
    };

    // Normalize relative deploy/config paths (e.g. `squirrel.sock`) to `$XDG_RUNTIME_DIR/biomeos/...`
    // or `/tmp/biomeos/...` so filesystem sockets match biomeOS composition scanning.
    let socket_path = unix_socket::resolve_socket_path_for_ipc(&socket_path)
        .to_string_lossy()
        .into_owned();

    // Daemon mode: re-exec as a detached child with stdio closed.
    // The child sees SQUIRREL_DAEMONIZED=1 and skips re-exec.
    if daemon && std::env::var(universal_constants::env_vars::squirrel::DAEMONIZED).is_err() {
        return daemonize_reexec();
    }

    let bind_host = config.server.bind.clone();

    info!("Starting JSON-RPC server...");
    info!("Socket: {socket_path}");
    if let Some(p) = port {
        info!("Port: {p} (TCP JSON-RPC on {bind_host}:{p})");
    } else {
        info!("UDS-only mode (no TCP listener)");
    }
    info!("Squirrel AI/MCP Primal Ready!");

    // Write primal manifest for biomeOS manifest-based discovery
    let manifest = universal_patterns::manifest_discovery::PrimalManifest {
        primal: squirrel::niche::PRIMAL_ID.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        socket: std::path::PathBuf::from(&socket_path),
        capabilities: squirrel::niche::CAPABILITIES
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        pid: Some(std::process::id()),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        family_id: std::env::var(universal_constants::env_vars::ecosystem::FAMILY_ID).ok(),
    };
    if let Err(e) = universal_patterns::manifest_discovery::write_manifest(&manifest) {
        warn!("Failed to write primal manifest: {e}");
    } else {
        info!("Primal manifest written for bootstrap discovery");
    }

    // Initialize AI router with capability-based discovery
    info!("Initializing AI router...");
    let ai_router = match squirrel::api::AiRouter::new_with_discovery(None).await {
        Ok(router) => {
            let provider_count = router.provider_count().await;
            if provider_count > 0 {
                info!("{provider_count} AI provider(s) discovered");
            } else {
                warn!("No AI providers found (query_ai will return 'not configured')");
                info!("Set AI_PROVIDER_SOCKETS env var for capability discovery");
            }
            Some(Arc::new(router))
        }
        Err(e) => {
            warn!("AI router initialization failed: {e}");
            info!("Server will start without AI capabilities");
            None
        }
    };

    // Resolve TCP port: CLI --port takes precedence, then SQUIRREL_PORT env
    let tcp_port: Option<u16> = port.or_else(|| {
        std::env::var(universal_constants::env_vars::squirrel::PORT)
            .or_else(|_| std::env::var(universal_constants::env_vars::squirrel::SERVER_PORT))
            .ok()
            .and_then(|p| p.parse().ok())
    });

    let shared_tracker = Arc::clone(metrics_collector.request_tracker());

    let security_orchestrator = match squirrel::security::SecuritySystemBuilder::new()
        .build()
        .await
    {
        Ok(orch) => {
            info!("Security orchestrator initialized (rate limiting + input validation active)");
            Some(orch)
        }
        Err(e) => {
            warn!(
                "Security orchestrator init failed: {e} — server will run without pre-dispatch security"
            );
            None
        }
    };

    let server = {
        let mut s = if let Some(router) = ai_router {
            JsonRpcServer::with_ai_router(socket_path.clone(), router)
        } else {
            JsonRpcServer::new(socket_path.clone())
        }
        .with_request_tracker(Arc::clone(&shared_tracker))
        .with_metrics_collector(Arc::clone(&metrics_collector));

        s = s.with_connection_timeout(std::time::Duration::from_secs(
            config.server.connection_timeout_secs,
        ));
        if let Some(orch) = security_orchestrator {
            s = s.with_security_orchestrator(orch);
        }
        if let Some(p) = tcp_port {
            s = s.with_tcp_port(p);
        }
        Arc::new(s)
    };
    let server_clone = Arc::clone(&server);

    // Start server in background task
    let server_task = tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            error!("Server error: {e}");
        }
    });

    // G64 Cephalization: tarpc binary RPC on sibling .tarpc.sock
    #[cfg(all(unix, feature = "tarpc-rpc"))]
    {
        use futures::StreamExt;
        use squirrel::rpc::SquirrelRpc;
        use tarpc::server::{BaseChannel, Channel};

        let tarpc_socket = format!(
            "{}.tarpc.sock",
            socket_path.strip_suffix(".sock").unwrap_or(&socket_path)
        );
        let _ = std::fs::remove_file(&tarpc_socket);
        let tarpc_server = squirrel::rpc::TarpcRpcServer::from_jsonrpc(Arc::clone(&server));
        match tarpc::serde_transport::unix::listen(
            &tarpc_socket,
            tokio_serde::formats::Bincode::default,
        )
        .await
        {
            Ok(mut listener) => {
                info!(path = %tarpc_socket, "tarpc UDS listener bound");
                tokio::spawn(async move {
                    while let Some(accept_result) = listener.next().await {
                        match accept_result {
                            Ok(transport) => {
                                let srv = tarpc_server.clone();
                                tokio::spawn(
                                    BaseChannel::with_defaults(transport)
                                        .execute(srv.serve())
                                        .for_each(|resp| async { tokio::spawn(resp); }),
                                );
                            }
                            Err(e) => {
                                warn!(error = %e, "tarpc accept failed");
                            }
                        }
                    }
                });
                info!(path = %tarpc_socket, "tarpc listener spawned");
            }
            Err(e) => {
                warn!(error = %e, path = %tarpc_socket, "tarpc listener failed (non-fatal)");
            }
        }
    }

    // Install signal handlers (SIGTERM + SIGINT) for socket cleanup
    let (shutdown_tx, signal_task) =
        squirrel::capabilities::lifecycle::install_signal_handlers(socket_path.clone());
    let shutdown_rx = shutdown_tx.subscribe();

    // Write PID file alongside socket (CAPABILITY_BASED_DISCOVERY_STANDARD v1.3.0 §6)
    let pid_path = format!("{socket_path}.pid");
    if let Err(e) = std::fs::write(&pid_path, process::id().to_string()) {
        warn!("Could not write PID file {pid_path}: {e}");
    } else {
        info!("PID file: {pid_path}");
    }

    // biomeOS lifecycle registration (healthSpring pattern)
    if let Some(biomeos_socket) = squirrel::capabilities::lifecycle::find_biomeos_socket() {
        let caps = server.capability_registry.method_names();
        let cap_refs: Vec<&str> = caps.as_slice().to_vec();
        if squirrel::capabilities::lifecycle::register_with_biomeos(
            &biomeos_socket,
            &socket_path,
            &cap_refs,
        )
        .await
        {
            info!("Registered with ecosystem orchestrator");

            let hb_interval = std::time::Duration::from_secs(config.server.heartbeat_interval_secs);
            let _heartbeat = squirrel::capabilities::lifecycle::spawn_heartbeat(
                biomeos_socket,
                socket_path.clone(),
                hb_interval,
                shutdown_rx,
            );
            info!("Heartbeat started ({}s interval)", hb_interval.as_secs());
        }
    } else {
        info!("No ecosystem orchestrator socket found — standalone mode");
    }

    // Neural API primal.announce with routing metadata (Wave 43/44)
    // Independent of lifecycle registration — targets neural-api socket specifically
    squirrel::capabilities::lifecycle::announce_to_neural_api(&socket_path).await;

    // Service-mesh registration via discovery socket
    let shutdown_rx_discovery = shutdown_tx.subscribe();
    if let Some(discovery_socket) = squirrel::capabilities::discovery_service::discover_socket() {
        if squirrel::capabilities::discovery_service::register(&discovery_socket, &socket_path)
            .await
        {
            info!("Registered with discovery service");

            let hb_interval = std::time::Duration::from_secs(config.server.heartbeat_interval_secs);
            let _discovery_heartbeat =
                squirrel::capabilities::discovery_service::start_heartbeat_loop(
                    discovery_socket,
                    socket_path.clone(),
                    hb_interval,
                    shutdown_rx_discovery,
                );
            info!(
                "Discovery heartbeat started ({}s interval)",
                hb_interval.as_secs()
            );
        }
    } else {
        info!("No discovery socket found — peer discovery unavailable");
    }

    info!("Press Ctrl+C to stop");

    let family_id = std::env::var(universal_constants::env_vars::ecosystem::FAMILY_ID).ok();
    tokio::select! {
        _ = signal_task => {
            info!("Shutting down gracefully...");

            let _ = std::fs::remove_file(&pid_path);

            if let Err(e) = universal_patterns::manifest_discovery::remove_manifest(
                squirrel::niche::PRIMAL_ID,
                family_id.as_deref(),
            ) {
                warn!("Failed to remove primal manifest: {e}");
            }

            if let Err(e) = shutdown_manager.request_shutdown().await {
                warn!("Shutdown error: {e}");
            }

            info!("Shutdown complete");
        }
        _ = server_task => {
            info!("Server task completed");
            let _ = std::fs::remove_file(&pid_path);
        }
    }

    Ok(())
}

/// Re-exec the current binary as a detached daemon (safe, no `unsafe`).
///
/// 1. Spawns a child process with `SQUIRREL_DAEMONIZED=1` and stdio → `/dev/null`.
/// 2. The child re-runs `main()`, sees the env var, and skips re-exec.
/// 3. Parent prints the child PID (for biomeOS / shell capture) and returns.
///
/// biomeOS discovers the daemon via socket probing + manifest;
/// the PID is written to the primal manifest by the child.
fn daemonize_reexec() -> Result<()> {
    use std::process::{Command, Stdio};

    let exe = std::env::current_exe().context("cannot determine executable path")?;
    let args: Vec<String> = std::env::args()
        .skip(1) // skip argv[0]
        .filter(|a| a != "--daemon" && a != "-d")
        .collect();

    let child = Command::new(exe)
        .args(&args)
        .env("SQUIRREL_DAEMONIZED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn daemon child")?;

    println!("squirrel daemon started (pid: {})", child.id());
    Ok(())
}

/// Print version information to stdout (CLI output, not logging).
fn print_version(verbose: bool) {
    println!("squirrel {}", env!("CARGO_PKG_VERSION"));
    if verbose {
        println!();
        println!("Architecture:   UniBin v1.0.0 / ecoBin v3.0");
        println!("Protocol:       JSON-RPC 2.0 + tarpc (dual)");
        println!("Transport:      Unix socket / Named pipe / TCP");
        println!("Discovery:      Capability-based runtime");
        println!("License:        AGPL-3.0-or-later");
    }
}

/// Parse a sourDough `TransportEndpoint` JSON string into a UDS socket path.
/// Returns `None` if the endpoint is not a UDS variant.
fn parse_transport_endpoint_socket(json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let transport = value.get("transport")?.as_str()?;
    if transport == "uds" {
        value.get("path")?.as_str().map(String::from)
    } else {
        None
    }
}

/// Fallback socket path resolution (CLI → config → auto-detect).
fn resolve_socket_fallback(
    socket: Option<String>,
    config: &squirrel::config::SquirrelConfig,
) -> String {
    use squirrel::rpc::unix_socket;
    if let Some(path) = socket {
        info!("Socket path from CLI argument: {path}");
        path
    } else if let Some(ref path) = config.server.socket {
        info!("Socket path from config: {path}");
        path.clone()
    } else {
        let node_id = unix_socket::get_node_id();
        let path = unix_socket::get_socket_path(&node_id);
        info!("Socket path from auto-detection: {path}");
        path
    }
}
