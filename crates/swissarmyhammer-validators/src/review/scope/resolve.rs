//! Scope resolution — the deterministic git side of engine stage 1.
//!
//! Resolves a [`Scope`](super::Scope) selector into the changed-file set and
//! every input the later steps need: the sem-diff before/after content, the
//! review-level change purpose, and blame's history anchor. Every path is
//! confined to the repository root ([`confine_to_repo`]) and every scope's
//! candidate file set passes the same `.reviewignore` + `.gitignore` filter.
//! [`split_ignored`] applies that filter BEFORE it reads any content. Thus an
//! escaping or ignored path can never reach the review agent. Thus an ignore
//! pattern can also exclude a path the engine could not read at all.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use swissarmyhammer_git::GitOperations;
use swissarmyhammer_sem::git_types::{FileChange as SemFileChange, FileStatus};
use swissarmyhammer_sem::parser::plugins::code::is_code_file;

use ::ignore::gitignore::Gitignore;

use crate::error::AvpError;
use crate::review::ignore::{
    ensure_reviewignore, load_review_ignore_matcher, review_ignore_reason,
};

use super::excluded::ExcludedFile;
use super::{Scope, SCOPE_VALIDATOR};

/// The resolved scope: the changed-file set, the sem-diff inputs, the per-file
/// after-content, the review-level change purpose, and blame's history anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedScope {
    pub(super) files: Vec<String>,
    pub(super) file_changes: Vec<SemFileChange>,
    pub(super) after_content: BTreeMap<String, String>,
    pub(super) change_purpose: String,
    /// The commit blame's history walk is bounded to, mirroring `git blame
    /// <blame_at> -- path`. [`Scope::Working`], [`Scope::File`], and
    /// [`Scope::Glob`] set this to [`working_tree_blame_anchor`]'s merge-base
    /// pin (a stable anchor for the life of a branch), falling back to `None`
    /// (blame against HEAD) only when no such anchor exists. [`Scope::Sha`]
    /// sets this to the range's "to" commit so a bounded historical review
    /// never attributes a line to a commit past that point.
    pub(super) blame_at: Option<git2::Oid>,
}

/// The scope stage's reviewable file set, and the files it dropped before any
/// validator paired with them.
///
/// The two travel together because the report names both: what a run reviewed,
/// and what it did not. Their sum is how many files the scope reached. That
/// sum makes "every file in scope was excluded" a claim the engine can prove
/// rather than infer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScopeFiles {
    /// The reviewable file set: the candidates that survived the ignore filter
    /// and whose content the stage could read as text.
    pub(super) resolved: ResolvedScope,
    /// One entry per candidate the stage dropped, in candidate order. A
    /// dropped candidate is a path an ignore pattern matched, or a path whose
    /// bytes are not UTF-8 text.
    pub(super) excluded: Vec<ExcludedFile>,
}

/// Resolve a [`Scope`] to its changed-file set and the inputs every later step
/// needs (sem-diff `FileChange`s, after-content, change purpose), alongside the
/// files the stage excluded.
pub(super) fn resolve_scope_files(scope: &Scope, repo_path: &Path) -> Result<ScopeFiles, AvpError> {
    // Auto-generate `.reviewignore` (defaulting to `.kanban/`) on the first
    // review of any repo, never clobbering a user-edited one. It is untracked
    // and non-code, so it never enters the working scope resolved below.
    ensure_reviewignore(repo_path)?;

    // `resolve_scope_files` builds the matcher BEFORE any resolver runs. Each
    // resolver then applies it to its candidate paths BEFORE it reads their
    // content. See `split_ignored`. This stage used to read first. A pattern
    // could then not reach a path whose own read failed. A `*.png` line could
    // not exclude a tracked picture, because the blob read raised first.
    let matcher = load_review_ignore_matcher(repo_path)?;
    match scope {
        Scope::Working => resolve_working(repo_path, &matcher),
        Scope::Sha(range) => resolve_sha(repo_path, range, &matcher),
        Scope::File(path) => resolve_file(repo_path, path, &matcher),
        Scope::Glob(pattern) => resolve_glob(repo_path, pattern, &matcher),
    }
}

/// Split a scope's candidate paths into the ones this function keeps to read,
/// and one [`ExcludedFile`] per path the review-scope ignore `matcher`
/// excludes.
///
/// The uniform choke point every scope passes its candidates through. Thus the
/// stage drops a `.kanban/` board or a gitignored artifact identically. The
/// scope that named it, Working, Sha, File or Glob, makes no difference. The
/// filter runs BEFORE the content read. Thus a pattern can exclude a path the
/// engine cannot decode at all. [`confine_to_repo`] rejects an escaping path
/// independently and earlier. So this filter is about relevance, not
/// containment. This function only ever hands the matcher a repo-relative
/// path, which is what [`Gitignore::matched_path_or_any_parents`] requires.
///
/// A `Scope::File` naming an ignored path therefore resolves to a scope with
/// nothing left to review — consistent with the other scopes, never an error.
/// The exclusion is REPORTED rather than silent: the returned entry carries the
/// excluding pattern and the ignore file it came from, so a report over a fully
/// excluded scope names its own cause instead of reading as an empty scope.
/// Each excluded path is also logged at DEBUG with its FULL path and the
/// excluding pattern's source, never truncated.
fn split_ignored(files: Vec<String>, matcher: &Gitignore) -> (Vec<String>, Vec<ExcludedFile>) {
    let mut kept: Vec<String> = Vec::with_capacity(files.len());
    let mut ignored: Vec<ExcludedFile> = Vec::new();
    for path in files {
        match review_ignore_reason(matcher, &path) {
            Some(pattern) => {
                tracing::debug!(
                    path = %path,
                    pattern = %pattern,
                    "review scope: excluded ignored path"
                );
                ignored.push(ExcludedFile::review_ignored(&path, pattern));
            }
            None => kept.push(path),
        }
    }
    (kept, ignored)
}

/// Narrow `resolved` to the `kept` paths, keeping its three views of the scope
/// (paths, sem-diff inputs, after-content) mutually consistent.
///
/// The single place a scope-stage filter narrows an already-built
/// [`ResolvedScope`]. The validator-fixture split in [`super::fixtures`] uses
/// it. Thus no filter can drop a path from one view and leave it in another.
/// The ignore filter needs none of this. It runs on the candidate paths,
/// before a [`ResolvedScope`] exists at all.
pub(super) fn retain_scope_files(resolved: ResolvedScope, kept: Vec<String>) -> ResolvedScope {
    let ResolvedScope {
        files: _,
        file_changes,
        after_content,
        change_purpose,
        blame_at,
    } = resolved;

    let keep: BTreeSet<&str> = kept.iter().map(String::as_str).collect();
    let file_changes = file_changes
        .into_iter()
        .filter(|change| keep.contains(change.file_path.as_str()))
        .collect();
    let after_content = after_content
        .into_iter()
        .filter(|(path, _)| keep.contains(path.as_str()))
        .collect();

    ResolvedScope {
        files: kept,
        file_changes,
        after_content,
        change_purpose,
        blame_at,
    }
}

/// Open the repo, mapping git failures to [`AvpError::Context`].
pub(super) fn open_repo(repo_path: &Path) -> Result<GitOperations, AvpError> {
    GitOperations::with_work_dir(repo_path)
        .map_err(|e| AvpError::Context(format!("failed to open git repo: {e}")))
}

/// The [`AvpError::Validator`] raised for a scope path that resolves outside the
/// repository root. Carries the FULL, untruncated offending path so the caller
/// can see exactly what was rejected; the message is lowercase and unpunctuated.
pub(super) fn path_escapes_repo_root(path: &str) -> AvpError {
    AvpError::Validator {
        validator: SCOPE_VALIDATOR.to_string(),
        message: format!("path '{path}' escapes the repository root"),
    }
}

/// Lexically normalize an absolute path, resolving `.` and `..` components
/// WITHOUT touching the filesystem.
///
/// Used to contain a not-yet-existing candidate, which [`Path::canonicalize`]
/// cannot resolve (it requires every component to exist). A `..` that would
/// climb above the root pops past it, so the resulting path no longer starts
/// with the root and the containment check rejects it.
pub(super) fn normalize_lexically(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve a repo-relative scope `path` to an on-disk path guaranteed to lie
/// under `repo_path`, enforcing the review-scope containment contract.
///
/// Review paths are repo-relative by contract, so an absolute input is rejected
/// outright: [`Path::join`] with an absolute argument REPLACES the base
/// entirely, which would otherwise read an arbitrary file (e.g. `/etc/passwd`).
/// For a relative input the candidate is joined onto the canonicalized root,
/// then contained: an existing candidate is canonicalized (following symlinks,
/// so a link whose target escapes the root is caught), and a not-yet-existing
/// one is normalized lexically (preserving the absent-path `Ok(None)` behavior
/// its caller relies on). Any resolved path not under the root is rejected.
///
/// # Errors
///
/// [`AvpError::Validator`] via [`path_escapes_repo_root`] when `path` is
/// absolute or resolves outside the repository root; [`AvpError::Context`] when
/// the root or an existing candidate cannot be canonicalized.
pub(super) fn confine_to_repo(repo_path: &Path, path: &str) -> Result<PathBuf, AvpError> {
    if Path::new(path).is_absolute() {
        return Err(path_escapes_repo_root(path));
    }
    let root = repo_path.canonicalize().map_err(|e| {
        AvpError::Context(format!(
            "failed to canonicalize repo root {}: {e}",
            repo_path.display()
        ))
    })?;
    let candidate = root.join(path);
    let resolved = match candidate.canonicalize() {
        Ok(real) => real,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => normalize_lexically(&candidate),
        Err(e) => {
            return Err(AvpError::Context(format!(
                "failed to resolve working-tree file {path}: {e}"
            )))
        }
    };
    if !resolved.starts_with(&root) {
        return Err(path_escapes_repo_root(path));
    }
    // Return the RESOLVED (canonicalized-when-present) path rather than the raw
    // join, so the subsequent read does not re-walk symlinks — closing the
    // check-then-read TOCTOU window against a concurrent filesystem swap.
    Ok(resolved)
}

/// Read a path's working-tree content from disk, confined to the repo root.
///
/// The `path` is a repo-relative scope target; it is first resolved through
/// [`confine_to_repo`], which rejects any absolute input or `..`/symlink escape
/// so a `review file` caller can never make the pipeline read a file outside the
/// repository into the review agent's context.
///
/// Returns [`FileText::Absent`] only when the contained path is **absent**.
/// That is the intended deletion or added signal for a file gone from the
/// working tree. Returns [`FileText::NotUtf8`] when the file is there and
/// [`read_to_string`](std::fs::read_to_string) cannot decode it. The two
/// states stay apart. Thus the stage never silently diffs an unreadable
/// tracked file as wholly added or wholly removed. Its caller drops that file
/// out of scope and reports it instead. This function propagates any *other*
/// failure, a permission error for one, as [`AvpError::Context`]. A
/// containment violation surfaces as [`AvpError::Validator`].
///
/// # Errors
///
/// [`AvpError::Validator`] when `path` escapes the repository root (see
/// [`confine_to_repo`]); [`AvpError::Context`] for a read failure that is
/// neither an absent path nor an undecodable one.
pub(super) fn read_working(repo_path: &Path, path: &str) -> Result<FileText, AvpError> {
    let resolved = confine_to_repo(repo_path, path)?;
    match std::fs::read_to_string(resolved) {
        Ok(content) => Ok(FileText::Text(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(FileText::Absent),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            tracing::debug!(
                path = %path,
                "review scope: working-tree file is not UTF-8 text"
            );
            Ok(FileText::NotUtf8)
        }
        Err(e) => Err(AvpError::Context(format!(
            "failed to read working-tree file {path}: {e}"
        ))),
    }
}

/// A git refspec — the revision half of a `refspec:path` blob address. Any
/// commit-ish the engine reads content at: `HEAD` (see [`GitRefSpec::head`]), a
/// sha, a branch, a tag, `HEAD~3`.
///
/// Distinct from [`FilePath`] on purpose. Both halves of a blob address are
/// strings, so the compiler is the only thing that can stop a call site passing
/// them in the wrong order; giving each half its own type makes the
/// transposition a type error instead of a silent mis-read.
///
/// This is deliberately **not** [`swissarmyhammer_git::BranchName`], the
/// workspace's other git-string newtype: that type's validation rejects `~`,
/// `^`, `:` and `..` — exactly the syntax a refspec needs — so it can only hold
/// a refspec via `new_unchecked`, which would defeat the type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GitRefSpec(pub(super) String);

impl GitRefSpec {
    /// Wrap a commit-ish.
    pub(super) fn new(refspec: impl Into<String>) -> Self {
        Self(refspec.into())
    }

    /// The current checkout tip — the implicit "before" side of a working-tree or
    /// single-file scope, and the implicit "to" side of a bare-ref range. This is
    /// the single place the `HEAD` literal appears; every caller goes through it.
    pub(super) fn head() -> Self {
        Self("HEAD".to_string())
    }

    /// The refspec as libgit2 wants it.
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GitRefSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A repo-relative file path — the path half of a `refspec:path` blob address.
///
/// Distinct from [`GitRefSpec`] so the two halves cannot be transposed at a call
/// site; see that type for why the pair is typed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct FilePath(pub(super) String);

impl FilePath {
    /// Wrap a repo-relative path.
    pub(super) fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Unwrap the path for a consumer that stores it as a plain `String`.
    pub(super) fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Read a blob at `refspec:path` via libgit2.
///
/// This is the same `git show refspec:path` content read the git tool does, via
/// the shared `swissarmyhammer-git` repository handle instead of a shell-out.
///
/// The two halves of the address are separate types ([`GitRefSpec`],
/// [`FilePath`]) so no call site can transpose them.
///
/// Returns [`FileText::Absent`] only when the path does **not exist** at the
/// ref. That is the intended Added or Deleted signal. It happens when
/// `revparse_single` resolves to not-found, and when the object is not a blob.
/// Returns [`FileText::NotUtf8`] when the blob is there and its bytes are not
/// text. The two states stay apart. Thus the stage never silently diffs an
/// undecodable tracked file as wholly added or wholly removed. Its caller
/// drops that file out of scope and reports it instead. This function
/// propagates any other libgit2 failure as [`AvpError::Context`].
pub(super) fn read_at_ref(
    repo: &GitOperations,
    refspec: GitRefSpec,
    path: FilePath,
) -> Result<FileText, AvpError> {
    // The blob address. This function composes it one time. The read, the
    // failure message and the undecodable log all reuse it. Thus the
    // `refspec:path` form lives in a single place.
    let spec = format!("{refspec}:{path}");
    let inner = repo.repository().inner();
    let object = match inner.revparse_single(&spec) {
        Ok(object) => object,
        // The path is absent at this ref — the intended Added/Deleted signal.
        Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(FileText::Absent),
        Err(e) => return Err(AvpError::Context(format!("failed to resolve {spec}: {e}"))),
    };
    // Not a blob (e.g. a tree at that path) — there is no file content to read.
    let Some(blob) = object.as_blob() else {
        return Ok(FileText::Absent);
    };
    match String::from_utf8(blob.content().to_vec()) {
        Ok(text) => Ok(FileText::Text(text)),
        Err(e) => {
            tracing::debug!(
                spec = %spec,
                error = %e,
                "review scope: blob is not UTF-8 text"
            );
            Ok(FileText::NotUtf8)
        }
    }
}

/// One side of a file as the semantic diff takes it. The side holds text, or
/// holds no content at all, or holds bytes that are not text.
///
/// The third state keeps a binary file out of the diff and does not fail the
/// run. The `Option<String>` this replaces could carry neither half of that.
/// `None` already means the side holds no content. To fold an undecodable file
/// into `None` would hand the review a picture. The review would then read
/// that picture as wholly added or wholly removed work. The two readers above
/// used to raise instead. One tracked picture then threw the WHOLE run away,
/// report and all. A third state lets the caller drop that one file, and only
/// that file, into the report's not-reviewed list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FileText {
    /// The file's text at this revision.
    Text(String),
    /// The side holds no content.
    ///
    /// Two kinds of producer answer `Absent`, and they mean different things.
    /// A reader answers `Absent` when the file does not exist at the
    /// revision. That is the Added or Deleted signal. A resolver instead
    /// CHOOSES `Absent` for a base side, because its scope reviews whole
    /// content. The file can be present at that revision.
    ///
    /// The two resolvers reach that choice differently. `resolve_glob` reads
    /// no base side at all, and it gives every matched file `Absent`.
    /// `resolve_file` DOES read a base side, at HEAD. It keeps that side when
    /// the bytes differ from the working side. It discards that side, and
    /// records `Absent` instead, only when the two sides are equal.
    ///
    /// Both kinds mean one thing to the diff. There is no before text, so
    /// every entity in the file reads as added work. The variant states that
    /// one downstream fact, never the cause behind it.
    Absent,
    /// The file is there and its bytes are not UTF-8, so it carries no text.
    NotUtf8,
}

impl FileText {
    /// This side as the sem differ takes it. A side with no content gives
    /// `Some(None)`. A side that holds text gives `Some(Some(text))`. A side
    /// whose bytes are not text gives `None`.
    ///
    /// The outer option says whether a reader could READ the side. The inner
    /// one says whether the side holds any CONTENT. They are never the same
    /// question, which is exactly what a bare `Option<String>` could not say.
    fn text(self) -> Option<Option<String>> {
        match self {
            FileText::Text(text) => Some(Some(text)),
            FileText::Absent => Some(None),
            FileText::NotUtf8 => None,
        }
    }
}

/// The two sides of one file as [`FileChangeBuilder::push`] takes them, or
/// `None` when either side is not text.
///
/// The single place a [`FileText`] becomes a [`BeforeContent`] and
/// [`AfterContent`] pair. Thus no resolver can quietly turn an undecodable
/// side into an absent one. A caller that gets `None` has no content to
/// record, and must exclude the file instead. One undecodable side is enough.
/// The sem differ cannot diff a file that is binary at either revision.
fn readable_sides(before: FileText, after: FileText) -> Option<(BeforeContent, AfterContent)> {
    let before = BeforeContent::new(before.text()?);
    let after = AfterContent::new(after.text()?);
    Some((before, after))
}

/// Read the two sides of every candidate path, recording each on a
/// [`FileChangeBuilder`] and dropping the paths whose content is not text into
/// `excluded` instead.
///
/// The shared tail of all four resolvers. Each one computes its candidate set
/// and reads its two sides from a different place. Each one then records them
/// identically. `sides` reads one path's base and post-change content, so a
/// resolver's only job is to say where each side comes from.
///
/// Returns the paths this function really read, in the candidate order, beside
/// the builder holding their sem-diff inputs. A candidate missing from that
/// list is in `excluded`, so reviewed plus excluded still accounts for every
/// candidate.
fn read_candidates<F>(
    files: &[String],
    renames: &BTreeMap<String, String>,
    excluded: &mut Vec<ExcludedFile>,
    mut sides: F,
) -> Result<(Vec<String>, FileChangeBuilder), AvpError>
where
    F: FnMut(&str) -> Result<(FileText, FileText), AvpError>,
{
    let mut kept: Vec<String> = Vec::with_capacity(files.len());
    let mut builder = FileChangeBuilder::new();
    for path in files {
        let (before, after) = sides(path)?;
        let Some((before, after)) = readable_sides(before, after) else {
            tracing::debug!(
                path = %path,
                "review scope: excluded a file whose content is not UTF-8 text"
            );
            excluded.push(ExcludedFile::not_utf8(path));
            continue;
        };
        push_moved_file(&mut builder, path, before, after, renames);
        kept.push(path.clone());
    }
    Ok((kept, builder))
}

/// The rename/copy detection thresholds a review diff is found with, as git
/// itself spells them: a percentage of similarity, 0 to 100.
///
/// Git's own defaults — 50% for a rename, 50% for a copy — which is what
/// `git log -M -C` and `git status` already report a move by. Naming them here
/// rather than leaving `DiffFindOptions` on its library defaults keeps the
/// number the engine detects a move at readable beside the call that uses it.
const RENAME_SIMILARITY_PERCENT: u16 = 50;

/// A `new path -> old path` map of every file the diff found as a rename or a
/// copy of another.
///
/// Without this, a relocated file has no base side at its new path, so it
/// diffs as wholly ADDED: every line of it is marked as touched and reviewed
/// as new work. `^0fn6dbf` read as 70 files, 20548 insertions and 20097
/// deletions when its real delta was about 39 lines of module wiring. With
/// it, the base side is read from the OLD path, so a pure move reviews as the
/// nothing it changed, and a move-plus-edit reviews as the edit.
///
/// A detection failure degrades to an empty map — every file then resolves
/// exactly as it did before — rather than failing the scope: rename detection
/// makes a review proportionate, and is never what makes it correct.
fn find_renames(diff: &mut git2::Diff<'_>) -> BTreeMap<String, String> {
    let mut options = git2::DiffFindOptions::new();
    options
        .renames(true)
        .copies(true)
        .rename_threshold(RENAME_SIMILARITY_PERCENT)
        .copy_threshold(RENAME_SIMILARITY_PERCENT);
    if let Err(e) = diff.find_similar(Some(&mut options)) {
        tracing::warn!(
            error = %e,
            "review scope: rename detection failed; a moved file will diff as wholly added"
        );
        return BTreeMap::new();
    }

    let mut sources = BTreeMap::new();
    for delta in diff.deltas() {
        if !matches!(delta.status(), git2::Delta::Renamed | git2::Delta::Copied) {
            continue;
        }
        let (Some(old), Some(new)) = (path_of(delta.old_file()), path_of(delta.new_file())) else {
            continue;
        };
        if old != new {
            sources.insert(new, old);
        }
    }
    sources
}

/// One side of a diff delta as a repo-relative path string, `None` when the
/// side names no path or the path is not UTF-8.
fn path_of(file: git2::DiffFile<'_>) -> Option<String> {
    file.path()?.to_str().map(str::to_string)
}

/// The base-revision path a file's "before" content is read from: the path it
/// was moved from when the diff found it as a rename or a copy, else its own
/// path.
fn base_path<'a>(renames: &'a BTreeMap<String, String>, path: &'a str) -> &'a str {
    renames.get(path).map_or(path, String::as_str)
}

/// The [`FileVersions::moved_from`] a file carries: its rename/copy source when
/// the diff found one, `None` when it did not move.
fn moved_from(renames: &BTreeMap<String, String>, path: &str) -> Option<FilePath> {
    renames.get(path).map(FilePath::new)
}

/// Record one resolved file's two sides on `builder`, attaching the rename/copy
/// source `renames` holds for its path.
///
/// The shared tail of [`resolve_working`] and [`resolve_sha`]: each reads the
/// two sides from a different place and then records them identically, the
/// [`moved_from`] lookup included. Taking the map rather than a ready
/// [`FilePath`] keeps that lookup in one place, and keeps a second bare path
/// out of the argument list — the hazard [`FileVersions::moved_from`]
/// documents.
fn push_moved_file(
    builder: &mut FileChangeBuilder,
    path: &str,
    before: BeforeContent,
    after: AfterContent,
    renames: &BTreeMap<String, String>,
) {
    builder.push(
        FilePath::new(path),
        FileVersions {
            before,
            after,
            moved_from: moved_from(renames, path),
        },
    );
}

/// The rename/copy sources of a diff, or an empty map when building that diff
/// failed.
///
/// The shared tail of both rename lookups below: each builds a different diff
/// and then reads it the same way. `what` names the lookup in the warning, so a
/// degraded run still says which diff it lost.
fn renames_of_diff(
    diff: Result<git2::Diff<'_>, git2::Error>,
    what: &str,
) -> BTreeMap<String, String> {
    match diff {
        Ok(mut diff) => find_renames(&mut diff),
        Err(e) => {
            tracing::warn!(
                error = %e,
                diff = what,
                "review scope: diff failed; a moved file will diff as wholly added"
            );
            BTreeMap::new()
        }
    }
}

/// Every rename/copy source in the working tree, keyed by the file's current
/// path.
///
/// Diffs `HEAD`'s tree against the working tree WITH the index, and includes
/// untracked files, so a move is recognized whether it is committed, staged,
/// or only saved to disk.
fn working_rename_sources(repo: &GitOperations) -> BTreeMap<String, String> {
    let inner = repo.repository().inner();
    let Ok(head) = inner.head().and_then(|head| head.peel_to_tree()) else {
        // No HEAD (an empty repository): nothing to have moved from.
        return BTreeMap::new();
    };
    let mut options = git2::DiffOptions::new();
    options.include_untracked(true).recurse_untracked_dirs(true);
    renames_of_diff(
        inner.diff_tree_to_workdir_with_index(Some(&head), Some(&mut options)),
        "working tree",
    )
}

/// Every rename/copy source between two commit-ish endpoints, keyed by the
/// file's path at the `to` endpoint.
fn range_rename_sources(
    repo: &GitOperations,
    from: &GitRefSpec,
    to: &GitRefSpec,
) -> BTreeMap<String, String> {
    let inner = repo.repository().inner();
    let Some(from_tree) = revspec_tree(repo, from) else {
        return BTreeMap::new();
    };
    let Some(to_tree) = revspec_tree(repo, to) else {
        return BTreeMap::new();
    };
    renames_of_diff(
        inner.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None),
        "range",
    )
}

/// The tree a refspec points at, `None` when it cannot be resolved — the same
/// best-effort degradation the rest of rename detection uses.
fn revspec_tree<'a>(repo: &'a GitOperations, refspec: &GitRefSpec) -> Option<git2::Tree<'a>> {
    let inner = repo.repository().inner();
    let object = inner.revparse_single(refspec.as_str()).ok()?;
    object.peel_to_commit().ok()?.tree().ok()
}

/// Resolve the working-tree scope: uncommitted changes vs HEAD (staged +
/// unstaged + untracked), reusing the git tool's changed-file accounting.
pub(super) fn resolve_working(
    repo_path: &Path,
    matcher: &Gitignore,
) -> Result<ScopeFiles, AvpError> {
    let repo = open_repo(repo_path)?;
    let status = repo
        .get_status()
        .map_err(|e| AvpError::Context(format!("failed to read git status: {e}")))?;
    // Tracked changes (deliberate edits) keep current behavior — per-validator
    // globs decide what's reviewed. UNTRACKED entries are filtered to code files
    // via the canonical `swissarmyhammer-sem` extension list: brand-new source
    // gets reviewed because it WILL be added, while unignored junk (logs, jsonl,
    // lockfiles) never has its content read into scope.
    let mut candidates = status.all_changed_files();
    candidates.extend(status.untracked.iter().filter(|p| is_code_file(p)).cloned());
    candidates.sort();
    candidates.dedup();

    let (candidates, mut excluded) = split_ignored(candidates, matcher);

    // Recognize a move as a move: a file relocated in the working tree reads
    // its base side from the path it came from, so its diff is the edit it
    // carries rather than the whole file over again.
    let renames = working_rename_sources(&repo);

    // This closure reads each candidate's working-tree content one time. A
    // file with no content is a deletion. It reads as absent, so the sem
    // differ treats it as a deletion.
    let (files, builder) = read_candidates(&candidates, &renames, &mut excluded, |path| {
        let after = read_working(repo_path, path)?;
        let base = base_path(&renames, path);
        let before = read_at_ref(&repo, GitRefSpec::head(), FilePath::new(base))?;
        Ok((before, after))
    })?;

    // Blame anchor: pinned to the branch's merge-base with main/master (see
    // `working_tree_blame_anchor`) so the sha column means the same thing on
    // every run for the life of this branch, rather than drifting with every
    // intervening commit. `None` when no stable anchor exists (falls back to
    // HEAD, the pre-existing behavior).
    Ok(ScopeFiles {
        resolved: builder.finish(
            files,
            auto_purpose("working-tree changes"),
            working_tree_blame_anchor(&repo),
        ),
        excluded,
    })
}

/// Resolve a commit/range scope, reusing the git tool's range semantics
/// (`from..to`, or a single ref treated as `ref..HEAD`).
pub(super) fn resolve_sha(
    repo_path: &Path,
    range: &str,
    matcher: &Gitignore,
) -> Result<ScopeFiles, AvpError> {
    let repo = open_repo(repo_path)?;
    let candidates = repo
        .get_changed_files_from_range(range)
        .map_err(|e| AvpError::Context(format!("failed to resolve range '{range}': {e}")))?;

    let (from_ref, to_ref) = match range.split_once("..") {
        Some((from, to)) => (GitRefSpec::new(from), GitRefSpec::new(to)),
        None => (GitRefSpec::new(range), GitRefSpec::head()),
    };

    let (candidates, mut excluded) = split_ignored(candidates, matcher);

    // Recognize a move as a move: a file relocated within the range reads its
    // base side from the path it came from, so a relocation commit reviews in
    // proportion to its real delta rather than as thousands of added lines.
    let renames = range_rename_sources(&repo, &from_ref, &to_ref);

    let (files, builder) = read_candidates(&candidates, &renames, &mut excluded, |path| {
        let base = base_path(&renames, path);
        let before = read_at_ref(&repo, from_ref.clone(), FilePath::new(base))?;
        let after = read_at_ref(&repo, to_ref.clone(), FilePath::new(path))?;
        Ok((before, after))
    })?;

    let purpose = commit_messages(&repo, &to_ref)
        .unwrap_or_else(|| auto_purpose(&format!("changes in range {range}")));
    // Bound blame to the range's "to" endpoint: a historical review must
    // never attribute a line to a commit past the point it reviews.
    Ok(ScopeFiles {
        resolved: builder.finish(files, purpose, resolve_oid(&repo, &to_ref)),
        excluded,
    })
}

/// Resolve a single-file scope: its working-tree changes if any, else its whole
/// content reviewed as all-added work.
///
/// `path` is repo-relative by contract. [`confine_to_repo`] contains it BEFORE
/// anything else reads it. Thus the stage rejects a `review file` target that
/// is absolute, or that escapes the repository root through `..` or a symlink.
/// It rejects that target with [`AvpError::Validator`], and never reads its
/// content into scope. Containment first also keeps the ignore matcher's own
/// repo-relative precondition true. This is the one scope whose path a caller
/// names.
pub(super) fn resolve_file(
    repo_path: &Path,
    path: &str,
    matcher: &Gitignore,
) -> Result<ScopeFiles, AvpError> {
    confine_to_repo(repo_path, path)?;
    let repo = open_repo(repo_path)?;

    let (candidates, mut excluded) = split_ignored(vec![path.to_string()], matcher);

    // This scope reviews a single named file whole, so there is no move to
    // recognize. Its base side is its own path at HEAD, or nothing.
    let renames = BTreeMap::new();
    let (files, builder) = read_candidates(&candidates, &renames, &mut excluded, |candidate| {
        let working = read_working(repo_path, candidate)?;
        let head = read_at_ref(&repo, GitRefSpec::head(), FilePath::new(candidate))?;
        // A file with no working-tree change carries NO base side. Every
        // entity in it then diffs as added work. That is the same
        // whole-content shape `resolve_glob` gives each of its files.
        //
        // `FileText::Absent` states that missing base side. Here it does NOT
        // mean the file is gone from HEAD, because the file IS at HEAD. This
        // scope reviews whole content, so it chooses to hold no base at all.
        //
        // Keeping the identical HEAD revision here would leave the semantic
        // diff empty, and an entity-bound probe with no entity emits no result
        // at all. A candidate-probe validator cannot tell that silence apart
        // from "the probe ran and found no candidates". So it judges from
        // nothing and answers differently on each run.
        let before = if head == working {
            FileText::Absent
        } else {
            head
        };
        Ok((before, working))
    })?;

    // Blame anchor: same stable merge-base pin as `resolve_working` — see
    // `working_tree_blame_anchor`.
    Ok(ScopeFiles {
        resolved: builder.finish(
            files,
            auto_purpose(&format!("review of {path}")),
            working_tree_blame_anchor(&repo),
        ),
        excluded,
    })
}

/// Resolve a glob scope: every matching tracked file as whole-content work (no
/// before side, so each diffs as all-added).
pub(super) fn resolve_glob(
    repo_path: &Path,
    pattern: &str,
    matcher: &Gitignore,
) -> Result<ScopeFiles, AvpError> {
    let compiled = glob::Pattern::new(pattern).map_err(|e| AvpError::Validator {
        validator: SCOPE_VALIDATOR.to_string(),
        message: format!("invalid glob pattern '{pattern}': {e}"),
    })?;

    let repo = open_repo(repo_path)?;
    let tracked = repo
        .get_all_tracked_files()
        .map_err(|e| AvpError::Context(format!("failed to list tracked files: {e}")))?;
    let candidates: Vec<String> = tracked
        .into_iter()
        .filter(|f| compiled.matches_with(f, crate::validators::GLOB_MATCH_OPTIONS))
        .collect();

    let (candidates, mut excluded) = split_ignored(candidates, matcher);

    // A glob scope has no base side. Every matched file diffs as all-added.
    // Nothing moved, so there is no rename source to look up either.
    //
    // `FileText::Absent` states that missing base side below. Here it does NOT
    // mean the file is gone from HEAD, because every matched file is tracked
    // and present. This scope reviews whole content, so it chooses to hold no
    // base at all.
    let renames = BTreeMap::new();
    let (files, builder) = read_candidates(&candidates, &renames, &mut excluded, |path| {
        Ok((FileText::Absent, read_working(repo_path, path)?))
    })?;

    // Blame anchor: same stable merge-base pin as `resolve_working` — see
    // `working_tree_blame_anchor`.
    Ok(ScopeFiles {
        resolved: builder.finish(
            files,
            auto_purpose(&format!("files matching {pattern}")),
            working_tree_blame_anchor(&repo),
        ),
        excluded,
    })
}

/// Wrap a one-line auto summary as the review-level change purpose.
pub(super) fn auto_purpose(what: &str) -> String {
    format!("Auto summary: reviewing {what}.")
}

/// The stable blame anchor for a working-tree-backed scope ([`Scope::Working`],
/// [`Scope::File`], [`Scope::Glob`]): the merge-base between `HEAD` and the
/// detected `main`/`master` branch.
///
/// Those three scopes read the file's LIVE working-tree content, which can
/// change shape (dirty → committed, tracked → staged) between two runs
/// without the underlying finding changing at all — a `/finish`-style loop
/// commits between iterations (`git add -A && git commit`), which sweeps up
/// every dirty file, not just the one whose finding it resolved. Binding
/// blame to `HEAD` (as `None` does) means every such commit — even one that
/// never touches the file under review — moves the anchor forward, so the
/// SAME still-open, byte-identical line flips from `worktree` to a real
/// commit sha the moment ANY intervening commit lands.
///
/// The merge-base with `main`/`master` does not move for the life of a
/// feature/task branch (main only moves if someone advances it, which a
/// `/finish` loop never does): every commit the loop makes lands strictly
/// AFTER this anchor, so blame bounded here never sees them — a line that
/// is `worktree` on the branch's first review stays `worktree` on every
/// later review, for as long as it postdates the anchor, regardless of how
/// many intervening commits happen. The column then answers one fixed
/// question all session long: "did this line exist before this unit of work
/// started?" — never "what does HEAD say right now?"
///
/// Falls back to `None` (blame against `HEAD`, the pre-existing behavior)
/// when no `main`/`master` branch exists, `HEAD` cannot be resolved, or the
/// two share no common ancestor — including the case of reviewing directly
/// ON `main` itself, where the merge-base IS `HEAD` and this degrades
/// transparently to the old per-run behavior. Blame attribution is always
/// best-effort, never load-bearing for the review itself.
pub(super) fn working_tree_blame_anchor(repo: &GitOperations) -> Option<git2::Oid> {
    let main_branch = repo.main_branch().ok()?;
    let head_oid = resolve_oid(repo, &GitRefSpec::head())?;
    let main_oid = resolve_oid(repo, &GitRefSpec::new(main_branch))?;
    repo.repository()
        .inner()
        .merge_base(head_oid, main_oid)
        .ok()
}

/// Resolve a refspec to its commit [`git2::Oid`] via libgit2, `None` when
/// unresolvable — the blame anchor [`resolve_sha`] binds a bounded historical
/// review's blame calls to. An unresolvable ref degrades to `None` (blame
/// against HEAD) rather than failing the whole scope resolution: blame
/// attribution is best-effort, never load-bearing for the review itself.
pub(super) fn resolve_oid(repo: &GitOperations, refspec: &GitRefSpec) -> Option<git2::Oid> {
    let inner = repo.repository().inner();
    let object = inner.revparse_single(refspec.as_str()).ok()?;
    object.peel_to_commit().ok().map(|c| c.id())
}

/// Read the commit message for a ref via libgit2, `None` when unresolvable.
pub(super) fn commit_messages(repo: &GitOperations, refspec: &GitRefSpec) -> Option<String> {
    let inner = repo.repository().inner();
    let object = inner.revparse_single(refspec.as_str()).ok()?;
    let commit = object.peel_to_commit().ok()?;
    let message = commit.message().unwrap_or("").trim().to_string();
    if message.is_empty() {
        None
    } else {
        Some(message)
    }
}

/// A file's content at the **base** revision of the change — `None` when the
/// file did not exist there (the Added signal).
///
/// Distinct from [`AfterContent`] on purpose, and the sharper case of the same
/// hazard as [`GitRefSpec`]/[`FilePath`]: both sides are `Option<String>` and
/// they arrive together at [`FileChangeBuilder::push`], so nothing but the
/// compiler can stop a call site swapping them — and a swap does not fail
/// loudly, it flips [`FileStatus::Added`] to [`FileStatus::Deleted`] and hands
/// the review a plausible-looking INVERTED diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BeforeContent(pub(super) Option<String>);

impl BeforeContent {
    /// Wrap the base-revision content of a file.
    pub(super) fn new(content: Option<String>) -> Self {
        Self(content)
    }

    /// Unwrap for the sem-diff input.
    pub(super) fn into_inner(self) -> Option<String> {
        self.0
    }
}

/// A file's content **after** the change — `None` when the file no longer
/// exists (the Deleted signal).
///
/// Distinct from [`BeforeContent`] so the two sides cannot be transposed; see
/// that type for what a transposition would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AfterContent(pub(super) Option<String>);

impl AfterContent {
    /// Wrap the post-change content of a file.
    pub(super) fn new(content: Option<String>) -> Self {
        Self(content)
    }

    /// Unwrap for the sem-diff input.
    pub(super) fn into_inner(self) -> Option<String> {
        self.0
    }
}

/// Both sides of one file's change, named rather than positional.
///
/// [`FileChangeBuilder::push`] takes this single argument instead of two
/// `Option<String>`s: the fields name each side at the call site, and their
/// distinct types ([`BeforeContent`], [`AfterContent`]) make a transposed
/// struct literal a compile error rather than an inverted diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FileVersions {
    /// The content at the base revision.
    pub(super) before: BeforeContent,
    /// The content after the change.
    pub(super) after: AfterContent,
    /// The path the base side was read from when git found this file as a
    /// rename or a copy of another — `None` when the file did not move.
    ///
    /// It lives beside the two contents rather than as a third argument to
    /// [`FileChangeBuilder::push`] because it names where `before` came from,
    /// and because a second bare [`FilePath`] parameter beside the destination
    /// path would be exactly the transposition the typed halves exist to
    /// prevent.
    pub(super) moved_from: Option<FilePath>,
}

/// Accumulates the per-file sem-diff inputs and after-content as files resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FileChangeBuilder {
    pub(super) file_changes: Vec<SemFileChange>,
    pub(super) after_content: BTreeMap<String, String>,
}

impl FileChangeBuilder {
    /// Create a new, empty [`FileChangeBuilder`].
    pub(super) fn new() -> Self {
        Self {
            file_changes: Vec::new(),
            after_content: BTreeMap::new(),
        }
    }

    /// Record one file's before/after content for the sem differ.
    ///
    /// The two sides arrive as one named-field [`FileVersions`], so they cannot
    /// be transposed into an inverted diff.
    ///
    /// A file git found as a rename or a copy carries its source path in
    /// [`FileVersions::moved_from`], which becomes the change's
    /// `old_file_path`. Its status stays [`FileStatus::Modified`] rather than
    /// `Added`, because the base side really was read — from the old path —
    /// and a move is an edit of content that already existed.
    pub(super) fn push(&mut self, path: FilePath, versions: FileVersions) {
        let FileVersions {
            before,
            after,
            moved_from,
        } = versions;
        let (before, after) = (before.into_inner(), after.into_inner());
        let path = path.into_string();
        if let Some(content) = &after {
            self.after_content.insert(path.clone(), content.clone());
        }
        let status = match (&before, &after) {
            (None, Some(_)) => FileStatus::Added,
            (Some(_), None) => FileStatus::Deleted,
            _ => FileStatus::Modified,
        };
        self.file_changes.push(SemFileChange {
            file_path: path,
            status,
            old_file_path: moved_from.map(FilePath::into_string),
            before_content: before,
            after_content: after,
        });
    }

    /// Finish into a [`ResolvedScope`]. `blame_at` is the commit blame's
    /// history walk is bounded to (see [`ResolvedScope::blame_at`]).
    pub(super) fn finish(
        self,
        files: Vec<String>,
        change_purpose: String,
        blame_at: Option<git2::Oid>,
    ) -> ResolvedScope {
        ResolvedScope {
            files,
            file_changes: self.file_changes,
            after_content: self.after_content,
            change_purpose,
            blame_at,
        }
    }
}
