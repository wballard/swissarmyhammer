//! Guards the source tree against state written by the test upstream server.
//!
//! The upstream these tests start is a real SwissArmyHammer MCP server. It
//! creates a kanban board directory inside its working directory, and
//! `start_mcp_server` falls back to the current directory when no working
//! directory is given. Cargo sets a test binary's current directory to its own
//! package root, so that fallback lands the board in
//! `crates/swissarmyhammer-mcp-proxy`. A board directory left in the source
//! tree is not merely clutter: the desktop application opens whatever board
//! directory it finds, and a board opened by mistake reads as empty.

use super::upstream::{make_client, shared_upstream_url};
use std::path::PathBuf;
use swissarmyhammer_mcp_proxy::ToolFilter;

/// Directory name the kanban tools create inside their working directory.
const BOARD_DIR_NAME: &str = ".kanban";

/// Path at which a stray board directory would appear for this crate.
///
/// Cargo runs the test binary with the current directory set to the package
/// root, so this resolves to `crates/swissarmyhammer-mcp-proxy/.kanban`.
fn crate_board_dir() -> PathBuf {
    std::env::current_dir()
        .expect("test process has a current directory")
        .join(BOARD_DIR_NAME)
}

/// Name of the upstream tool that materializes a board in its working directory.
const KANBAN_TOOL_NAME: &str = "kanban";

/// Calling the upstream kanban tool through the proxy must not create a board
/// directory in the crate directory.
///
/// The kanban tool opens the board that belongs to the upstream server's
/// working directory, creating it when it is absent. That is the operation
/// that wrote the board recovered from this crate: the original version of
/// `handler_tests.rs` called `tools[0]`, which at the time resolved to the
/// kanban tool. The board it left behind carried a single actor named
/// `filtering-proxy`, the client identity the proxy sends upstream.
///
/// Calling the tool by name rather than by list position keeps this test
/// pinned to the operation that matters, instead of depending on which tool
/// happens to sort first.
#[tokio::test]
async fn upstream_does_not_write_board_into_crate_directory() {
    let board_dir = crate_board_dir();
    assert!(
        !board_dir.exists(),
        "precondition failed: a board directory already exists at {}. \
         Remove it before running this test.",
        board_dir.display()
    );

    let filter = ToolFilter::new(vec![], vec![]).expect("valid filter");
    let peer = make_client(shared_upstream_url(), filter).await;

    let mut arguments = serde_json::Map::new();
    arguments.insert("op".to_string(), serde_json::json!("get board"));
    let params =
        rmcp::model::CallToolRequestParams::new(KANBAN_TOOL_NAME).with_arguments(arguments);

    // The call may succeed or return a tool-level error; either way the
    // upstream has resolved and opened its board by this point, which is what
    // this test observes. A transport failure would mean nothing was
    // exercised, so surface that.
    if let Err(rmcp::service::ServiceError::TransportClosed) = peer.call_tool(params).await {
        panic!("kanban call was never forwarded to the upstream");
    }

    assert!(
        !board_dir.exists(),
        "the upstream MCP server wrote a kanban board into the crate directory at {}. \
         It must be started with an isolated working directory, not the default \
         current directory.",
        board_dir.display()
    );
}
