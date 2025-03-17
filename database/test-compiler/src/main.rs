use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input JSON file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output JSON file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Validate the test file
    #[arg(short, long)]
    validate: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestFile {
    context: Option<Vec<Value>>,
    vorgang: Value,
    result: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "should_skip_shouldfail")]
    shouldfail: Option<bool>,
}

fn should_skip_shouldfail(value: &Option<bool>) -> bool {
    match value {
        Some(true) => false, // Don't skip if true
        _ => true,           // Skip if false or None
    }
}

fn merge_json_values(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            // If overlay is an empty object, return base
            if overlay_map.is_empty() {
                return base.clone();
            }
            
            let mut result = base_map.clone();
            
            for (key, overlay_value) in overlay_map {
                if let Some(base_value) = base_map.get(key) {
                    // Special handling for "stationen" field
                    if key == "stationen" {
                        result[key] = merge_stations(base_value, overlay_value);
                    } else {
                        result[key] = merge_json_values(base_value, overlay_value);
                    }
                } else {
                    result[key] = overlay_value.clone();
                }
            }
            
            Value::Object(result)
        },
        (Value::Array(base_array), Value::Array(overlay_array)) => {
            if !overlay_array.is_empty() {
                // If overlay array is not empty, use it
                overlay.clone()
            } else {
                // Otherwise use the base array
                base.clone()
            }
        },
        (_, _) => {
            // For primitive values, if overlay is not null, use it
            if overlay.is_null() {
                base.clone()
            } else {
                overlay.clone()
            }
        }
    }
}

fn merge_stations(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Array(base_stations), Value::Array(overlay_stations)) => {
            if overlay_stations.is_empty() {
                return base.clone();
            }
            
            let mut result = Vec::new();
            
            // Merge stations by index
            for (i, overlay_station) in overlay_stations.iter().enumerate() {
                if i < base_stations.len() {
                    // If there's a corresponding base station, merge them
                    let base_station = &base_stations[i];
                    result.push(merge_station_objects(base_station, overlay_station));
                } else {
                    // If there's no corresponding base station, use the overlay station
                    result.push(overlay_station.clone());
                }
            }
            
            // Add any remaining base stations
            if base_stations.len() > overlay_stations.len() {
                for i in overlay_stations.len()..base_stations.len() {
                    result.push(base_stations[i].clone());
                }
            }
            
            Value::Array(result)
        },
        _ => {
            // If not both arrays, use overlay if not null, otherwise base
            if overlay.is_null() {
                base.clone()
            } else {
                overlay.clone()
            }
        }
    }
}

fn merge_station_objects(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            // If overlay is an empty object, return base
            if overlay_map.is_empty() {
                return base.clone();
            }
            
            let mut result = base_map.clone();
            
            for (key, overlay_value) in overlay_map {
                if let Some(base_value) = base_map.get(key) {
                    result[key] = merge_json_values(base_value, overlay_value);
                } else {
                    result[key] = overlay_value.clone();
                }
            }
            
            Value::Object(result)
        },
        _ => {
            // If not both objects, use overlay if not null, otherwise base
            if overlay.is_null() {
                base.clone()
            } else {
                overlay.clone()
            }
        }
    }
}

fn validate_test_file(test_file: &TestFile) -> Result<bool> {
    if let Some(should_fail) = test_file.shouldfail {
        if should_fail {
            // Check if context and result are different
            if let (Some(context), Some(result)) = (&test_file.context, &test_file.result) {
                if context != result {
                    println!("Test validation passed: shouldfail=true and context != result");
                    return Ok(true);
                } else {
                    println!("Test validation failed: shouldfail=true but context == result");
                    return Ok(false);
                }
            }
        } else {
            // Check if context and result are the same
            if let (Some(context), Some(result)) = (&test_file.context, &test_file.result) {
                if context == result {
                    println!("Test validation passed: shouldfail=false and context == result");
                    return Ok(true);
                } else {
                    println!("Test validation failed: shouldfail=false but context != result");
                    return Ok(false);
                }
            }
        }
    }
    
    // If shouldfail is not specified, assume the test passes
    Ok(true)
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Read the input file
    let content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {:?}", args.input))?;
    
    // Parse the JSON as Value first to check for field existence
    let json_value: Value = serde_json::from_str(&content)
        .with_context(|| "Failed to parse JSON")?;
    
    // Parse the JSON
    let mut test_file: TestFile = serde_json::from_str(&content)
        .with_context(|| "Failed to parse JSON")?;
    
    // Check if "context" key exists in the input JSON
    let context_exists = json_value.get("context").is_some();
    let context_is_empty_array = match json_value.get("context") {
        Some(Value::Array(arr)) if arr.is_empty() => true,
        _ => false,
    };
    
    // Process context
    let context = if context_exists {
        if context_is_empty_array {
            // If context is an empty array in the input, keep it as an empty array
            Some(Vec::new())
        } else if let Some(context_values) = &test_file.context {
            let mut processed_context = Vec::new();
            
            for context_value in context_values {
                // Merge vorgang with context value
                let merged = merge_json_values(&test_file.vorgang, context_value);
                processed_context.push(merged);
            }
            
            Some(processed_context)
        } else {
            // If context is explicitly null in the input, keep it as null
            None
        }
    } else {
        // If context key doesn't exist in the input, don't include it
        None
    };
    
    // Check if "result" key exists in the input JSON
    let result_exists = json_value.get("result").is_some();
    let result_is_empty_array = match json_value.get("result") {
        Some(Value::Array(arr)) if arr.is_empty() => true,
        _ => false,
    };
    
    // Process result
    let result = if result_exists {
        if result_is_empty_array {
            // If result is an empty array in the input, keep it as an empty array
            Some(Vec::new())
        } else if let Some(result_values) = &test_file.result {
            let mut processed_result = Vec::new();
            
            for result_value in result_values {
                // Merge vorgang with result value
                let merged = merge_json_values(&test_file.vorgang, result_value);
                processed_result.push(merged);
            }
            
            Some(processed_result)
        } else {
            // If result is explicitly null in the input, keep it as null
            None
        }
    } else {
        // If result key doesn't exist in the input, don't include it
        None
    };
    
    // Update the test file
    test_file.context = context;
    test_file.result = result;
    
    // Validate the test file if requested
    if args.validate {
        let valid = validate_test_file(&test_file)?;
        if !valid {
            return Err(anyhow::anyhow!("Test validation failed"));
        }
    }
    
    // Serialize back to JSON
    let output_json = serde_json::to_string_pretty(&test_file)
        .with_context(|| "Failed to serialize JSON")?;
    
    // Write to output file or stdout
    if let Some(output_path) = args.output {
        fs::write(&output_path, output_json)
            .with_context(|| format!("Failed to write to output file: {:?}", output_path))?;
        println!("Output written to {:?}", output_path);
    } else {
        println!("{}", output_json);
    }
    
    Ok(())
}
