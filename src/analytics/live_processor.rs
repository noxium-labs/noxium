use arrow::array::{Float64Array, Int64Array, StringArray, BooleanArray};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use arrow::util::pretty::pretty_format_batches;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use chrono::Utc;

pub fn analyze_data(json_data: &str) {
    let data: Value = match serde_json::from_str(json_data) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return;
        }
    };

    // Validate required fields
    let name = match data["name"].as_str() {
        Some(val) if !val.is_empty() => val,
        _ => {
            eprintln!("Invalid or missing 'name' field");
            return;
        }
    };

    let status = match data["status"].as_str() {
        Some(val) if !val.is_empty() => val,
        _ => {
            eprintln!("Invalid or missing 'status' field");
            return;
        }
    };

    let uptime = match data["uptime"].as_i64() {
        Some(val) if val > 0 => val,
        _ => {
            eprintln!("Invalid or missing 'uptime' field");
            return;
        }
    };

    // Additional fields
    let timestamp = match data["timestamp"].as_i64() {
        Some(val) => val,
        None => Utc::now().timestamp(), // Default to current time if not provided
    };

    let is_active = match data["is_active"].as_bool() {
        Some(val) => val,
        None => false, // Default to false if not provided
    };

    // Define the schema for the data
    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("uptime", DataType::Int64, false),
        Field::new("timestamp", DataType::Timestamp(TimeUnit::Second, None), false),
        Field::new("is_active", DataType::Boolean, false),
    ]));

    // Create Arrow arrays
    let name_array = StringArray::from(vec![name]);
    let status_array = StringArray::from(vec![status]);
    let uptime_array = Int64Array::from(vec![uptime]);
    let timestamp_array = Int64Array::from(vec![timestamp]);
    let is_active_array = BooleanArray::from(vec![is_active]);

    // Create a record batch
    let batch = match RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(name_array) as Arc<dyn arrow::array::Array>,
            Arc::new(status_array),
            Arc::new(uptime_array),
            Arc::new(timestamp_array),
            Arc::new(is_active_array),
        ],
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error creating RecordBatch: {}", e);
            return;
        }
    };

    // Print the batch
    let formatted = match pretty_format_batches(&[batch]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error formatting batches: {}", e);
            return;
        }
    };
    println!("Analyzing data:\n{}", formatted);

    // Additional features

    // 1. Basic statistics
    let uptime_col = batch.column(2).as_any().downcast_ref::<Int64Array>().unwrap();
    let total_uptime: i64 = uptime_col.iter().map(|v| v.unwrap_or(&0)).sum();
    let count = uptime_col.len();
    let avg_uptime = if count > 0 { total_uptime as f64 / count as f64 } else { 0.0 };
    println!("Total Uptime: {}", total_uptime);
    println!("Average Uptime: {:.2}", avg_uptime);

    // 2. Find max uptime
    let max_uptime = uptime_col.iter().filter_map(|v| v).max().unwrap_or(&0);
    println!("Max Uptime: {}", max_uptime);

    // 3. Find min uptime
    let min_uptime = uptime_col.iter().filter_map(|v| v).min().unwrap_or(&0);
    println!("Min Uptime: {}", min_uptime);

    // 4. Generate a histogram
    let mut histogram = HashMap::new();
    for value in uptime_col.iter().filter_map(|v| v) {
        *histogram.entry(value).or_insert(0) += 1;
    }
    println!("Uptime Histogram: {:?}", histogram);

    // 5. Filter records based on status
    if status == "Active" {
        println!("Record is Active");
    } else {
        println!("Record is Inactive");
    }

    // 6. Write record to file
    let file_path = Path::new("record_output.json");
    let json_output = serde_json::json!({
        "name": name,
        "status": status,
        "uptime": uptime,
        "timestamp": timestamp,
        "is_active": is_active
    });
    if let Err(e) = write_to_file(&json_output.to_string(), file_path) {
        eprintln!("Error writing to file: {}", e);
    }

    // 7. Save batch to a Parquet file
    let parquet_file_path = Path::new("record_output.parquet");
    if let Err(e) = save_batch_to_parquet(&batch, parquet_file_path) {
        eprintln!("Error saving batch to Parquet: {}", e);
    }

    // 8. Display data schema
    println!("Schema: {:?}", schema);

    // 9. Show data types of columns
    for (i, column) in batch.columns().iter().enumerate() {
        println!("Column {} Type: {:?}", i, column.data_type());
    }

    // 10. Calculate uptime variance
    let variance: f64 = uptime_col.iter()
        .filter_map(|v| v)
        .map(|&v| (v as f64 - avg_uptime).powi(2))
        .sum::<f64>() / count as f64;
    println!("Uptime Variance: {:.2}", variance);

    // 11. Calculate uptime standard deviation
    let std_dev = variance.sqrt();
    println!("Uptime Standard Deviation: {:.2}", std_dev);

    // 12. Create a summary report
    let report = format!(
        "Summary Report:\n\
        - Total Uptime: {}\n\
        - Average Uptime: {:.2}\n\
        - Max Uptime: {}\n\
        - Min Uptime: {}\n\
        - Uptime Variance: {:.2}\n\
        - Uptime Standard Deviation: {:.2}",
        total_uptime, avg_uptime, max_uptime, min_uptime, variance, std_dev
    );
    println!("{}", report);

    // 13. Compare record against a threshold
    let threshold = 1000;
    if uptime > threshold {
        println!("Uptime exceeds threshold of {}", threshold);
    } else {
        println!("Uptime is below threshold of {}", threshold);
    }

    // 14. Display record timestamp
    let timestamp = Utc.timestamp(timestamp, 0);
    println!("Record Timestamp: {}", timestamp);

    // 15. Convert record to CSV format
    let csv_output = format!("{},{},{},{}", name, status, uptime, timestamp);
    println!("CSV Output: {}", csv_output);

    // 16. Convert record to XML format
    let xml_output = format!(
        "<record>\n\
        <name>{}</name>\n\
        <status>{}</status>\n\
        <uptime>{}</uptime>\n\
        <timestamp>{}</timestamp>\n\
        <is_active>{}</is_active>\n\
        </record>",
        name, status, uptime, timestamp, is_active
    );
    println!("XML Output:\n{}", xml_output);

    // 17. Extract fields as HashMap
    let mut fields = HashMap::new();
    fields.insert("name", name);
    fields.insert("status", status);
    fields.insert("uptime", uptime.to_string());
    fields.insert("timestamp", timestamp.to_string());
    fields.insert("is_active", is_active.to_string());
    println!("Fields HashMap: {:?}", fields);

    // 18. Check if record is recent
    let is_recent = (Utc::now().timestamp() - timestamp) < 3600; // within the last hour
    println!("Record is recent: {}", is_recent);

    // 19. Validate data schema against expected schema
    validate_schema(&batch.schema(), &schema);

    // 20. Serialize batch to a byte vector
    let serialized_batch = serialize_batch(&batch);
    println!("Serialized Batch: {:?}", serialized_batch);

    // 21. Deserialize batch from a byte vector
    let deserialized_batch = deserialize_batch(&serialized_batch);
    match deserialized_batch {
        Ok(batch) => println!("Deserialized Batch: {:?}", batch),
        Err(e) => eprintln!("Error deserializing batch: {}", e),
    }

    // 22. Print number of columns
    println!("Number of Columns: {}", batch.num_columns());

    // 23. Print number of rows
    println!("Number of Rows: {}", batch.num_rows());

    // 24. Filter records where uptime is greater than 5000
    let filtered_uptime = uptime_col.iter()
        .filter_map(|v| v)
        .filter(|&&v| v > 5000)
        .collect::<Vec<_>>();
    println!("Filtered Uptime (greater than 5000): {:?}", filtered_uptime);

    // 25. Find the most common status
    let mut status_count = HashMap::new();
    *status_count.entry(status).or_insert(0) += 1;
    let most_common_status = status_count.into_iter().max_by_key(|&(_, count)| count);
    println!("Most Common Status: {:?}", most_common_status);

    // 26. Print raw data
    println!("Raw Data: {:?}", data);

    // 27. Extract and display uptime as a percentage of max value (assuming max is 10000)
    let max_uptime_value = 10000;
    let uptime_percentage = (uptime as f64 / max_uptime_value as f64) * 100.0;
    println!("Uptime Percentage: {:.2}%", uptime_percentage);

    // 28. Save data to a JSON file
    let json_file_path = Path::new("data_output.json");
    if let Err(e) = write_to_file(&json_data.to_string(), json_file_path) {
        eprintln!("Error saving JSON data to file: {}", e);
    }

    // 29. Generate a random record ID
    let record_id = uuid::Uuid::new_v4();
    println!("Record ID: {}", record_id);

    // 30. Count the number of fields in the JSON
    let field_count = data.as_object().map(|obj| obj.len()).unwrap_or(0);
    println!("Number of Fields in JSON: {}", field_count);

    // 31. Calculate uptime growth rate (dummy implementation)
    let previous_uptime = uptime - 100; // example previous value
    let growth_rate = if previous_uptime > 0 {
        (uptime - previous_uptime) as f64 / previous_uptime as f64 * 100.0
    } else {
        0.0
    };
    println!("Uptime Growth Rate: {:.2}%", growth_rate);

    // 32. Perform data aggregation (sum of uptimes)
    let sum_uptime = uptime_col.iter().filter_map(|v| v).sum::<i64>();
    println!("Sum of Uptimes: {}", sum_uptime);

    // 33. Check if record is flagged for review (dummy condition)
    let flagged_for_review = uptime < 1000 && status == "Inactive";
    println!("Flagged for Review: {}", flagged_for_review);

    // 34. Display record in YAML format
    let yaml_output = serde_yaml::to_string(&json_output).unwrap_or_default();
    println!("YAML Output:\n{}", yaml_output);

    // 35. Check if uptime falls within a range
    let in_range = (1000..5000).contains(&uptime);
    println!("Uptime falls within range 1000-5000: {}", in_range);

    // 36. Generate a summary of active/inactive statuses
    let active_count = if status == "Active" { 1 } else { 0 };
    let inactive_count = if status == "Inactive" { 1 } else { 0 };
    println!("Active Count: {}", active_count);
    println!("Inactive Count: {}", inactive_count);

    // 37. Log record analysis result to a file
    let log_file_path = Path::new("analysis_log.txt");
    let log_entry = format!(
        "Log Entry - {}:\n{}\n",
        Utc::now().to_rfc3339(),
        report
    );
    if let Err(e) = append_to_file(&log_entry, log_file_path) {
        eprintln!("Error appending to log file: {}", e);
    }

    // 38. Validate data for specific conditions
    if uptime > 5000 && is_active {
        println!("Record is active and uptime is high");
    } else {
        println!("Record does not meet criteria");
    }

    // 39. Apply transformations to data
    let transformed_data = format!("Transformed Data: {}, {}, {}", name.to_uppercase(), status.to_uppercase(), uptime * 2);
    println!("{}", transformed_data);

    // 40. Display data in a tabular format
    println!("Tabular Format:\nName | Status | Uptime | Timestamp | Active");
    println!("{} | {} | {} | {} | {}", name, status, uptime, timestamp, is_active);

    // 41. Create a report summary and save to file
    let report_file_path = Path::new("report_summary.txt");
    let report_summary = format!("Report Summary:\n{}", report);
    if let Err(e) = write_to_file(&report_summary, report_file_path) {
        eprintln!("Error saving report summary to file: {}", e);
    }

    // 42. Create a data dictionary with field names and values
    let data_dict = json!({
        "name": name,
        "status": status,
        "uptime": uptime,
        "timestamp": timestamp,
        "is_active": is_active
    });
    println!("Data Dictionary: {}", data_dict);

    // 43. Print data field names and types
    println!("Field Names and Types:");
    for field in schema.fields() {
        println!("Field: {}, Type: {:?}", field.name(), field.data_type());
    }

    // 44. Generate a summary of record fields
    let field_summary = format!(
        "Field Summary:\n\
        Name: {}\n\
        Status: {}\n\
        Uptime: {}\n\
        Timestamp: {}\n\
        Active: {}",
        name, status, uptime, timestamp, is_active
    );
    println!("{}", field_summary);

    // 45. Print data size in bytes
    let data_size = json_data.len();
    println!("Data Size (in bytes): {}", data_size);

    // 46. Save processed data to an Excel file (dummy implementation)
    let excel_file_path = Path::new("data_output.xlsx");
    println!("Saved data to Excel file (dummy implementation): {:?}", excel_file_path);

    // 47. Print JSON data with pretty formatting
    let pretty_json = serde_json::to_string_pretty(&data).unwrap_or_default();
    println!("Pretty JSON Output:\n{}", pretty_json);

    // 48. Save JSON data to a database (dummy implementation)
    println!("Saved JSON data to database (dummy implementation)");

    // 49. Perform data validation checks
    let is_valid = validate_data(&data);
    println!("Data is valid: {}", is_valid);

    // 50. Create a summary of data types in JSON
    let data_types_summary = data.as_object()
        .map(|obj| obj.iter().map(|(k, v)| format!("{}: {:?}", k, v.type_of())).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    println!("Data Types Summary: {}", data_types_summary);

    // 51. Analyze record for anomalies
    let anomalies = if uptime < 1000 {
        "Anomaly detected: Low uptime"
    } else {
        "No anomalies detected"
    };
    println!("{}", anomalies);

    // 52. Generate a random sample of records (dummy implementation)
    println!("Generated random sample of records (dummy implementation)");

    // 53. Print metadata about the record
    println!("Record Metadata:\nName: {}\nStatus: {}\nUptime: {}\nTimestamp: {}", name, status, uptime, timestamp);

    // 54. Compute and print uptime range
    let uptime_range = uptime_col.iter()
        .filter_map(|v| v)
        .fold((i64::MAX, i64::MIN), |(min, max), v| (min.min(v), max.max(v)));
    println!("Uptime Range: {} - {}", uptime_range.0, uptime_range.1);

    // 55. Serialize record to BSON format (dummy implementation)
    println!("Serialized Record to BSON format (dummy implementation)");

    // 56. Print data as a markdown table
    let markdown_table = format!(
        "| Name | Status | Uptime | Timestamp | Active |\n\
        |------|--------|--------|-----------|--------|\n\
        "| {} | {} | {} | {} | {} |\n",
        name, status, uptime, timestamp, is_active
    );
    println!("Markdown Table:\n{}", markdown_table);

    // 57. Check if uptime exceeds a predefined threshold
    let threshold = 5000;
    let exceeds_threshold = uptime > threshold;
    println!("Uptime exceeds threshold of {}: {}", threshold, exceeds_threshold);

    // 58. Print data in different locales
    println!("Data in different locales: Name: {}, Status: {}, Uptime: {}", name.to_uppercase(), status.to_lowercase(), uptime);

    // 59. Show record status based on uptime
    let status_message = if uptime > 10000 {
        "High uptime"
    } else if uptime > 5000 {
        "Moderate uptime"
    } else {
        "Low uptime"
    };
    println!("Uptime Status: {}", status_message);

    // 60. Print JSON data with a timestamp
    let json_with_timestamp = format!(
        "{{\n\
        \"data\": {},\n\
        \"timestamp\": {}\n\
        }}",
        pretty_json,
        Utc::now().to_rfc3339()
    );
    println!("JSON Data with Timestamp:\n{}", json_with_timestamp);
}

fn validate_data(data: &Value) -> bool {
    // Example validation logic (to be expanded)
    data.is_object()
}

fn write_to_file(content: &str, path: &Path) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn append_to_file(content: &str, path: &Path) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new().append(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}