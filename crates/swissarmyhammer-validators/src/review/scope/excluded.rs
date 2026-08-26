//! The files the scope stage drops before any validator pairs with them, and
//! the reasons it drops one.
//!
//! An exclusion is never silent. A run that reviewed fewer files than it
//! resolved says which files and why, so a reader can tell a deliberate
//! exclusion from a file the run missed. The report renders each kind, and a
//! scope every one of whose files was excluded DELIBERATELY is a clean review
//! that states the exclusion — never an empty scope.
//!
//! The kinds arrive from different stages and must never be conflated:
//! [`ExclusionKind::ReviewIgnore`] comes from the `.reviewignore` /
//! `.gitignore` filter in [`super::resolve`],
//! [`ExclusionKind::ValidatorFixture`] from the fixture split in
//! [`super::fixtures`], and [`ExclusionKind::NoMatchingValidator`] from the
//! validator pairing in [`super::scope_review`].
//!
//! Only the first two are deliberate. A file no validator matched is a
//! coverage GAP: nothing asked for it to go unread, so it is reported the same
//! way but never counted toward the clean full-exclusion claim
//! ([`ExclusionKind::is_deliberate`]).

use serde::Serialize;

/// Why the scope stage dropped a file.
///
/// The report renders each kind differently — an ignore exclusion is grouped
/// under the pattern that excluded it, a fixture exclusion and an unmatched
/// file are named per file under their own heading — so a reader sees at a
/// glance whether a configuration, the validator store, or a plain lack of
/// coverage took the file out of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ExclusionKind {
    /// A pattern in `.reviewignore` or `.gitignore` matched the file.
    ReviewIgnore,
    /// The file is a validator set's own fixture data.
    ValidatorFixture,
    /// No loaded validator's `match` accepted the file, so the pairing stage
    /// had nothing to review it with.
    NoMatchingValidator,
}

impl ExclusionKind {
    /// Whether this kind is an exclusion someone ASKED for, as opposed to a
    /// coverage gap.
    ///
    /// An ignore pattern is the repository's own configuration and a fixture
    /// is data the validator store declares: both mean "do not read this", so
    /// a scope covered entirely by them is a clean, passing review. No
    /// validator matching a file means nothing read it and nothing intended
    /// that, so it can never carry that claim — it is the exact reading
    /// `^g7d3tzq` exists to prevent.
    pub fn is_deliberate(self) -> bool {
        match self {
            ExclusionKind::ReviewIgnore | ExclusionKind::ValidatorFixture => true,
            ExclusionKind::NoMatchingValidator => false,
        }
    }
}

/// The reason recorded for a file dropped because it is a validator set's own
/// fixture data.
const VALIDATOR_FIXTURE_REASON: &str = "validator fixture";

/// The reason recorded for a file no loaded validator matched.
const NO_MATCHING_VALIDATOR_REASON: &str = "no validator matches this file";

/// A changed file the scope stage dropped before any validator paired with it,
/// carrying the reason it was dropped.
///
/// An excluded file is never reviewed: it becomes no LLM (validator, file) pair
/// and is never an argument to a tool rule's `run` script. It is reported rather
/// than dropped in silence — the report names every one of them and its reason,
/// so a reader can tell a deliberate exclusion from a file the run missed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExcludedFile {
    /// The excluded file's repo-relative path.
    path: String,
    /// Why the scope stage dropped it, in the reader's words.
    reason: String,
    /// Which stage dropped it.
    kind: ExclusionKind,
}

impl ExcludedFile {
    /// The file dropped because it lives under a validator set's `fixtures/`
    /// directory.
    pub(crate) fn validator_fixture(path: &str) -> Self {
        Self {
            path: path.to_string(),
            reason: VALIDATOR_FIXTURE_REASON.to_string(),
            kind: ExclusionKind::ValidatorFixture,
        }
    }

    /// The file dropped because an ignore pattern matched it.
    ///
    /// `pattern` is the human string
    /// [`review_ignore_reason`](crate::review::ignore::review_ignore_reason)
    /// builds — the excluding glob and the ignore file it came from — and it is
    /// the key the report groups this kind of exclusion under.
    pub(crate) fn review_ignored(path: &str, pattern: String) -> Self {
        Self {
            path: path.to_string(),
            reason: pattern,
            kind: ExclusionKind::ReviewIgnore,
        }
    }

    /// The file dropped because no loaded validator's `match` accepted it.
    ///
    /// Unlike the other two, this is not an exclusion anyone asked for: the
    /// file simply has no coverage. Recording it is what keeps it out of the
    /// silence that makes an unread file's counts identical to a clean pass's.
    pub(crate) fn no_matching_validator(path: &str) -> Self {
        Self {
            path: path.to_string(),
            reason: NO_MATCHING_VALIDATOR_REASON.to_string(),
            kind: ExclusionKind::NoMatchingValidator,
        }
    }

    /// The excluded file's repo-relative path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Why the scope stage dropped it, in the reader's words.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Which stage dropped it.
    pub fn kind(&self) -> ExclusionKind {
        self.kind
    }
}
