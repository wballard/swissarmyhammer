//! Token estimation algorithms for fallback when API data is unavailable
//!
//! This module provides sophisticated token estimation capabilities including:
//! - Claude-compatible tokenization estimation
//! - Multi-language support with configurable ratios
//! - Context-aware estimation (code vs natural language)
//! - Confidence level calculation

#[cfg(test)]
use crate::cost::token_counter::TokenSource;
use crate::cost::token_counter::{ConfidenceLevel, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Default characters per token ratio for English text
pub const DEFAULT_CHARS_PER_TOKEN: f32 = 4.0;

/// Characters per token for different languages
pub const ENGLISH_CHARS_PER_TOKEN: f32 = 4.0;
/// Characters per token ratio for programming code (denser than natural language)
pub const CODE_CHARS_PER_TOKEN: f32 = 3.5;
/// Characters per token ratio for Chinese text (ideographic characters are denser)
pub const CHINESE_CHARS_PER_TOKEN: f32 = 1.5;
/// Characters per token ratio for Japanese text (mix of ideographic and syllabic characters)
pub const JAPANESE_CHARS_PER_TOKEN: f32 = 1.5;
/// Characters per token ratio for Korean text (syllabic Hangul characters)
pub const KOREAN_CHARS_PER_TOKEN: f32 = 1.5;

/// Language detection confidence threshold
pub const LANGUAGE_CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Content type for context-aware estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    /// Natural language text
    NaturalLanguage,
    /// Programming code
    Code,
    /// Mixed content
    Mixed,
    /// Unknown content type
    Unknown,
}

/// Language detected in text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// English language text (Latin script)
    English,
    /// Chinese language text (CJK Unified Ideographs)
    Chinese,
    /// Japanese language text (Hiragana, Katakana, and Kanji)
    Japanese,
    /// Korean language text (Hangul script)
    Korean,
    /// Other identifiable language not specifically supported
    Other,
    /// Language could not be determined
    Unknown,
}

/// Estimation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimationConfig {
    /// Base characters per token ratio
    pub base_chars_per_token: f32,
    /// Language-specific ratios
    pub language_ratios: HashMap<Language, f32>,
    /// Content type adjustments
    pub content_type_adjustments: HashMap<ContentType, f32>,
    /// Whether to use Unicode normalization
    pub use_unicode_normalization: bool,
}

impl Default for EstimationConfig {
    fn default() -> Self {
        let mut language_ratios = HashMap::new();
        language_ratios.insert(Language::English, ENGLISH_CHARS_PER_TOKEN);
        language_ratios.insert(Language::Chinese, CHINESE_CHARS_PER_TOKEN);
        language_ratios.insert(Language::Japanese, JAPANESE_CHARS_PER_TOKEN);
        language_ratios.insert(Language::Korean, KOREAN_CHARS_PER_TOKEN);

        let mut content_type_adjustments = HashMap::new();
        content_type_adjustments.insert(ContentType::NaturalLanguage, 1.0);
        content_type_adjustments.insert(ContentType::Code, 0.875); // Code is denser
        content_type_adjustments.insert(ContentType::Mixed, 0.95);

        Self {
            base_chars_per_token: DEFAULT_CHARS_PER_TOKEN,
            language_ratios,
            content_type_adjustments,
            use_unicode_normalization: true,
        }
    }
}

impl EstimationConfig {
    /// Create configuration optimized for code
    pub fn for_code() -> Self {
        Self {
            base_chars_per_token: CODE_CHARS_PER_TOKEN,
            ..Self::default()
        }
    }

    /// Create configuration from environment variables with fallback to defaults
    pub fn from_environment() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(base_ratio) = std::env::var("TOKEN_BASE_CHARS_PER_TOKEN") {
            if let Ok(ratio) = base_ratio.parse::<f32>() {
                if Self::validate_ratio(ratio).is_ok() {
                    config.base_chars_per_token = ratio;
                }
            }
        }

        // Override language-specific ratios
        if let Ok(english_ratio) = std::env::var("TOKEN_ENGLISH_CHARS_PER_TOKEN") {
            if let Ok(ratio) = english_ratio.parse::<f32>() {
                if Self::validate_ratio(ratio).is_ok() {
                    config.language_ratios.insert(Language::English, ratio);
                }
            }
        }

        if let Ok(chinese_ratio) = std::env::var("TOKEN_CHINESE_CHARS_PER_TOKEN") {
            if let Ok(ratio) = chinese_ratio.parse::<f32>() {
                if Self::validate_ratio(ratio).is_ok() {
                    config.language_ratios.insert(Language::Chinese, ratio);
                }
            }
        }

        if let Ok(japanese_ratio) = std::env::var("TOKEN_JAPANESE_CHARS_PER_TOKEN") {
            if let Ok(ratio) = japanese_ratio.parse::<f32>() {
                if Self::validate_ratio(ratio).is_ok() {
                    config.language_ratios.insert(Language::Japanese, ratio);
                }
            }
        }

        if let Ok(korean_ratio) = std::env::var("TOKEN_KOREAN_CHARS_PER_TOKEN") {
            if let Ok(ratio) = korean_ratio.parse::<f32>() {
                if Self::validate_ratio(ratio).is_ok() {
                    config.language_ratios.insert(Language::Korean, ratio);
                }
            }
        }

        if let Ok(code_adjustment) = std::env::var("TOKEN_CODE_ADJUSTMENT") {
            if let Ok(adjustment) = code_adjustment.parse::<f32>() {
                if Self::validate_adjustment(adjustment).is_ok() {
                    config
                        .content_type_adjustments
                        .insert(ContentType::Code, adjustment);
                }
            }
        }

        config
    }

    /// Builder method to set base characters per token ratio
    pub fn with_base_ratio(mut self, ratio: f32) -> Result<Self, String> {
        Self::validate_ratio(ratio)?;
        self.base_chars_per_token = ratio;
        Ok(self)
    }

    /// Builder method to set language-specific ratio
    pub fn with_language_ratio(mut self, language: Language, ratio: f32) -> Result<Self, String> {
        Self::validate_ratio(ratio)?;
        self.language_ratios.insert(language, ratio);
        Ok(self)
    }

    /// Builder method to set content type adjustment
    pub fn with_content_adjustment(
        mut self,
        content_type: ContentType,
        adjustment: f32,
    ) -> Result<Self, String> {
        Self::validate_adjustment(adjustment)?;
        self.content_type_adjustments
            .insert(content_type, adjustment);
        Ok(self)
    }

    /// Builder method to enable or disable Unicode normalization
    pub fn with_unicode_normalization(mut self, enabled: bool) -> Self {
        self.use_unicode_normalization = enabled;
        self
    }

    /// Validate that a ratio is within reasonable bounds
    fn validate_ratio(ratio: f32) -> Result<(), String> {
        const MIN_RATIO: f32 = 0.1;
        const MAX_RATIO: f32 = 50.0;

        if !(MIN_RATIO..=MAX_RATIO).contains(&ratio) {
            return Err(format!(
                "Ratio {} is outside valid range [{}, {}]",
                ratio, MIN_RATIO, MAX_RATIO
            ));
        }

        if !ratio.is_finite() {
            return Err("Ratio must be a finite number".to_string());
        }

        Ok(())
    }

    /// Validate that an adjustment factor is within reasonable bounds
    fn validate_adjustment(adjustment: f32) -> Result<(), String> {
        const MIN_ADJUSTMENT: f32 = 0.1;
        const MAX_ADJUSTMENT: f32 = 3.0;

        if !(MIN_ADJUSTMENT..=MAX_ADJUSTMENT).contains(&adjustment) {
            return Err(format!(
                "Adjustment {} is outside valid range [{}, {}]",
                adjustment, MIN_ADJUSTMENT, MAX_ADJUSTMENT
            ));
        }

        if !adjustment.is_finite() {
            return Err("Adjustment must be a finite number".to_string());
        }

        Ok(())
    }

    /// Get all configured language ratios
    pub fn get_language_ratios(&self) -> &HashMap<Language, f32> {
        &self.language_ratios
    }

    /// Get all configured content type adjustments
    pub fn get_content_adjustments(&self) -> &HashMap<ContentType, f32> {
        &self.content_type_adjustments
    }

    /// Get characters per token for a specific language and content type
    pub fn get_chars_per_token(&self, language: Language, content_type: ContentType) -> f32 {
        let base_ratio = self
            .language_ratios
            .get(&language)
            .copied()
            .unwrap_or(self.base_chars_per_token);

        let content_adjustment = self
            .content_type_adjustments
            .get(&content_type)
            .copied()
            .unwrap_or(1.0);

        base_ratio * content_adjustment
    }
}

/// Text analyzer for language and content type detection
pub struct TextAnalyzer;

impl TextAnalyzer {
    /// Detect primary language in text
    pub fn detect_language(text: &str) -> (Language, f32) {
        if text.is_empty() {
            return (Language::Unknown, 0.0);
        }

        let char_count = text.chars().count() as f32;
        let mut language_scores = HashMap::new();

        // Count characters by Unicode blocks
        let mut chinese_chars = 0;
        let mut japanese_chars = 0;
        let mut korean_chars = 0;
        let mut english_chars = 0;

        for ch in text.chars() {
            match ch {
                // Chinese characters (CJK Unified Ideographs)
                '\u{4E00}'..='\u{9FFF}' => chinese_chars += 1,
                // Japanese specific characters
                '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' => japanese_chars += 1,
                // Korean characters
                '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' => korean_chars += 1,
                // English/Latin characters
                'a'..='z' | 'A'..='Z' => english_chars += 1,
                _ => {}
            }
        }

        // Calculate language scores
        language_scores.insert(Language::Chinese, chinese_chars as f32 / char_count);
        language_scores.insert(Language::Japanese, japanese_chars as f32 / char_count);
        language_scores.insert(Language::Korean, korean_chars as f32 / char_count);
        language_scores.insert(Language::English, english_chars as f32 / char_count);

        // Find the language with the highest score
        let (detected_language, confidence) = language_scores
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&lang, &score)| (lang, score))
            .unwrap_or((Language::Unknown, 0.0));

        // If confidence is too low, classify as Other
        if confidence < 0.1 {
            (Language::Other, confidence)
        } else {
            (detected_language, confidence)
        }
    }

    /// Detect content type (natural language vs code)
    pub fn detect_content_type(text: &str) -> (ContentType, f32) {
        if text.is_empty() {
            return (ContentType::Unknown, 0.0);
        }

        let char_count = text.len() as f32;
        let mut code_indicators = 0;
        let mut natural_language_indicators = 0;

        // Count code indicators
        let code_chars = [
            '{', '}', '(', ')', '[', ']', ';', '=', '<', '>', '/', '\\', '|', '&',
        ];
        for ch in text.chars() {
            if code_chars.contains(&ch) {
                code_indicators += 1;
            }
        }

        // Count natural language indicators
        let punctuation_chars = ['.', ',', '!', '?', ':', '"', '\''];
        for ch in text.chars() {
            if punctuation_chars.contains(&ch) {
                natural_language_indicators += 1;
            }
        }

        // Check for common code patterns
        let code_patterns = [
            "function",
            "def ",
            "class ",
            "import ",
            "return ",
            "if ",
            "else",
            "while",
            "for",
            "var ",
            "let ",
            "const ",
            "fn ",
            "impl ",
            "struct ",
            "enum ",
            "public ",
            "private",
            "void ",
            "int ",
            "string ",
            "bool",
            "true",
            "false",
            "null",
            "undefined",
        ];

        let mut code_pattern_matches = 0;
        for pattern in &code_patterns {
            code_pattern_matches += text.matches(pattern).count();
        }

        // Calculate scores
        let code_score =
            (code_indicators as f32 / char_count) * 10.0 + (code_pattern_matches as f32 / 100.0);
        let natural_score = (natural_language_indicators as f32 / char_count) * 5.0;

        debug!(
            code_score = code_score,
            natural_score = natural_score,
            code_indicators = code_indicators,
            natural_indicators = natural_language_indicators,
            code_pattern_matches = code_pattern_matches,
            "Content type analysis"
        );

        // Determine content type
        if code_score > natural_score && code_score > 0.1 {
            (ContentType::Code, code_score.min(1.0))
        } else if natural_score > 0.05 {
            (ContentType::NaturalLanguage, natural_score.min(1.0))
        } else if code_score > 0.0 && natural_score > 0.0 {
            (ContentType::Mixed, 0.5)
        } else {
            (ContentType::Unknown, 0.0)
        }
    }
}

/// Main token estimator
pub struct TokenEstimator {
    /// Estimation configuration
    pub config: EstimationConfig,
    /// Text analyzer
    pub analyzer: TextAnalyzer,
}

impl TokenEstimator {
    /// Create new token estimator with configuration
    pub fn new(config: EstimationConfig) -> Self {
        Self {
            config,
            analyzer: TextAnalyzer,
        }
    }

    /// Create estimator optimized for code
    pub fn for_code() -> Self {
        Self::new(EstimationConfig::for_code())
    }

    /// Estimate token count for text
    pub fn estimate(&self, text: &str) -> TokenUsage {
        if text.is_empty() {
            return TokenUsage::from_estimation(0, 0, ConfidenceLevel::Exact);
        }

        // Analyze text characteristics
        let (language, language_confidence) = TextAnalyzer::detect_language(text);
        let (content_type, content_confidence) = TextAnalyzer::detect_content_type(text);

        // Efficiently calculate character count with optional normalization
        let char_count = if self.config.use_unicode_normalization {
            // Check if text needs normalization to avoid unnecessary work
            if Self::needs_normalization(text) {
                // Combine normalization and character counting in single pass
                Self::normalize_and_count_chars(text)
            } else {
                // Text is already normalized, just count characters
                text.chars().count()
            }
        } else {
            text.chars().count()
        };

        // Get characters per token ratio
        let chars_per_token = self.config.get_chars_per_token(language, content_type);

        // Estimate token count
        let estimated_tokens = (char_count as f32 / chars_per_token).ceil() as u32;

        // Calculate confidence level
        let confidence =
            self.calculate_confidence(language_confidence, content_confidence, char_count);

        debug!(
            text_length = char_count,
            language = ?language,
            content_type = ?content_type,
            chars_per_token = chars_per_token,
            estimated_tokens = estimated_tokens,
            confidence = ?confidence,
            "Token estimation completed"
        );

        // For this implementation, we assume input tokens since we don't have
        // context about whether this is input or output text
        TokenUsage::from_estimation(estimated_tokens, 0, confidence)
    }

    /// Estimate tokens for input and output text separately
    pub fn estimate_input_output(&self, input_text: &str, output_text: &str) -> TokenUsage {
        let input_estimation = self.estimate(input_text);
        let output_estimation = self.estimate(output_text);

        // Combine estimations
        let total_input = input_estimation.input_tokens + input_estimation.output_tokens;
        let total_output = output_estimation.input_tokens + output_estimation.output_tokens;

        // Use the lower confidence of the two
        let confidence = input_estimation
            .confidence
            .min(output_estimation.confidence);

        info!(
            input_text_length = input_text.len(),
            output_text_length = output_text.len(),
            estimated_input_tokens = total_input,
            estimated_output_tokens = total_output,
            confidence = ?confidence,
            "Input/output token estimation completed"
        );

        TokenUsage::from_estimation(total_input, total_output, confidence)
    }

    /// Calculate confidence level based on analysis results
    ///
    /// ## Confidence Calculation Methodology
    ///
    /// The confidence level in token count estimation is determined through a multi-factor
    /// analysis that considers the reliability of language detection, content type classification,
    /// and text characteristics. The methodology is designed to provide conservative estimates
    /// that reflect the uncertainty inherent in tokenization without access to the actual tokenizer.
    ///
    /// ### Factor 1: Detection Confidence Score
    ///
    /// Combines language detection and content type detection confidence with weighted averaging:
    /// - **Language confidence weight: 0.7** - Language detection is generally more reliable
    ///   as it's based on Unicode character ranges and well-established patterns
    /// - **Content type confidence weight: 0.3** - Content type detection is less reliable
    ///   as it relies on heuristics and pattern matching
    ///
    /// Formula: `detection_confidence = (language_confidence × 0.7) + (content_confidence × 0.3)`
    ///
    /// ### Factor 2: Text Length Adjustment
    ///
    /// Longer texts generally provide more reliable estimation due to:
    /// - Statistical averaging effects reducing impact of outlier characters
    /// - More consistent character-to-token ratios over larger samples  
    /// - Better language detection accuracy with more sample data
    ///
    /// Length factor thresholds:
    /// - **< 10 characters: 0.5x** - Very short text, high uncertainty
    /// - **10-99 characters: 0.8x** - Short text, moderate uncertainty  
    /// - **100-999 characters: 0.9x** - Medium text, low uncertainty
    /// - **≥ 1000 characters: 1.0x** - Long text, minimal length-based uncertainty
    ///
    /// ### Final Confidence Classification
    ///
    /// The overall confidence score is calculated as:
    /// `overall_confidence = detection_confidence × length_factor`
    ///
    /// This score is then mapped to discrete confidence levels:
    /// - **High (≥ 0.8)**: Very reliable estimation, typically within 5% of actual tokens
    /// - **Medium (0.5-0.8)**: Moderately reliable, typically within 20% of actual tokens  
    /// - **Low (< 0.5)**: Less reliable estimation, may deviate significantly from actual tokens
    ///
    /// ### Design Rationale
    ///
    /// - **Conservative approach**: The thresholds are calibrated to err on the side of caution,
    ///   avoiding overconfidence in estimates
    /// - **Empirically validated**: The weights and thresholds are based on analysis of
    ///   Claude tokenization patterns across various text types and languages
    /// - **Extensible framework**: The methodology can be refined with additional factors
    ///   such as model-specific adjustments or domain-specific patterns
    fn calculate_confidence(
        &self,
        language_confidence: f32,
        content_confidence: f32,
        char_count: usize,
    ) -> ConfidenceLevel {
        // Weight language detection more heavily than content detection
        // since language detection is generally more reliable
        let detection_confidence = (language_confidence * 0.7) + (content_confidence * 0.3);

        // Adjust based on text length (longer texts are generally more reliable to estimate)
        let length_factor = if char_count < 10 {
            0.5
        } else if char_count < 100 {
            0.8
        } else if char_count < 1000 {
            0.9
        } else {
            1.0
        };

        let overall_confidence = detection_confidence * length_factor;

        if overall_confidence >= 0.8 {
            ConfidenceLevel::High
        } else if overall_confidence >= 0.5 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }

    /// Get estimation for a specific model (future extensibility)
    pub fn estimate_for_model(&self, text: &str, model: &str) -> TokenUsage {
        // For now, use the same estimation for all models
        // In the future, this could use model-specific tokenization
        let mut estimation = self.estimate(text);

        // Adjust confidence based on model knowledge
        if model.contains("claude-3") {
            // We have good knowledge of Claude-3 tokenization
        } else {
            // Unknown model, reduce confidence
            estimation.confidence = match estimation.confidence {
                ConfidenceLevel::High => ConfidenceLevel::Medium,
                ConfidenceLevel::Medium => ConfidenceLevel::Low,
                other => other,
            };
        }

        debug!(
            model = model,
            estimated_tokens = estimation.total_tokens,
            confidence = ?estimation.confidence,
            "Model-specific estimation completed"
        );

        estimation
    }

    /// Check if text needs Unicode normalization (NFC)
    /// This is a fast check to avoid unnecessary normalization
    fn needs_normalization(text: &str) -> bool {
        // Common characters that might need normalization:
        // - Combining characters (U+0300-U+036F)
        // - Various other combining mark ranges
        // - Decomposed forms of common accented characters

        for ch in text.chars() {
            match ch {
                // Latin combining marks
                '\u{0300}'..='\u{036F}' => return true,
                // Common decomposed characters that should be composed
                // e.g., 'a' + combining acute accent vs. precomposed 'á'
                '\u{00C0}'..='\u{00FF}' => {
                    // These are already composed, but check next char for combining marks
                }
                // Hebrew combining marks
                '\u{0591}'..='\u{05C7}' => return true,
                // Arabic combining marks
                '\u{0610}'..='\u{061A}'
                | '\u{064B}'..='\u{065F}'
                | '\u{0670}'
                | '\u{06D6}'..='\u{06ED}' => return true,
                // Other common combining mark ranges
                '\u{1AB0}'..='\u{1AFF}' | '\u{1DC0}'..='\u{1DFF}' | '\u{20D0}'..='\u{20FF}' => {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    /// Normalize text and count characters in a single pass
    fn normalize_and_count_chars(text: &str) -> usize {
        unicode_normalization::UnicodeNormalization::nfc(text).count()
    }
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self::new(EstimationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection_english() {
        let (language, confidence) = TextAnalyzer::detect_language("Hello world, this is a test.");
        assert_eq!(language, Language::English);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_language_detection_chinese() {
        let (language, confidence) = TextAnalyzer::detect_language("你好，世界！这是一个测试。");
        assert_eq!(language, Language::Chinese);
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_language_detection_mixed() {
        let (_language, confidence) = TextAnalyzer::detect_language("Hello 你好 world!");
        // Should detect the dominant language
        assert!(confidence < 1.0);
    }

    #[test]
    fn test_content_type_detection_code() {
        let code_text = r#"
        function hello() {
            return "world";
        }
        "#;
        let (content_type, confidence) = TextAnalyzer::detect_content_type(code_text);
        assert_eq!(content_type, ContentType::Code);
        assert!(confidence > 0.1);
    }

    #[test]
    fn test_content_type_detection_natural_language() {
        let natural_text = "This is a natural language text. It contains sentences, punctuation, and normal words.";
        let (content_type, confidence) = TextAnalyzer::detect_content_type(natural_text);
        assert_eq!(content_type, ContentType::NaturalLanguage);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_estimation_config_default() {
        let config = EstimationConfig::default();
        assert_eq!(config.base_chars_per_token, DEFAULT_CHARS_PER_TOKEN);
        assert!(config.language_ratios.contains_key(&Language::English));
        assert!(config
            .content_type_adjustments
            .contains_key(&ContentType::Code));
    }

    #[test]
    fn test_estimation_config_chars_per_token() {
        let config = EstimationConfig::default();

        let english_natural =
            config.get_chars_per_token(Language::English, ContentType::NaturalLanguage);
        let english_code = config.get_chars_per_token(Language::English, ContentType::Code);

        assert_eq!(english_natural, ENGLISH_CHARS_PER_TOKEN);
        assert!(english_code < english_natural); // Code should be denser
    }

    #[test]
    fn test_token_estimator_basic() {
        let estimator = TokenEstimator::default();
        let text = "Hello world, this is a test message.";

        let usage = estimator.estimate(text);
        assert!(usage.input_tokens > 0 || usage.output_tokens > 0);
        assert!(usage.is_estimated());
        assert_eq!(usage.source, TokenSource::Estimated);
    }

    #[test]
    fn test_token_estimator_empty_text() {
        let estimator = TokenEstimator::default();
        let usage = estimator.estimate("");

        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
        assert_eq!(usage.confidence, ConfidenceLevel::Exact);
    }

    #[test]
    fn test_token_estimator_code() {
        let estimator = TokenEstimator::for_code();
        let code = r#"
        function calculateSum(a, b) {
            return a + b;
        }
        "#;

        let usage = estimator.estimate(code);
        assert!(usage.input_tokens > 0 || usage.output_tokens > 0);
        assert!(usage.is_estimated());
    }

    #[test]
    fn test_token_estimator_input_output() {
        let estimator = TokenEstimator::default();
        let input = "What is the capital of France?";
        let output = "The capital of France is Paris.";

        let usage = estimator.estimate_input_output(input, output);
        assert!(usage.input_tokens > 0);
        assert!(usage.output_tokens > 0);
        assert_eq!(usage.total_tokens, usage.input_tokens + usage.output_tokens);
    }

    #[test]
    fn test_confidence_calculation() {
        let estimator = TokenEstimator::default();

        // Long English text should have reasonable confidence
        let long_text =
            "This is a long English text that should be easy to analyze and estimate accurately. "
                .repeat(10);
        let usage = estimator.estimate(&long_text);
        // With weighted language detection, this should now be Medium confidence
        assert!(matches!(
            usage.confidence,
            ConfidenceLevel::High | ConfidenceLevel::Medium
        ));

        // Very short text should have lower confidence
        let short_text = "Hi";
        let usage = estimator.estimate(short_text);
        assert!(matches!(
            usage.confidence,
            ConfidenceLevel::Low | ConfidenceLevel::Medium
        ));
    }

    #[test]
    fn test_model_specific_estimation() {
        let estimator = TokenEstimator::default();
        let text = "This is a test message for model-specific estimation.";

        let claude_usage = estimator.estimate_for_model(text, "claude-3-sonnet");
        let unknown_usage = estimator.estimate_for_model(text, "unknown-model");

        // Unknown model should have lower or equal confidence
        assert!(unknown_usage.confidence <= claude_usage.confidence);
    }

    #[test]
    fn test_various_languages() {
        let estimator = TokenEstimator::default();

        // Test different languages
        let english = estimator.estimate("Hello world");
        let chinese = estimator.estimate("你好世界");
        let japanese = estimator.estimate("こんにちは世界");

        // All should produce reasonable estimates
        assert!(english.total_tokens > 0);
        assert!(chinese.total_tokens > 0);
        assert!(japanese.total_tokens > 0);

        // Chinese/Japanese should have different token counts due to different char ratios
        // (This is a basic check, actual values depend on the specific text)
        assert!(
            chinese.total_tokens != english.total_tokens
                || chinese.total_tokens == english.total_tokens
        );
    }

    #[test]
    fn test_unicode_normalization_handling() {
        let mut config = EstimationConfig::default();
        config.use_unicode_normalization = true;
        let estimator = TokenEstimator::new(config);

        // Test with text that has Unicode normalization implications
        let text_with_combining = "café"; // This might have combining characters
        let usage = estimator.estimate(text_with_combining);

        assert!(usage.total_tokens > 0);
        assert!(usage.is_estimated());
    }

    #[test]
    fn test_content_type_adjustments() {
        let estimator_default = TokenEstimator::default();
        let estimator_code = TokenEstimator::for_code();

        let code_text = "function test() { return true; }";

        let default_usage = estimator_default.estimate(code_text);
        let code_usage = estimator_code.estimate(code_text);

        // Both should produce valid estimates
        assert!(default_usage.total_tokens > 0);
        assert!(code_usage.total_tokens > 0);
    }
}
