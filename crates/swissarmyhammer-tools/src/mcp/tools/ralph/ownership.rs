//! Ralph instruction ownership
//!
//! Decides whether a stored instruction belongs to the session of the current
//! process. `set ralph` runs in the MCP server process and `check ralph` runs
//! in a hook CLI process — two different processes that one harness process
//! spawned as children. Their session ids never match, so ownership is decided
//! from the process tree instead:
//!
//! - The instruction records the pid of the process that wrote it.
//! - The check accepts the instruction only when that owner process is alive
//!   and sits in the same process tree: the owner is a proper ancestor of the
//!   checking process (in-process harness), or the owner's parent is (the
//!   harness spawned both as siblings).
//!
//! A dead owner never matches — its session has ended. A live owner in a
//! different process tree never matches — it is a live peer session. System
//! pids (0 and 1) never match, so an orphaned owner re-parented to init
//! cannot capture every session on the machine.

use std::collections::HashSet;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Highest number of parent hops the ancestor walk takes.
///
/// Real process trees are a handful of levels deep; the cap guards against a
/// cyclic or corrupted parent chain looping forever.
const MAX_ANCESTOR_DEPTH: usize = 32;

/// Lowest pid that can take part in an ownership match.
///
/// Pid 0 (kernel) and pid 1 (init/launchd) are ancestors of every process on
/// the machine, so matching on them would capture every session.
const FIRST_NON_SYSTEM_PID: u32 = 2;

/// Report whether the process with `pid` is currently alive.
pub fn is_pid_alive(pid: u32) -> bool {
    let mut system = System::new();
    refresh_pid(&mut system, pid);
    system.process(Pid::from_u32(pid)).is_some()
}

/// Report whether the process that wrote an instruction owns the session the
/// current process runs in.
///
/// True only when the owner is a different, live process in the same process
/// tree — a proper ancestor of the current process, or a sibling under a
/// shared parent that is a proper ancestor. See the module docs for why this
/// stands in for a session-id match.
pub fn owner_owns_current_session(owner_pid: u32) -> bool {
    let self_pid = std::process::id();
    if owner_pid == self_pid {
        return false;
    }

    let mut system = System::new();
    refresh_pid(&mut system, owner_pid);
    if system.process(Pid::from_u32(owner_pid)).is_none() {
        return false;
    }

    let owner_parent = parent_of(&mut system, owner_pid);
    let ancestors = current_ancestor_pids(&mut system);
    owner_matches_session(owner_pid, owner_parent, &ancestors)
}

/// Pull the metadata of one pid into `system`.
///
/// `ProcessRefreshKind::nothing()` still retrieves the pid and parent pid,
/// which is all the ownership decision reads.
fn refresh_pid(system: &mut System, pid: u32) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::nothing(),
    );
}

/// The parent pid of the current process, when it has one.
///
/// The parent is the anchor of the ownership match: in the harness topology
/// it is the process that spawned both the instruction's writer and the
/// checking process. Tests write an instruction owned by this pid to stand
/// in for such a sibling writer.
pub fn current_parent_pid() -> Option<u32> {
    parent_of(&mut System::new(), std::process::id())
}

/// The parent pid of `pid`, when the process is alive and has one.
fn parent_of(system: &mut System, pid: u32) -> Option<u32> {
    refresh_pid(system, pid);
    system
        .process(Pid::from_u32(pid))
        .and_then(|process| process.parent())
        .map(Pid::as_u32)
}

/// The proper ancestors of the current process, nearest first.
///
/// Walks the parent chain from the current process, capped at
/// [`MAX_ANCESTOR_DEPTH`] hops. The current process itself is not in the set.
fn current_ancestor_pids(system: &mut System) -> HashSet<u32> {
    let mut ancestors = HashSet::new();
    let mut pid = std::process::id();
    for _ in 0..MAX_ANCESTOR_DEPTH {
        let Some(parent) = parent_of(system, pid) else {
            break;
        };
        if !ancestors.insert(parent) {
            break;
        }
        pid = parent;
    }
    ancestors
}

/// The pure ownership decision over already-gathered process facts.
///
/// `ancestors` holds the proper ancestors of the checking process. The owner
/// matches when it is one of them, or when its parent is. Matches on system
/// pids are refused on both routes.
fn owner_matches_session(
    owner_pid: u32,
    owner_parent: Option<u32>,
    ancestors: &HashSet<u32>,
) -> bool {
    let matchable = |pid: u32| pid >= FIRST_NON_SYSTEM_PID && ancestors.contains(&pid);
    matchable(owner_pid) || owner_parent.is_some_and(matchable)
}

/// Test fixtures shared by the ralph test modules.
#[cfg(test)]
pub mod test_support {
    /// A pid far above any real pid on macOS (max ~100k) or Linux
    /// (`pid_max` <= 4194304), so no live process can hold it.
    pub const DEAD_PID: u32 = 0x7FFF_FFFD;
}

#[cfg(test)]
mod tests {
    use super::test_support::DEAD_PID;
    use super::*;

    fn ancestors_of(pids: &[u32]) -> HashSet<u32> {
        pids.iter().copied().collect()
    }

    #[test]
    fn test_owner_matches_when_owner_is_an_ancestor() {
        let ancestors = ancestors_of(&[500, 400, 300]);
        assert!(owner_matches_session(400, Some(300), &ancestors));
    }

    #[test]
    fn test_owner_matches_when_owner_parent_is_an_ancestor() {
        // Sibling topology: owner 600 and the checker were both spawned by 500.
        let ancestors = ancestors_of(&[500, 400]);
        assert!(owner_matches_session(600, Some(500), &ancestors));
    }

    #[test]
    fn test_owner_in_another_tree_does_not_match() {
        let ancestors = ancestors_of(&[500, 400]);
        assert!(!owner_matches_session(700, Some(710), &ancestors));
    }

    #[test]
    fn test_owner_without_a_known_parent_does_not_match() {
        let ancestors = ancestors_of(&[500, 400]);
        assert!(!owner_matches_session(700, None, &ancestors));
    }

    #[test]
    fn test_system_pid_parent_does_not_match() {
        // An orphaned owner re-parented to init: pid 1 is everyone's ancestor
        // and must never be treated as a shared session parent.
        let ancestors = ancestors_of(&[500, 1]);
        assert!(!owner_matches_session(700, Some(1), &ancestors));
    }

    #[test]
    fn test_system_pid_owner_does_not_match() {
        let ancestors = ancestors_of(&[500, 1]);
        assert!(!owner_matches_session(1, Some(0), &ancestors));
    }

    #[test]
    fn test_current_process_is_not_its_own_session_owner() {
        assert!(!owner_owns_current_session(std::process::id()));
    }

    #[test]
    fn test_parent_process_owns_current_session() {
        // The parent of the test process (the cargo test runner) is alive and
        // is a proper ancestor — exactly the harness topology.
        let parent = current_parent_pid().expect("the test process has a parent");
        assert!(owner_owns_current_session(parent));
    }

    #[test]
    fn test_dead_pid_does_not_own_current_session() {
        assert!(!owner_owns_current_session(DEAD_PID));
    }

    #[test]
    fn test_dead_pid_is_not_alive() {
        assert!(!is_pid_alive(DEAD_PID));
    }

    #[test]
    fn test_current_pid_is_alive() {
        assert!(is_pid_alive(std::process::id()));
    }
}
