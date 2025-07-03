use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tar::Builder;
use zip::write::{FileOptions, ZipWriter};

use crate::cli::{ExportFormat, PromptSource};
use crate::prompt_loader::PromptResolver;
use swissarmyhammer::PromptLibrary;

#[derive(Serialize, Deserialize)]
pub struct ExportManifest {
    pub version: String,
    pub format_version: String,
    pub created_at: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub prompts: Vec<PromptMetadata>,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptMetadata {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub category: Option<String>,
    pub path: String,
    pub size: u64,
    pub checksum: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn run_export_command(
    prompt_name: Option<String>,
    all: bool,
    category: Option<String>,
    source_filter: Option<PromptSource>,
    format: ExportFormat,
    output: Option<String>,
    metadata: bool,
    exclude: Vec<String>,
) -> Result<()> {
    let exporter = Exporter::new()?;

    let prompts = exporter.collect_prompts(prompt_name, all, category, source_filter, &exclude)?;

    if prompts.is_empty() {
        println!("No prompts found matching the criteria.");
        return Ok(());
    }

    let output_path = determine_output_path(&output, &format)?;

    match format {
        ExportFormat::TarGz => {
            exporter
                .export_tar_gz(&prompts, &output_path, metadata)
                .await?;
        }
        ExportFormat::Zip => {
            exporter
                .export_zip(&prompts, &output_path, metadata)
                .await?;
        }
        ExportFormat::Directory => {
            exporter
                .export_directory(&prompts, &output_path, metadata)
                .await?;
        }
    }

    println!(
        "Successfully exported {} prompts to {}",
        prompts.len(),
        output_path.display()
    );

    Ok(())
}

pub struct Exporter {
    library: PromptLibrary,
    prompt_sources: HashMap<String, PromptSource>,
}

impl Exporter {
    pub fn new() -> Result<Self> {
        let mut library = PromptLibrary::new();
        let mut resolver = PromptResolver::new();
        resolver.load_all_prompts(&mut library)?;
        Ok(Self {
            library,
            prompt_sources: resolver.prompt_sources,
        })
    }

    pub fn collect_prompts(
        &self,
        prompt_name: Option<String>,
        all: bool,
        category: Option<String>,
        source_filter: Option<PromptSource>,
        exclude: &[String],
    ) -> Result<Vec<(String, PathBuf)>> {
        let mut prompts = Vec::new();

        for prompt in self.library.list()? {
            // Apply filters
            if let Some(ref filter_name) = prompt_name {
                if prompt.name != filter_name.as_str() {
                    continue;
                }
            }

            if !all && prompt_name.is_none() && category.is_none() {
                continue;
            }

            if let Some(ref filter_category) = category {
                // TODO: Implement category filtering when category is added to Prompt struct
                // For now, skip category filtering
                if !filter_category.is_empty() {
                    continue;
                }
            }

            if let Some(ref filter_source) = source_filter {
                let prompt_source = self
                    .prompt_sources
                    .get(&prompt.name)
                    .cloned()
                    .unwrap_or(PromptSource::Dynamic);

                if filter_source != &prompt_source && filter_source != &PromptSource::Dynamic {
                    continue;
                }
            }

            // Apply exclude patterns
            if self.should_exclude(&prompt.name, exclude) {
                continue;
            }

            // For now, create a dummy path since file_path isn't available in Prompt
            // TODO: Add file_path tracking to Prompt struct
            let file_path = prompt
                .source
                .clone()
                .unwrap_or_else(|| PathBuf::from(format!("{}.md", prompt.name)));
            prompts.push((prompt.name.clone(), file_path));
        }

        Ok(prompts)
    }

    fn should_exclude(&self, name: &str, exclude: &[String]) -> bool {
        for pattern in exclude {
            if self.matches_pattern(name, pattern) {
                return true;
            }
        }
        false
    }

    fn matches_pattern(&self, name: &str, pattern: &str) -> bool {
        // Simple pattern matching - supports * wildcard
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                name.starts_with(parts[0]) && name.ends_with(parts[1])
            } else {
                false
            }
        } else {
            name == pattern
        }
    }

    pub async fn export_tar_gz(
        &self,
        prompts: &[(String, PathBuf)],
        output_path: &Path,
        include_metadata: bool,
    ) -> Result<()> {
        let file = File::create(output_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut tar = Builder::new(encoder);

        let progress = create_progress_bar(prompts.len());

        for (name, path) in prompts {
            let mut file = File::open(path)?;
            tar.append_file(format!("{}.md", name), &mut file)?;
            progress.inc(1);
        }

        if include_metadata {
            let manifest = self.create_manifest(prompts).await?;
            let manifest_json = serde_json::to_string_pretty(&manifest)?;
            let mut header = tar::Header::new_gnu();
            header.set_path("manifest.json")?;
            header.set_size(manifest_json.len() as u64);
            header.set_cksum();
            tar.append_data(&mut header, "manifest.json", manifest_json.as_bytes())?;
        }

        tar.finish()?;
        progress.finish_with_message("Export completed");

        Ok(())
    }

    pub async fn export_zip(
        &self,
        prompts: &[(String, PathBuf)],
        output_path: &Path,
        include_metadata: bool,
    ) -> Result<()> {
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let progress = create_progress_bar(prompts.len());

        for (name, path) in prompts {
            let content = std::fs::read_to_string(path)?;
            zip.start_file(format!("{}.md", name), options)?;
            zip.write_all(content.as_bytes())?;
            progress.inc(1);
        }

        if include_metadata {
            let manifest = self.create_manifest(prompts).await?;
            let manifest_json = serde_json::to_string_pretty(&manifest)?;
            zip.start_file("manifest.json", options)?;
            zip.write_all(manifest_json.as_bytes())?;
        }

        zip.finish()?;
        progress.finish_with_message("Export completed");

        Ok(())
    }

    pub async fn export_directory(
        &self,
        prompts: &[(String, PathBuf)],
        output_path: &Path,
        include_metadata: bool,
    ) -> Result<()> {
        std::fs::create_dir_all(output_path)?;

        let progress = create_progress_bar(prompts.len());

        for (name, path) in prompts {
            let target_path = output_path.join(format!("{}.md", name));
            std::fs::copy(path, target_path)?;
            progress.inc(1);
        }

        if include_metadata {
            let manifest = self.create_manifest(prompts).await?;
            let manifest_json = serde_json::to_string_pretty(&manifest)?;
            let manifest_path = output_path.join("manifest.json");
            std::fs::write(manifest_path, manifest_json)?;
        }

        progress.finish_with_message("Export completed");

        Ok(())
    }

    async fn create_manifest(&self, prompts: &[(String, PathBuf)]) -> Result<ExportManifest> {
        let mut prompt_metadata = Vec::new();

        for (name, path) in prompts {
            let metadata = std::fs::metadata(path)?;
            let content = std::fs::read(path)?;
            let checksum = format!("{:x}", sha2::Sha256::digest(&content));

            if let Some(prompt) = self.library.list()?.iter().find(|p| &p.name == name) {
                prompt_metadata.push(PromptMetadata {
                    name: name.clone(),
                    title: None, // API doesn't have title
                    description: prompt.description.clone(),
                    source: if let Some(ref source_path) = prompt.source {
                        let path_str = source_path.to_string_lossy();
                        if path_str.contains(".swissarmyhammer") || path_str.contains("data") {
                            "builtin".to_string()
                        } else if path_str.contains(".prompts") {
                            "user".to_string()
                        } else {
                            "local".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    },
                    category: None, // TODO: Add category support
                    path: format!("{}.md", name),
                    size: metadata.len(),
                    checksum,
                });
            }
        }

        Ok(ExportManifest {
            version: env!("CARGO_PKG_VERSION").to_string(),
            format_version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            author: None,
            description: None,
            license: None,
            prompts: prompt_metadata,
            dependencies: Vec::new(),
        })
    }
}

fn determine_output_path(output: &Option<String>, format: &ExportFormat) -> Result<PathBuf> {
    if let Some(path) = output {
        Ok(PathBuf::from(path))
    } else {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = match format {
            ExportFormat::TarGz => format!("prompts_{}.tar.gz", timestamp),
            ExportFormat::Zip => format!("prompts_{}.zip", timestamp),
            ExportFormat::Directory => format!("prompts_{}", timestamp),
        };
        Ok(PathBuf::from(filename))
    }
}

fn create_progress_bar(len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_exporter_creation() {
        let exporter = Exporter::new();
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_pattern_matching() {
        let exporter = Exporter::new().unwrap();

        assert!(exporter.matches_pattern("test-prompt", "test-prompt"));
        assert!(exporter.matches_pattern("test-prompt", "test-*"));
        assert!(exporter.matches_pattern("test-prompt", "*-prompt"));
        assert!(!exporter.matches_pattern("test-prompt", "other-*"));
    }

    #[test]
    fn test_should_exclude() {
        let exporter = Exporter::new().unwrap();
        let exclude = vec!["draft-*".to_string(), "temp.md".to_string()];

        assert!(exporter.should_exclude("draft-prompt", &exclude));
        assert!(exporter.should_exclude("temp.md", &exclude));
        assert!(!exporter.should_exclude("my-prompt", &exclude));
    }

    #[test]
    fn test_determine_output_path() {
        let result =
            determine_output_path(&Some("custom.tar.gz".to_string()), &ExportFormat::TarGz);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("custom.tar.gz"));

        let result = determine_output_path(&None, &ExportFormat::TarGz);
        assert!(result.is_ok());
        assert!(result.unwrap().to_string_lossy().contains("prompts_"));
    }

    #[tokio::test]
    async fn test_create_manifest() {
        let exporter = Exporter::new().unwrap();
        let prompts = vec![];

        let manifest = exporter.create_manifest(&prompts).await;
        assert!(manifest.is_ok());

        let manifest = manifest.unwrap();
        assert_eq!(manifest.format_version, "1.0");
        assert!(!manifest.created_at.is_empty());
    }

    #[tokio::test]
    async fn test_export_directory() {
        let exporter = Exporter::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("exported");

        let prompts = vec![];
        let result = exporter
            .export_directory(&prompts, &output_path, false)
            .await;
        assert!(result.is_ok());
        assert!(output_path.exists());
    }
}
