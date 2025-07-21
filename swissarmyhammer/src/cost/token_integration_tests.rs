//! Integration tests for token counting with existing cost tracking system
//!
//! These tests verify that the new token counting and estimation functionality
//! works correctly with the existing cost tracking infrastructure.

#[cfg(test)]
mod tests {
    use crate::cost::{
        ApiCall, ApiCallStatus, CostCalculator, CostTracker, IssueId, 
        TokenCounter, TokenEstimator, ConfidenceLevel
    };

    #[test]
    fn test_end_to_end_token_counting_workflow() {
        // Create the components
        let mut cost_tracker = CostTracker::new();
        let cost_calculator = CostCalculator::paid_default();
        let mut token_counter = TokenCounter::default();
        let token_estimator = TokenEstimator::default();

        // Start a cost session
        let issue_id = IssueId::new("integration-test-issue").unwrap();
        let session_id = cost_tracker.start_session(issue_id).unwrap();

        // Create an API call
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022"
        ).unwrap();

        // Simulate API response with token usage
        let response_json = r#"{
            "id": "msg_test_123",
            "content": [{"text": "This is a test response from Claude that demonstrates token counting."}],
            "usage": {
                "input_tokens": 25,
                "output_tokens": 15
            }
        }"#;

        // Extract token usage from response
        let api_usage = token_counter.count_from_response(
            response_json,
            None,
            "claude-3-sonnet-20241022"
        ).unwrap();

        // Verify token extraction worked
        assert_eq!(api_usage.input_tokens, 25);
        assert_eq!(api_usage.output_tokens, 15);
        assert_eq!(api_usage.total_tokens, 40);
        assert!(api_usage.is_from_api());

        // Complete the API call with extracted token counts
        api_call.complete(
            api_usage.input_tokens,
            api_usage.output_tokens,
            ApiCallStatus::Success,
            None
        );

        // Add the API call to the session
        let call_id = cost_tracker.add_api_call(&session_id, api_call).unwrap();

        // Calculate cost using the existing cost calculator
        let session = cost_tracker.get_session(&session_id).unwrap();
        let cost_calculation = cost_calculator.calculate_session_cost(session).unwrap();

        // Verify cost calculation includes token counts
        assert_eq!(cost_calculation.input_tokens, 25);
        assert_eq!(cost_calculation.output_tokens, 15);
        assert_eq!(cost_calculation.total_tokens(), 40);
        assert!(cost_calculation.total_cost > rust_decimal::Decimal::ZERO);

        // Complete the session
        cost_tracker.complete_session(&session_id, crate::cost::CostSessionStatus::Completed).unwrap();

        // Verify session statistics
        let completed_session = cost_tracker.get_session(&session_id).unwrap();
        assert!(completed_session.is_completed());
        assert_eq!(completed_session.total_tokens(), 40);
        assert_eq!(completed_session.api_call_count(), 1);
    }

    #[test]
    fn test_token_estimation_fallback() {
        let mut token_counter = TokenCounter::default();
        let token_estimator = TokenEstimator::default();

        // Test scenario: API response doesn't include token usage
        let response_without_usage = r#"{
            "id": "msg_test_456",
            "content": [{"text": "Response without usage data"}]
        }"#;

        // Extraction should fail
        let extraction_result = token_counter.api_extractor.extract_from_response(response_without_usage);
        assert!(extraction_result.is_err());

        // Use estimation as fallback
        let input_text = "What is the weather like today?";
        let output_text = "Response without usage data";
        
        let estimated_usage = token_estimator.estimate_input_output(input_text, output_text);
        
        // Verify estimation worked
        assert!(estimated_usage.input_tokens > 0);
        assert!(estimated_usage.output_tokens > 0);
        assert!(estimated_usage.is_estimated());
        assert!(matches!(
            estimated_usage.confidence,
            ConfidenceLevel::Low | ConfidenceLevel::Medium | ConfidenceLevel::High
        ));
    }

    #[test]
    fn test_token_validation_workflow() {
        let mut token_counter = TokenCounter::default();
        let token_estimator = TokenEstimator::default();

        // Simulate scenario with both API data and estimation
        let response_json = r#"{
            "usage": {
                "input_tokens": 50,
                "output_tokens": 30
            }
        }"#;

        let input_text = "This is a longer input text that should be estimated to verify API accuracy.";
        let output_text = "This is the corresponding output.";
        
        // Get estimation first
        let estimated_usage = token_estimator.estimate_input_output(input_text, output_text);
        
        // Get API usage with validation
        let api_usage = token_counter.count_from_response(
            response_json,
            Some(estimated_usage.clone()),
            "claude-3-sonnet"
        ).unwrap();

        // Verify the API usage is marked as from API
        assert_eq!(api_usage.input_tokens, 50);
        assert_eq!(api_usage.output_tokens, 30);
        assert!(api_usage.is_from_api());

        // Check that validation was performed
        let stats = token_counter.get_validation_stats();
        assert_eq!(stats.total_validations, 1);
        assert!(stats.accuracy_percentage >= 0.0 && stats.accuracy_percentage <= 100.0);
    }

    #[test]
    fn test_multiple_api_calls_with_different_token_sources() {
        let mut cost_tracker = CostTracker::new();
        let cost_calculator = CostCalculator::paid_default();
        let mut token_counter = TokenCounter::default();
        let token_estimator = TokenEstimator::default();

        // Start session
        let issue_id = IssueId::new("multi-call-test").unwrap();
        let session_id = cost_tracker.start_session(issue_id).unwrap();

        // First API call - with API usage data
        let mut api_call1 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet"
        ).unwrap();

        let response1 = r#"{"usage": {"input_tokens": 100, "output_tokens": 75}}"#;
        let usage1 = token_counter.count_from_response(response1, None, "claude-3-sonnet").unwrap();
        
        api_call1.complete(usage1.input_tokens, usage1.output_tokens, ApiCallStatus::Success, None);
        cost_tracker.add_api_call(&session_id, api_call1).unwrap();

        // Second API call - using estimation fallback
        let mut api_call2 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-haiku"
        ).unwrap();

        let input_text = "Tell me about machine learning algorithms.";
        let output_text = "Machine learning algorithms are computational methods that enable systems to learn and improve from data without being explicitly programmed for each task.";
        let usage2 = token_estimator.estimate_input_output(input_text, output_text);
        
        api_call2.complete(usage2.input_tokens, usage2.output_tokens, ApiCallStatus::Success, None);
        cost_tracker.add_api_call(&session_id, api_call2).unwrap();

        // Calculate total session cost
        let session = cost_tracker.get_session(&session_id).unwrap();
        let total_cost = cost_calculator.calculate_session_cost(session).unwrap();

        // Verify both API calls contributed to cost
        let expected_total_tokens = usage1.total_tokens + usage2.total_tokens;
        assert_eq!(total_cost.total_tokens(), expected_total_tokens);
        assert!(total_cost.total_cost > rust_decimal::Decimal::ZERO);
        
        // Verify session has both calls
        assert_eq!(session.api_call_count(), 2);
        assert_eq!(session.total_tokens(), expected_total_tokens);
    }

    #[test]
    fn test_token_counting_performance() {
        let mut token_counter = TokenCounter::default();
        let token_estimator = TokenEstimator::default();

        // Test with various text sizes
        let small_text = "Hello world";
        let medium_text = "This is a medium-length text that contains several sentences and should provide a reasonable estimate for token counting purposes.";
        let large_text = "This is a much longer text that repeats content to test performance characteristics. ".repeat(100);

        // Measure estimation performance
        let start = std::time::Instant::now();
        
        for _ in 0..100 {
            let _small = token_estimator.estimate(small_text);
            let _medium = token_estimator.estimate(medium_text);
            let _large = token_estimator.estimate(&large_text);
        }
        
        let duration = start.elapsed();
        
        // Performance should be reasonable (less than 1 second for 300 estimations)
        assert!(duration.as_secs() < 1, "Token estimation took too long: {:?}", duration);

        // Test API extraction performance
        let response_json = r#"{"usage": {"input_tokens": 150, "output_tokens": 75}}"#;
        
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _usage = token_counter.api_extractor.extract_from_response(response_json).unwrap();
        }
        let duration = start.elapsed();
        
        // API extraction should be very fast
        assert!(duration.as_millis() < 100, "API extraction took too long: {:?}", duration);
    }

    #[test]
    fn test_comprehensive_validation_accuracy() {
        let mut token_counter = TokenCounter::new(0.15); // 15% threshold
        let token_estimator = TokenEstimator::default();

        // Test with various scenarios
        let test_cases = vec![
            // (input_text, output_text, api_input, api_output)
            ("Hello world", "Hi there", 3, 2),
            ("What is the weather?", "It's sunny today.", 5, 4),
            ("Explain quantum physics", "Quantum physics deals with subatomic particles.", 4, 8),
        ];

        for (input_text, output_text, api_input, api_output) in test_cases {
            // Get estimation
            let estimated = token_estimator.estimate_input_output(input_text, output_text);
            
            // Simulate API response
            let api_response = format!(
                r#"{{"usage": {{"input_tokens": {}, "output_tokens": {}}}}}"#,
                api_input, api_output
            );
            
            // Perform validation
            let _api_usage = token_counter.count_from_response(
                &api_response,
                Some(estimated),
                "claude-3-sonnet"
            ).unwrap();
        }

        // Check validation statistics
        let stats = token_counter.get_validation_stats();
        assert!(stats.total_validations > 0);
        assert!(stats.accuracy_percentage >= 0.0 && stats.accuracy_percentage <= 100.0);
        assert!(stats.average_discrepancy >= 0.0);
    }
}