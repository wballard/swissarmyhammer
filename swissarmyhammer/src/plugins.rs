//! Plugin system for extending SwissArmyHammer functionality
//!
//! This module provides a plugin architecture that allows users to extend
//! SwissArmyHammer with custom functionality, including Liquid filters,
//! prompt sources, and output formatters.

use crate::{Result, SwissArmyHammerError};
use std::collections::HashMap;
use std::sync::Arc;

/// Core plugin trait that all plugins must implement
pub trait SwissArmyHammerPlugin: Send + Sync {
    /// Plugin name (must be unique)
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str;

    /// Get custom Liquid filters provided by this plugin
    fn filters(&self) -> Vec<Box<dyn CustomLiquidFilter>>;

    /// Initialize the plugin (called when plugin is loaded)
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cleanup when plugin is unloaded
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Trait for custom Liquid filters
pub trait CustomLiquidFilter: Send + Sync {
    /// Filter name (will be used in templates)
    fn name(&self) -> &str;

    /// Filter description for documentation
    fn description(&self) -> &str;

    /// Apply the filter to input value
    fn apply(&self, input: &liquid::model::Value) -> Result<liquid::model::Value>;
}

/// Plugin registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn SwissArmyHammerPlugin>>,
    filters: HashMap<String, Arc<dyn CustomLiquidFilter>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, mut plugin: Box<dyn SwissArmyHammerPlugin>) -> Result<()> {
        let name = plugin.name().to_string();

        // Check if plugin already exists
        if self.plugins.contains_key(&name) {
            return Err(SwissArmyHammerError::Config(format!(
                "Plugin '{}' is already registered",
                name
            )));
        }

        // Initialize the plugin
        plugin.initialize()?;

        // Register all filters from the plugin
        for filter in plugin.filters() {
            let filter_name = filter.name().to_string();
            if self.filters.contains_key(&filter_name) {
                return Err(SwissArmyHammerError::Config(format!(
                    "Filter '{}' is already registered",
                    filter_name
                )));
            }
            self.filters.insert(filter_name, Arc::from(filter));
        }

        // Store the plugin
        self.plugins.insert(name, Arc::from(plugin));

        Ok(())
    }

    /// Get a registered plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn SwissArmyHammerPlugin>> {
        self.plugins.get(name).cloned()
    }

    /// Get all registered plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get a custom filter by name
    pub fn get_filter(&self, name: &str) -> Option<Arc<dyn CustomLiquidFilter>> {
        self.filters.get(name).cloned()
    }

    /// Get all registered filter names
    pub fn filter_names(&self) -> Vec<String> {
        self.filters.keys().cloned().collect()
    }

    /// Create a liquid parser with standard filters
    ///
    /// Note: Custom filter integration with liquid parser is not yet implemented.
    /// Custom filters can be accessed directly through the registry but are not
    /// automatically available in liquid templates.
    pub fn create_parser(&self) -> liquid::Parser {
        liquid::ParserBuilder::with_stdlib()
            .build()
            .expect("Failed to build Liquid parser")
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquid::model::Value;
    use liquid::ValueView;

    // Test plugin implementation
    struct TestPlugin {
        name: String,
        version: String,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                name: "test-plugin".to_string(),
                version: "1.0.0".to_string(),
            }
        }
    }

    impl SwissArmyHammerPlugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn description(&self) -> &str {
            "A test plugin for unit testing"
        }

        fn filters(&self) -> Vec<Box<dyn CustomLiquidFilter>> {
            vec![Box::new(TestFilter::new())]
        }
    }

    // Test filter implementation
    struct TestFilter {
        name: String,
    }

    impl TestFilter {
        fn new() -> Self {
            Self {
                name: "test_filter".to_string(),
            }
        }
    }

    impl CustomLiquidFilter for TestFilter {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A test filter that reverses strings"
        }

        fn apply(&self, input: &Value) -> Result<Value> {
            // Extract string value properly from liquid Value
            let str_val = input.render().to_string();

            let reversed: String = str_val.chars().rev().collect();
            Ok(Value::scalar(reversed))
        }
    }

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugin_names().len(), 0);
        assert_eq!(registry.filter_names().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = TestPlugin::new();

        assert!(registry.register_plugin(Box::new(plugin)).is_ok());
        assert_eq!(registry.plugin_names().len(), 1);
        assert_eq!(registry.filter_names().len(), 1);
        assert!(registry.get_plugin("test-plugin").is_some());
        assert!(registry.get_filter("test_filter").is_some());
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin1 = TestPlugin::new();
        let plugin2 = TestPlugin::new();

        assert!(registry.register_plugin(Box::new(plugin1)).is_ok());
        assert!(registry.register_plugin(Box::new(plugin2)).is_err());
    }

    #[test]
    fn test_filter_application() {
        let filter = TestFilter::new();
        let input = Value::scalar("hello");
        let result = filter.apply(&input).unwrap();

        // Check that the result is a scalar with the expected value
        match result {
            Value::Scalar(_) => {
                let result_str = result.render().to_string();
                assert_eq!(result_str, "olleh");
            }
            _ => panic!("Expected scalar result"),
        }
    }
}
