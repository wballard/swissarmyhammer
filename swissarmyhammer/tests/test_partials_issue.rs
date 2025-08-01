use std::collections::HashMap;
use swissarmyhammer::{Prompt, PromptLibrary};

#[test]
fn test_partials_with_liquid_extension() {
    // Create a library and add a partial
    let mut library = PromptLibrary::new();

    // Add the partial prompt (like principals.md.liquid)
    let partial = Prompt::new(
        "principals.md.liquid",
        "{% partial %}\n\n## Principals\n\nDon't hold back!",
    );
    library.add(partial).unwrap();

    // Add a main prompt that uses the partial
    let main_prompt = Prompt::new(
        "do_next_issue",
        "## Goal\n\n{% render \"principals\" %}\n\nDo the work!",
    );
    library.add(main_prompt).unwrap();

    // Try to render the main prompt
    let args = HashMap::new();
    match library.render_prompt("do_next_issue", &args) {
        Ok(result) => {
            println!("Success:\n{result}");
            assert!(result.contains("Principals"));
            assert!(result.contains("Don't hold back"));
        }
        Err(e) => {
            println!("Error: {e}");
            panic!("Failed to render prompt with partial: {e}");
        }
    }
}

#[test]
fn test_partials_without_extension() {
    // Create a library and add a partial
    let mut library = PromptLibrary::new();

    // Add the partial prompt (without extension)
    let partial = Prompt::new(
        "principals",
        "{% partial %}\n\n## Principals\n\nDon't hold back!",
    );
    library.add(partial).unwrap();

    // Add a main prompt that uses the partial
    let main_prompt = Prompt::new(
        "do_next_issue",
        "## Goal\n\n{% render \"principals\" %}\n\nDo the work!",
    );
    library.add(main_prompt).unwrap();

    // Try to render the main prompt
    let args = HashMap::new();
    match library.render_prompt("do_next_issue", &args) {
        Ok(result) => {
            println!("Success:\n{result}");
            assert!(result.contains("Principals"));
            assert!(result.contains("Don't hold back"));
        }
        Err(e) => {
            println!("Error: {e}");
            panic!("Failed to render prompt with partial: {e}");
        }
    }
}
