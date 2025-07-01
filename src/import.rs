use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use fs_extra::dir::{copy, CopyOptions};
use git2::Repository;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
};
use tar::Archive;
use url::Url;
use zip::ZipArchive;

use crate::cli::ImportStrategy;
use crate::export::{ExportManifest, PromptMetadata};
use crate::prompts::{PromptLoader, PromptStorage};

pub async fn run_import_command(
    source: String,
    dry_run: bool,
    strategy: ImportStrategy,
    target: Option<String>,
    validate: bool,
    backup: bool,
    verbose: bool,
) -> Result<()> {
    let importer = Importer::new(target, strategy, validate, backup, verbose)?;
    
    if dry_run {
        println!("🔍 Dry run mode: showing what would be imported without making changes");
    }
    
    importer.import_from_source(&source, dry_run).await?;
    
    Ok(())
}

pub struct Importer {
    target_dir: PathBuf,
    strategy: ImportStrategy,
    validate: bool,
    backup: bool,
    verbose: bool,
    storage: PromptStorage,
}

impl Importer {
    pub fn new(
        target: Option<String>,
        strategy: ImportStrategy,
        validate: bool,
        backup: bool,
        verbose: bool,
    ) -> Result<Self> {
        let target_dir = if let Some(target) = target {
            PathBuf::from(target)
        } else {
            dirs::home_dir()
                .context("Could not find home directory")?
                .join(".swissarmyhammer")
                .join("prompts")
        };
        
        let storage = PromptStorage::new();
        
        Ok(Self {
            target_dir,
            strategy,
            validate,
            backup,
            verbose,
            storage,
        })
    }
    
    pub async fn import_from_source(&self, source: &str, dry_run: bool) -> Result<()> {
        if self.verbose {
            println!("📦 Importing from source: {}", source);
        }
        
        let temp_dir = tempfile::tempdir()?;
        let extracted_path = if self.is_url(source) {
            self.download_and_extract(source, temp_dir.path()).await?
        } else if self.is_git_repo(source) {
            self.clone_git_repo(source, temp_dir.path()).await?
        } else {
            self.extract_local_archive(source, temp_dir.path()).await?
        };
        
        let manifest = self.read_manifest(&extracted_path).await?;
        let prompts = self.discover_prompts(&extracted_path, &manifest).await?;
        
        if self.verbose {
            println!("📋 Found {} prompts to import", prompts.len());
        }
        
        if prompts.is_empty() {
            println!("⚠️  No prompts found in the source");
            return Ok(());
        }
        
        // Show preview
        self.show_import_preview(&prompts, &manifest)?;
        
        if !dry_run {
            if self.backup {
                self.create_backup().await?;
            }
            
            let conflicts = self.detect_conflicts(&prompts).await?;
            if !conflicts.is_empty() {
                self.handle_conflicts(&conflicts, &prompts).await?;
            }
            
            self.install_prompts(&prompts).await?;
            println!("✅ Successfully imported {} prompts", prompts.len());
        } else {
            println!("🔍 Dry run completed - no changes made");
        }
        
        Ok(())
    }
    
    fn is_url(&self, source: &str) -> bool {
        Url::parse(source).is_ok()
    }
    
    fn is_git_repo(&self, source: &str) -> bool {
        source.ends_with(".git") || 
        source.starts_with("git@") ||
        source.contains("github.com") ||
        source.contains("gitlab.com")
    }
    
    async fn download_and_extract(&self, url: &str, temp_dir: &Path) -> Result<PathBuf> {
        if self.verbose {
            println!("⬇️  Downloading from {}", url);
        }
        
        let client = Client::new();
        let response = client.get(url).send().await?;
        
        let total_size = response.content_length().unwrap_or(0);
        let progress = ProgressBar::new(total_size);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        progress.set_message("Downloading");
        
        let archive_path = temp_dir.join("archive");
        let mut file = File::create(&archive_path)?;
        let mut downloaded = 0u64;
        
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded = std::cmp::min(downloaded + chunk.len() as u64, total_size);
            progress.set_position(downloaded);
        }
        
        progress.finish_with_message("Download completed");
        
        self.extract_archive(&archive_path, temp_dir).await
    }
    
    async fn clone_git_repo(&self, repo_url: &str, temp_dir: &Path) -> Result<PathBuf> {
        if self.verbose {
            println!("📥 Cloning Git repository from {}", repo_url);
        }
        
        let clone_path = temp_dir.join("repo");
        Repository::clone(repo_url, &clone_path)
            .with_context(|| format!("Failed to clone repository: {}", repo_url))?;
        
        // Look for prompts in common subdirectories
        let search_paths = ["prompts", ".", "src", "templates"];
        for subdir in &search_paths {
            let potential_path = clone_path.join(subdir);
            if potential_path.exists() && self.has_markdown_files(&potential_path)? {
                return Ok(potential_path);
            }
        }
        
        Ok(clone_path)
    }
    
    async fn extract_local_archive(&self, archive_path: &str, temp_dir: &Path) -> Result<PathBuf> {
        if self.verbose {
            println!("📂 Extracting local archive {}", archive_path);
        }
        
        let path = PathBuf::from(archive_path);
        self.extract_archive(&path, temp_dir).await
    }
    
    async fn extract_archive(&self, archive_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
        let extract_path = temp_dir.join("extracted");
        create_dir_all(&extract_path)?;
        
        let extension = archive_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension {
            "gz" | "tgz" => {
                let file = File::open(archive_path)?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                archive.unpack(&extract_path)?;
            }
            "zip" => {
                let file = File::open(archive_path)?;
                let mut archive = ZipArchive::new(file)?;
                archive.extract(&extract_path)?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported archive format: {}", extension));
            }
        }
        
        Ok(extract_path)
    }
    
    fn has_markdown_files(&self, dir: &Path) -> Result<bool> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    async fn read_manifest(&self, dir: &Path) -> Result<Option<ExportManifest>> {
        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            let content = std::fs::read_to_string(manifest_path)?;
            let manifest: ExportManifest = serde_json::from_str(&content)?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }
    
    async fn discover_prompts(&self, dir: &Path, manifest: &Option<ExportManifest>) -> Result<Vec<ImportPrompt>> {
        let mut prompts = Vec::new();
        
        if let Some(manifest) = manifest {
            // Use manifest information
            for prompt_meta in &manifest.prompts {
                let file_path = dir.join(&prompt_meta.path);
                if file_path.exists() {
                    prompts.push(ImportPrompt {
                        name: prompt_meta.name.clone(),
                        file_path,
                        metadata: Some(prompt_meta.clone()),
                    });
                }
            }
        } else {
            // Discover prompts by scanning directory
            self.scan_directory_for_prompts(dir, &mut prompts)?;
        }
        
        Ok(prompts)
    }
    
    fn scan_directory_for_prompts(&self, dir: &Path, prompts: &mut Vec<ImportPrompt>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    prompts.push(ImportPrompt {
                        name: name.to_string(),
                        file_path: path,
                        metadata: None,
                    });
                }
            } else if path.is_dir() {
                self.scan_directory_for_prompts(&path, prompts)?;
            }
        }
        Ok(())
    }
    
    fn show_import_preview(&self, prompts: &[ImportPrompt], manifest: &Option<ExportManifest>) -> Result<()> {
        println!("\n📋 Import Preview:");
        println!("{}", "─".repeat(60));
        
        if let Some(manifest) = manifest {
            println!("📦 Package: {} (v{})", 
                manifest.description.as_deref().unwrap_or("Unnamed package"),
                manifest.version
            );
            if let Some(author) = &manifest.author {
                println!("👤 Author: {}", author);
            }
            println!("📅 Created: {}", manifest.created_at);
            println!();
        }
        
        for prompt in prompts {
            print!("  • {}", prompt.name);
            if let Some(metadata) = &prompt.metadata {
                if let Some(title) = &metadata.title {
                    print!(" - {}", title);
                }
                print!(" ({})", metadata.source);
            }
            println!();
        }
        
        println!("{}", "─".repeat(60));
        Ok(())
    }
    
    async fn detect_conflicts(&self, prompts: &[ImportPrompt]) -> Result<Vec<String>> {
        let mut conflicts = Vec::new();
        
        // Load existing prompts
        let mut loader = PromptLoader::new();
        loader.storage = self.storage.clone();
        loader.load_all()?;
        
        for prompt in prompts {
            if self.storage.get(&prompt.name).is_some() {
                conflicts.push(prompt.name.clone());
            }
        }
        
        if !conflicts.is_empty() && self.verbose {
            println!("⚠️  Found {} conflicts: {:?}", conflicts.len(), conflicts);
        }
        
        Ok(conflicts)
    }
    
    async fn handle_conflicts(&self, conflicts: &[String], _prompts: &[ImportPrompt]) -> Result<()> {
        match self.strategy {
            ImportStrategy::Skip => {
                println!("⏭️  Skipping {} conflicting prompts", conflicts.len());
            }
            ImportStrategy::Overwrite => {
                println!("🔄 Overwriting {} existing prompts", conflicts.len());
            }
            ImportStrategy::Rename => {
                println!("📝 Renaming {} conflicting prompts", conflicts.len());
            }
            ImportStrategy::Merge => {
                println!("🔀 Merging {} conflicting prompts", conflicts.len());
            }
        }
        Ok(())
    }
    
    async fn create_backup(&self) -> Result<()> {
        if !self.target_dir.exists() {
            return Ok(());
        }
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_dir = self.target_dir.parent()
            .unwrap_or(&self.target_dir)
            .join(format!("prompts_backup_{}", timestamp));
        
        if self.verbose {
            println!("💾 Creating backup at {}", backup_dir.display());
        }
        
        let mut options = CopyOptions::new();
        options.overwrite = true;
        copy(&self.target_dir, &backup_dir, &options)?;
        
        Ok(())
    }
    
    async fn install_prompts(&self, prompts: &[ImportPrompt]) -> Result<()> {
        create_dir_all(&self.target_dir)?;
        
        let progress = ProgressBar::new(prompts.len() as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        for prompt in prompts {
            let target_name = self.resolve_name_conflict(&prompt.name).await?;
            let target_path = self.target_dir.join(format!("{}.md", target_name));
            
            if self.validate {
                self.validate_prompt(&prompt.file_path)?;
            }
            
            std::fs::copy(&prompt.file_path, target_path)?;
            progress.set_message(format!("Installing {}", prompt.name));
            progress.inc(1);
        }
        
        progress.finish_with_message("Installation completed");
        Ok(())
    }
    
    async fn resolve_name_conflict(&self, name: &str) -> Result<String> {
        match self.strategy {
            ImportStrategy::Skip => {
                if self.storage.get(name).is_some() {
                    return Err(anyhow::anyhow!("Prompt {} already exists and strategy is skip", name));
                }
                Ok(name.to_string())
            }
            ImportStrategy::Overwrite => Ok(name.to_string()),
            ImportStrategy::Rename => {
                if self.storage.get(name).is_some() {
                    let mut counter = 1;
                    loop {
                        let new_name = format!("{}-{}", name, counter);
                        if self.storage.get(&new_name).is_none() {
                            return Ok(new_name);
                        }
                        counter += 1;
                    }
                } else {
                    Ok(name.to_string())
                }
            }
            ImportStrategy::Merge => {
                // For now, merge strategy is the same as overwrite
                // TODO: Implement intelligent merging of metadata
                Ok(name.to_string())
            }
        }
    }
    
    fn validate_prompt(&self, _path: &Path) -> Result<()> {
        if !self.validate {
            return Ok(());
        }
        
        // TODO: Implement file-level validation once public API is available
        // For now, just check if the file exists and has .md extension
        if !_path.exists() {
            return Err(anyhow::anyhow!("Prompt file does not exist: {}", _path.display()));
        }
        
        if _path.extension().and_then(|s| s.to_str()) != Some("md") {
            return Err(anyhow::anyhow!("Prompt file must have .md extension: {}", _path.display()));
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct ImportPrompt {
    pub name: String,
    pub file_path: PathBuf,
    pub metadata: Option<PromptMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_importer_creation() {
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false);
        assert!(importer.is_ok());
    }

    #[test]
    fn test_is_url() {
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false).unwrap();
        
        assert!(importer.is_url("https://example.com/prompts.tar.gz"));
        assert!(importer.is_url("http://example.com/prompts.zip"));
        assert!(!importer.is_url("local-file.tar.gz"));
        assert!(!importer.is_url("git@github.com:user/repo.git"));
    }

    #[test]
    fn test_is_git_repo() {
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false).unwrap();
        
        assert!(importer.is_git_repo("git@github.com:user/repo.git"));
        assert!(importer.is_git_repo("https://github.com/user/repo.git"));
        assert!(importer.is_git_repo("https://gitlab.com/user/repo"));
        assert!(!importer.is_git_repo("https://example.com/archive.tar.gz"));
    }

    #[test]
    fn test_has_markdown_files() {
        let temp_dir = TempDir::new().unwrap();
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false).unwrap();
        
        // Empty directory
        assert!(!importer.has_markdown_files(temp_dir.path()).unwrap());
        
        // Directory with markdown file
        std::fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
        assert!(importer.has_markdown_files(temp_dir.path()).unwrap());
    }

    #[tokio::test]
    async fn test_resolve_name_conflict() {
        let importer = Importer::new(None, ImportStrategy::Rename, true, true, false).unwrap();
        
        // Non-conflicting name
        let result = importer.resolve_name_conflict("new-prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "new-prompt");
    }

    #[tokio::test] 
    async fn test_discover_prompts_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false).unwrap();
        
        let prompts = importer.discover_prompts(temp_dir.path(), &None).await;
        assert!(prompts.is_ok());
        assert!(prompts.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_discover_prompts_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let importer = Importer::new(None, ImportStrategy::Skip, true, true, false).unwrap();
        
        // Create test markdown files
        std::fs::write(temp_dir.path().join("prompt1.md"), "# Prompt 1").unwrap();
        std::fs::write(temp_dir.path().join("prompt2.md"), "# Prompt 2").unwrap();
        std::fs::write(temp_dir.path().join("readme.txt"), "Not a prompt").unwrap();
        
        let prompts = importer.discover_prompts(temp_dir.path(), &None).await;
        assert!(prompts.is_ok());
        
        let prompts = prompts.unwrap();
        assert_eq!(prompts.len(), 2);
        assert!(prompts.iter().any(|p| p.name == "prompt1"));
        assert!(prompts.iter().any(|p| p.name == "prompt2"));
    }
}