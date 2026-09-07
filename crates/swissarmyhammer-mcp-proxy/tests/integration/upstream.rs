//! Shared upstream MCP server used by the proxy integration tests.
//!
//! Every test that needs a live upstream goes through this module. Keeping a
//! single entry point matters: the upstream is a real SwissArmyHammer MCP
//! server, and it materializes working state — a kanban board directory among
//! other things — inside whatever working directory it is given. Cargo runs a
//! test binary with the current directory set to its own package root, so an
//! upstream started without an explicit working directory writes that state
//! into `crates/swissarmyhammer-mcp-proxy` and pollutes the source tree.

use rmcp::ServiceExt;
use std::sync::OnceLock;
use swissarmyhammer_mcp_proxy::{FilteringMcpProxy, ToolFilter};
use swissarmyhammer_tools::mcp::unified_server::{start_mcp_server, McpServerMode};

/// Maximum number of times to poll the upstream health endpoint before giving up.
const MAX_HEALTH_RETRIES: usize = 50;

/// Delay in milliseconds between upstream health-check retries.
/// Combined with `MAX_HEALTH_RETRIES`, this gives a 5-second ceiling.
const HEALTH_RETRY_DELAY_MS: u64 = 100;

/// In-memory duplex buffer size used for the client↔proxy transport.
/// Large enough to comfortably hold a full MCP message without blocking.
pub(crate) const DUPLEX_BUFFER_SIZE: usize = 65536;

/// A minimal MCP client handler used in tests.
#[derive(Debug, Clone, Default)]
pub(crate) struct TestClientHandler;

impl rmcp::ClientHandler for TestClientHandler {
    fn get_info(&self) -> rmcp::model::ClientInfo {
        rmcp::model::ClientInfo::default()
    }
}

/// URL of a shared upstream MCP server, started once per test binary.
///
/// Each `#[tokio::test]` uses its own runtime, so the upstream cannot live
/// on a test's runtime (its background tasks would die when that runtime
/// shuts down). Instead we spawn a dedicated OS thread with its own
/// multi-threaded tokio runtime that runs for the lifetime of the process.
static UPSTREAM_URL: OnceLock<String> = OnceLock::new();

/// Return the URL of the shared upstream MCP server, starting it on first call.
///
/// Safe to call concurrently from multiple tests — the `OnceLock` ensures
/// exactly one server is started. Waits until the upstream health endpoint
/// responds before returning so that tests can immediately issue MCP requests.
pub(crate) fn shared_upstream_url() -> String {
    UPSTREAM_URL.get_or_init(start_shared_upstream).clone()
}

/// Spawn the upstream MCP server on a dedicated OS thread and block until it is
/// ready, returning its URL. The server runs forever on that thread's runtime.
fn start_shared_upstream() -> String {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("mcp-proxy-test-upstream".into())
        .spawn(move || run_upstream_forever(tx))
        .expect("spawn upstream thread");
    rx.recv().expect("receive upstream url")
}

/// Thread body: run a dedicated tokio runtime that hosts the upstream MCP server
/// for the lifetime of the test process, sending its URL once it is healthy.
///
/// The server is given a temporary working directory. Passing `None` here
/// would make `start_mcp_server` fall back to the current directory, which
/// cargo sets to this crate's package root, and the server would then write
/// its working state into the source tree.
fn run_upstream_forever(url_tx: std::sync::mpsc::Sender<String>) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build upstream runtime");
    rt.block_on(async move {
        let work_dir = tempfile::TempDir::new().expect("create upstream working directory");
        let handle = start_mcp_server(
            McpServerMode::Http { port: None },
            None,
            Some(work_dir.path().to_path_buf()),
        )
        .await
        .expect("failed to start upstream MCP server");
        let port = handle.info().port.expect("upstream has no port");
        let url = format!("http://127.0.0.1:{}/mcp", port);

        wait_for_health(port).await;

        url_tx.send(url).expect("send upstream url");
        // Hold the handle and the working directory so the server stays up,
        // and its state stays on disk, until process exit.
        std::future::pending::<()>().await;
        drop(handle);
        drop(work_dir);
    });
}

/// Poll the upstream health endpoint until it responds or the retry budget is
/// exhausted. Returns in either case; failures surface as later test errors.
async fn wait_for_health(port: u16) {
    let health_url = format!("http://127.0.0.1:{}/health", port);
    let client = reqwest::Client::new();
    for _ in 0..MAX_HEALTH_RETRIES {
        if client.get(&health_url).send().await.is_ok() {
            return;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(HEALTH_RETRY_DELAY_MS)).await;
    }
}

/// Build a FilteringMcpProxy pointed at the given upstream URL with the given filter
/// and serve it over the given transport in the background.
///
/// Returns the RunningService for the client side — callers must keep this alive
/// for the duration of the test so the underlying service does not stop.
pub(crate) async fn make_client(
    upstream_url: String,
    filter: ToolFilter,
) -> rmcp::service::RunningService<rmcp::RoleClient, TestClientHandler> {
    let proxy = FilteringMcpProxy::new(upstream_url, filter);

    let (server_transport, client_transport) = tokio::io::duplex(DUPLEX_BUFFER_SIZE);
    tokio::spawn(async move {
        if let Ok(service) = proxy.serve(server_transport).await {
            let _ = service.waiting().await;
        }
    });

    TestClientHandler
        .serve(client_transport)
        .await
        .expect("client connection failed")
}
