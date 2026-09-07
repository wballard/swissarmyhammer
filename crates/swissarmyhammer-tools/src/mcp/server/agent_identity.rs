//! The kanban actor a connecting MCP client is given.
//!
//! A client announces a name; the board needs a stable id, a colour, and an
//! avatar for it. All three are derived from that name alone, so the same
//! client reconnecting is the same actor.

use super::McpServer;
use std::path::PathBuf;
use swissarmyhammer_kanban::auto_color::palette_color;
use swissarmyhammer_templating::filters::slugify_string;

impl McpServer {
    /// Ensure an agent actor exists for the connecting MCP client.
    ///
    /// Slugifies the client name as the actor ID, derives a deterministic color,
    /// and generates a geometric SVG avatar. Idempotent via `ensure: true`.
    pub(super) async fn ensure_agent_actor(&self, client_name: &str) {
        use swissarmyhammer_kanban::actor::AddActor;
        use swissarmyhammer_kanban::{Execute, KanbanContext};

        let working_dir = self
            .tool_context
            .working_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));
        let kanban_dir = working_dir.join(".kanban");

        if !kanban_dir.is_dir() {
            tracing::debug!("no .kanban directory, skipping agent actor creation");
            return;
        }

        let actor_id = slugify_string(client_name);
        let color = agent_deterministic_color(&actor_id);

        // No stored avatar — frontend renders initials as fallback
        let ctx = KanbanContext::new(kanban_dir);
        let cmd = AddActor::new(actor_id.as_str(), client_name)
            .with_ensure()
            .with_color(&color);

        match cmd.execute(&ctx).await.into_result() {
            Ok(result) => {
                let created = result["created"].as_bool().unwrap_or(false);
                if created {
                    tracing::info!(id = %actor_id, name = %client_name, "created MCP agent actor");
                } else {
                    tracing::debug!(id = %actor_id, "MCP agent actor already exists");
                }
                // Store the actor_id so tool calls can auto-inject it
                *self.tool_context.session_actor.write().await = Some(actor_id);
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to ensure MCP agent actor");
            }
        }
    }
}

/// Curated palette for agent actors (cooler tones to distinguish from human actors).
const AGENT_COLORS: &[&str] = &[
    "5a67d8", "3182ce", "319795", "2f855a", "805ad5", "6b46c1", "2b6cb0", "2c7a7b", "4c51bf",
    "38a169",
];

/// Derive a deterministic hex color for an agent actor.
///
/// The actor id picks one entry of [`AGENT_COLORS`] through the shared djb2
/// hash, so the same client name always arrives at the same colour. The
/// palette stays here because it is this surface's own; only the hash is
/// shared, with the kanban app that colours human actors.
fn agent_deterministic_color(id: &str) -> String {
    palette_color(AGENT_COLORS, id).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The actor id comes from the shared `slugify_string`. These are the client
    /// names the board must keep answering the same id for, so the contract this
    /// module leans on is pinned here.
    #[test]
    fn test_slugify_string_derives_the_actor_id() {
        assert_eq!(slugify_string("Claude Code"), "claude-code");
        assert_eq!(slugify_string("my_agent"), "my-agent");
        assert_eq!(slugify_string("  spaces  "), "spaces");
        assert_eq!(slugify_string("UPPER"), "upper");
        assert_eq!(slugify_string("a--b"), "a-b");
    }

    #[test]
    fn test_agent_deterministic_color_stable() {
        let c1 = agent_deterministic_color("claude-code");
        let c2 = agent_deterministic_color("claude-code");
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 6);
    }

    /// The colour every listed actor id has always been given.
    ///
    /// A colour is written onto the actor the first time a client connects and
    /// is read back on every board that already holds that actor, so a new
    /// answer here re-colours agents that exist. The table was computed from
    /// the djb2 definition and [`AGENT_COLORS`] alone, not from this module's
    /// code, so it fails whenever the two stop agreeing.
    const PINNED_AGENT_COLORS: &[(&str, &str)] = &[
        ("claude-code", "6b46c1"),
        ("claude", "2c7a7b"),
        ("cursor", "2f855a"),
        ("codex", "319795"),
        ("my-agent", "3182ce"),
        ("spaces", "805ad5"),
        ("upper", "38a169"),
        ("a-b", "2f855a"),
        ("", "3182ce"),
    ];

    #[test]
    fn test_agent_deterministic_color_matches_pinned_table() {
        for (id, expected) in PINNED_AGENT_COLORS {
            assert_eq!(
                agent_deterministic_color(id),
                *expected,
                "the colour of agent {id:?} changed"
            );
        }
    }
}
