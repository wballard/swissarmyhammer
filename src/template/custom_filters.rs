use liquid::model::Value;
use liquid::{Error, ValueView};
use regex::Regex;
use serde_json;
use chrono::{DateTime, Utc, TimeZone};
use csv::{Reader, Writer};
use std::io::Cursor;
use rand::seq::SliceRandom;

type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// Code and Development Filters
// ============================================================================

pub fn format_lang(input: &Value, args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let _language = args.first()
        .map(|v| v.to_kstr().to_string())
        .unwrap_or_else(|| "text".to_string());
    
    // For now, just wrap in code blocks. Could be enhanced with syntax highlighting
    let formatted = format!("```\n{}\n```", text);
    Ok(Value::scalar(formatted))
}

pub fn extract_functions(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    
    // Simple regex to match function definitions (supports multiple languages)
    let patterns = [
        r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(",     // Python
        r"function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(", // JavaScript
        r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(",      // Rust
        r"func\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(",    // Go
    ];
    
    let mut functions = Vec::new();
    for pattern in &patterns {
        if let Ok(regex) = Regex::new(pattern) {
            for caps in regex.captures_iter(&text) {
                if let Some(func_name) = caps.get(1) {
                    let name = func_name.as_str().to_string();
                    if !functions.contains(&name) {
                        functions.push(name);
                    }
                }
            }
        }
    }
    
    let liquid_values: Vec<Value> = functions
        .into_iter()
        .map(Value::scalar)
        .collect();
    
    Ok(Value::Array(liquid_values))
}

pub fn basename(input: &Value, _args: &[Value]) -> Result<Value> {
    let path_str = input.to_kstr();
    let path = std::path::Path::new(&*path_str);
    let basename = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    Ok(Value::scalar(basename))
}

pub fn dirname(input: &Value, _args: &[Value]) -> Result<Value> {
    let path_str = input.to_kstr();
    let path = std::path::Path::new(&*path_str);
    let dirname = path.parent()
        .and_then(|dir| dir.to_str())
        .unwrap_or("")
        .to_string();
    Ok(Value::scalar(dirname))
}

pub fn count_lines(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let count = text.lines().count();
    Ok(Value::scalar(count as i64))
}

pub fn count_tokens(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let count = text.split_whitespace().count();
    Ok(Value::scalar(count as i64))
}

pub fn dedent(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let lines: Vec<&str> = text.lines().collect();
    
    if lines.is_empty() {
        return Ok(Value::scalar(""));
    }
    
    // Find minimum indentation of non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    
    let dedented_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    
    Ok(Value::scalar(dedented_lines.join("\n")))
}

// ============================================================================
// Text Processing Filters
// ============================================================================

pub fn extract_urls(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    
    // Regex to match URLs
    let url_regex = Regex::new(r"https?://[^\s\]]+")
        .map_err(|e| Error::with_msg(format!("Regex error: {}", e)))?;
    
    let urls: Vec<Value> = url_regex
        .find_iter(&text)
        .map(|m| Value::scalar(m.as_str().to_string()))
        .collect();
    
    Ok(Value::Array(urls))
}

pub fn extract_emails(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    
    // Basic email regex
    let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        .map_err(|e| Error::with_msg(format!("Regex error: {}", e)))?;
    
    let emails: Vec<Value> = email_regex
        .find_iter(&text)
        .map(|m| Value::scalar(m.as_str().to_string()))
        .collect();
    
    Ok(Value::Array(emails))
}

pub fn slugify(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    
    // Convert to lowercase, replace non-alphanumeric with hyphens
    let slug = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        // Remove consecutive hyphens
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    
    Ok(Value::scalar(slug))
}

pub fn word_wrap(input: &Value, args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let width = args.first()
        .and_then(|v| v.as_scalar())
        .and_then(|s| s.to_integer())
        .unwrap_or(80) as usize;
    
    let wrapped = textwrap::fill(&text, width);
    Ok(Value::scalar(wrapped))
}

pub fn indent(input: &Value, args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let spaces = args.first()
        .and_then(|v| v.as_scalar())
        .and_then(|s| s.to_integer())
        .unwrap_or(4) as usize;
    
    let indent = " ".repeat(spaces);
    let indented_lines: Vec<String> = text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect();
    
    Ok(Value::scalar(indented_lines.join("\n")))
}

pub fn bullet_list(input: &Value, _args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let bullet_lines: Vec<String> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!("• {}", line.trim()))
        .collect();
    
    Ok(Value::scalar(bullet_lines.join("\n")))
}

// ============================================================================
// Data Transformation Filters  
// ============================================================================

pub fn to_json(input: &Value, _args: &[Value]) -> Result<Value> {
    // Convert Liquid value to serde_json::Value then serialize
    let json_value = liquid_to_json_value(input);
    let json_string = serde_json::to_string(&json_value)
        .map_err(|e| Error::with_msg(format!("JSON serialization error: {}", e)))?;
    Ok(Value::scalar(json_string))
}

pub fn from_json(input: &Value, _args: &[Value]) -> Result<Value> {
    let json_str = input.to_kstr();
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| Error::with_msg(format!("JSON parsing error: {}", e)))?;
    
    Ok(json_to_liquid_value(&json_value))
}

pub fn from_csv(input: &Value, _args: &[Value]) -> Result<Value> {
    let csv_str = input.to_kstr();
    let cursor = Cursor::new(csv_str.as_bytes());
    let mut reader = Reader::from_reader(cursor);
    
    let mut result = Vec::new();
    let headers: Vec<String> = reader.headers()
        .map_err(|e| Error::with_msg(format!("CSV header error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();
    
    for record in reader.records() {
        let record = record.map_err(|e| Error::with_msg(format!("CSV record error: {}", e)))?;
        let mut row = liquid::Object::new();
        
        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                row.insert(header.clone().into(), Value::scalar(field.to_string()));
            }
        }
        result.push(Value::Object(row));
    }
    
    Ok(Value::Array(result))
}

pub fn to_csv(input: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Array(arr) = input {
        if arr.is_empty() {
            return Ok(Value::scalar(""));
        }
        
        let mut output = Vec::new();
        let mut writer = Writer::from_writer(&mut output);
        
        // Extract headers from first object
        if let Some(Value::Object(first_obj)) = arr.first() {
            let headers: Vec<String> = first_obj.keys().map(|k| k.to_string()).collect();
            writer.write_record(&headers)
                .map_err(|e| Error::with_msg(format!("CSV write error: {}", e)))?;
            
            // Write data rows
            for item in arr {
                if let Value::Object(obj) = item {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|h| {
                            obj.get(h.as_str())
                                .map(|v| v.to_kstr().to_string())
                                .unwrap_or_default()
                        })
                        .collect();
                    writer.write_record(&row)
                        .map_err(|e| Error::with_msg(format!("CSV write error: {}", e)))?;
                }
            }
        }
        
        writer.flush()
            .map_err(|e| Error::with_msg(format!("CSV flush error: {}", e)))?;
        
        drop(writer);
        let csv_string = String::from_utf8(output)
            .map_err(|e| Error::with_msg(format!("UTF-8 error: {}", e)))?;
        
        Ok(Value::scalar(csv_string))
    } else {
        Err(Error::with_msg("Input must be an array"))
    }
}

pub fn keys(input: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Object(obj) = input {
        let keys: Vec<Value> = obj.keys()
            .map(|k| Value::scalar(k.to_string()))
            .collect();
        Ok(Value::Array(keys))
    } else {
        Err(Error::with_msg("Input must be an object"))
    }
}

pub fn values(input: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Object(obj) = input {
        let values: Vec<Value> = obj.values().cloned().collect();
        Ok(Value::Array(values))
    } else {
        Err(Error::with_msg("Input must be an object"))
    }
}

// ============================================================================
// Utility Filters
// ============================================================================

pub fn format_date(input: &Value, args: &[Value]) -> Result<Value> {
    let date_str = input.to_kstr();
    let format = args.first()
        .map(|s| s.to_kstr().to_string())
        .unwrap_or_else(|| "%Y-%m-%d".to_string());
    
    // Try to parse as Unix timestamp first, then as RFC3339
    let datetime = if let Ok(timestamp) = date_str.parse::<i64>() {
        Utc.timestamp_opt(timestamp, 0).single()
            .ok_or_else(|| Error::with_msg("Invalid timestamp"))?
    } else if let Ok(dt) = DateTime::parse_from_rfc3339(&date_str) {
        dt.with_timezone(&Utc)
    } else {
        return Err(Error::with_msg("Could not parse date"));
    };
    
    let formatted = datetime.format(&format).to_string();
    Ok(Value::scalar(formatted))
}

pub fn lorem(_input: &Value, args: &[Value]) -> Result<Value> {
    let word_count = args.first()
        .and_then(|v| v.as_scalar())
        .and_then(|s| s.to_integer())
        .unwrap_or(50) as usize;
    
    let lorem_words = [
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
        "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
        "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis", "nostrud",
        "exercitation", "ullamco", "laboris", "nisi", "aliquip", "ex", "ea", "commodo",
        "consequat", "duis", "aute", "irure", "in", "reprehenderit", "voluptate",
        "velit", "esse", "cillum", "fugiat", "nulla", "pariatur", "excepteur", "sint",
        "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui", "officia",
        "deserunt", "mollit", "anim", "id", "est", "laborum"
    ];
    
    let mut result = Vec::new();
    for i in 0..word_count {
        result.push(lorem_words[i % lorem_words.len()]);
    }
    
    Ok(Value::scalar(result.join(" ")))
}

pub fn ordinal(input: &Value, _args: &[Value]) -> Result<Value> {
    let num = input.as_scalar()
        .and_then(|s| s.to_integer())
        .ok_or_else(|| Error::with_msg("Input must be a number"))?;
    
    let suffix = match num % 100 {
        11..=13 => "th",
        _ => match num % 10 {
            1 => "st",
            2 => "nd", 
            3 => "rd",
            _ => "th",
        }
    };
    
    Ok(Value::scalar(format!("{}{}", num, suffix)))
}

pub fn highlight(input: &Value, args: &[Value]) -> Result<Value> {
    let text = input.to_kstr();
    let keyword = args.first()
        .map(|s| s.to_kstr().to_string())
        .unwrap_or_default();
    
    if keyword.is_empty() {
        return Ok(input.clone());
    }
    
    let highlighted = text.replace(&keyword, &format!("**{}**", keyword));
    Ok(Value::scalar(highlighted))
}

pub fn sample(input: &Value, args: &[Value]) -> Result<Value> {
    if let Value::Array(arr) = input {
        let count = args.first()
            .and_then(|v| v.as_scalar())
            .and_then(|s| s.to_integer())
            .unwrap_or(1) as usize;
        
        let mut rng = rand::thread_rng();
        let sampled: Vec<Value> = arr
            .choose_multiple(&mut rng, count.min(arr.len()))
            .cloned()
            .collect();
        
        Ok(Value::Array(sampled))
    } else {
        Err(Error::with_msg("Input must be an array"))
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn liquid_to_json_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Nil => serde_json::Value::Null,
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(liquid_to_json_value).collect())
        }
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (key, value) in obj.iter() {
                map.insert(key.to_string(), liquid_to_json_value(value));
            }
            serde_json::Value::Object(map)
        }
        Value::Scalar(scalar) => {
            if let Some(i) = scalar.to_integer() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = scalar.to_float() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else if let Some(b) = scalar.to_bool() {
                serde_json::Value::Bool(b)
            } else {
                serde_json::Value::String(scalar.to_kstr().to_string())
            }
        }
        Value::State(_) => serde_json::Value::Null,
    }
}

fn json_to_liquid_value(value: &serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::scalar(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::scalar(i)
            } else if let Some(f) = n.as_f64() {
                Value::scalar(f)
            } else {
                Value::scalar(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::scalar(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.iter().map(json_to_liquid_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut liquid_obj = liquid::Object::new();
            for (key, value) in obj {
                liquid_obj.insert(key.clone().into(), json_to_liquid_value(value));
            }
            Value::Object(liquid_obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquid::model::Value;

    #[test]
    fn test_basename_filter() {
        let input = Value::scalar("/path/to/file.txt");
        let result = basename(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "file.txt");
    }

    #[test]
    fn test_dirname_filter() {
        let input = Value::scalar("/path/to/file.txt");
        let result = dirname(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "/path/to");
    }

    #[test]
    fn test_count_lines_filter() {
        let input = Value::scalar("line1\nline2\nline3");
        let result = count_lines(&input, &[]).unwrap();
        assert_eq!(result.as_scalar().unwrap().to_integer().unwrap(), 3);
    }

    #[test]
    fn test_slugify_filter() {
        let input = Value::scalar("Hello World! This is a Test");
        let result = slugify(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "hello-world-this-is-a-test");
    }

    #[test]
    fn test_extract_functions_filter() {
        let input = Value::scalar("def hello_world():\n    pass\n\nfunction test() {\n    return true;\n}");
        let result = extract_functions(&input, &[]).unwrap();
        
        if let Value::Array(functions) = result {
            assert_eq!(functions.len(), 2);
            assert!(functions.iter().any(|f| f.to_kstr() == "hello_world"));
            assert!(functions.iter().any(|f| f.to_kstr() == "test"));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_extract_urls_filter() {
        let input = Value::scalar("Visit https://example.com and http://test.org for more info");
        let result = extract_urls(&input, &[]).unwrap();
        
        if let Value::Array(urls) = result {
            assert_eq!(urls.len(), 2);
            assert!(urls.iter().any(|u| u.to_kstr() == "https://example.com"));
            assert!(urls.iter().any(|u| u.to_kstr() == "http://test.org"));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_extract_emails_filter() {
        let input = Value::scalar("Contact us at support@example.com or admin@test.org");
        let result = extract_emails(&input, &[]).unwrap();
        
        if let Value::Array(emails) = result {
            assert_eq!(emails.len(), 2);
            assert!(emails.iter().any(|e| e.to_kstr() == "support@example.com"));
            assert!(emails.iter().any(|e| e.to_kstr() == "admin@test.org"));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_dedent_filter() {
        let input = Value::scalar("    line1\n    line2\n        indented");
        let result = dedent(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "line1\nline2\n    indented");
    }

    #[test]
    fn test_to_json_filter() {
        let input = Value::scalar("hello");
        let result = to_json(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "\"hello\"");
    }

    #[test]
    fn test_from_json_filter() {
        let input = Value::scalar("{\"name\":\"test\",\"value\":42}");
        let result = from_json(&input, &[]).unwrap();
        
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("name").unwrap().to_kstr(), "test");
            assert_eq!(obj.get("value").unwrap().as_scalar().unwrap().to_integer().unwrap(), 42);
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_count_tokens_filter() {
        let input = Value::scalar("hello world test");
        let result = count_tokens(&input, &[]).unwrap();
        assert_eq!(result.as_scalar().unwrap().to_integer().unwrap(), 3);
    }

    #[test]
    fn test_indent_filter() {
        let input = Value::scalar("line1\nline2\nline3");
        let args = vec![Value::scalar(2)];
        let result = indent(&input, &args).unwrap();
        assert_eq!(result.to_kstr(), "  line1\n  line2\n  line3");
    }

    #[test]
    fn test_bullet_list_filter() {
        let input = Value::scalar("item1\nitem2\nitem3");
        let result = bullet_list(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "• item1\n• item2\n• item3");
    }

    #[test]
    fn test_ordinal_filter() {
        let input = Value::scalar(1);
        let result = ordinal(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "1st");
        
        let input = Value::scalar(2);
        let result = ordinal(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "2nd");
        
        let input = Value::scalar(3);
        let result = ordinal(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "3rd");
        
        let input = Value::scalar(4);
        let result = ordinal(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "4th");
        
        let input = Value::scalar(11);
        let result = ordinal(&input, &[]).unwrap();
        assert_eq!(result.to_kstr(), "11th");
    }

    #[test]
    fn test_highlight_filter() {
        let input = Value::scalar("This is a test sentence");
        let args = vec![Value::scalar("test")];
        let result = highlight(&input, &args).unwrap();
        assert_eq!(result.to_kstr(), "This is a **test** sentence");
    }

    #[test]
    fn test_lorem_filter() {
        let input = Value::scalar("");
        let args = vec![Value::scalar(5)];
        let result = lorem(&input, &args).unwrap();
        let result_str = result.to_kstr();
        let words: Vec<&str> = result_str.split_whitespace().collect();
        assert_eq!(words.len(), 5);
        assert_eq!(words[0], "lorem");
        assert_eq!(words[1], "ipsum");
    }

    #[test]
    fn test_keys_filter() {
        let mut obj = liquid::Object::new();
        obj.insert("name".into(), Value::scalar("test"));
        obj.insert("value".into(), Value::scalar(42));
        let input = Value::Object(obj);
        
        let result = keys(&input, &[]).unwrap();
        if let Value::Array(keys) = result {
            assert_eq!(keys.len(), 2);
            let key_strs: Vec<String> = keys.iter().map(|k| k.to_kstr().to_string()).collect();
            assert!(key_strs.contains(&"name".to_string()));
            assert!(key_strs.contains(&"value".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_values_filter() {
        let mut obj = liquid::Object::new();
        obj.insert("name".into(), Value::scalar("test"));
        obj.insert("value".into(), Value::scalar(42));
        let input = Value::Object(obj);
        
        let result = values(&input, &[]).unwrap();
        if let Value::Array(values) = result {
            assert_eq!(values.len(), 2);
        } else {
            panic!("Expected array result");
        }
    }
}