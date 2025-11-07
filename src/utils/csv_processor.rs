






use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;











pub fn generate_headers() -> Vec<String> {
    let mut headers = vec![
        "row_index".to_string(),
        "timestamp_us".to_string(),
        "datetime".to_string(),
    ];

    
    for i in 1..=10 {
        headers.push(format!("bid_price_{}", i));
        headers.push(format!("bid_qty_{}", i));
    }

    
    for i in 1..=10 {
        headers.push(format!("ask_price_{}", i));
        headers.push(format!("ask_qty_{}", i));
    }

    headers
}












pub fn add_headers(input_path: &Path, output_path: &Path) -> Result<usize> {
    println!("📊 Processing CSV file...");
    println!("   Input:  {}", input_path.display());
    println!("   Output: {}", output_path.display());

    
    let input_file = File::open(input_path)
        .context(format!("Failed to open input file: {}", input_path.display()))?;
    let reader = BufReader::new(input_file);

    
    let output_file = File::create(output_path)
        .context(format!("Failed to create output file: {}", output_path.display()))?;
    let mut writer = BufWriter::new(output_file);

    
    let headers = generate_headers();
    writeln!(writer, "{}", headers.join(","))
        .context("Failed to write headers")?;

    println!("   ✓ Headers: {} columns", headers.len());

    
    let mut row_count = 0;
    let mut lines = reader.lines();

    
    if let Some(_) = lines.next() {
        
    } else {
        anyhow::bail!("Input file is empty");
    }

    
    for line in lines {
        let line = line.context("Failed to read line from input file")?;
        writeln!(writer, "{}", line)
            .context("Failed to write line to output file")?;

        row_count += 1;

        
        if row_count % 100_000 == 0 {
            println!("   Processed {} rows...", row_count);
        }
    }

    
    writer.flush().context("Failed to flush output file")?;

    println!("   ✓ Processed {} data rows", row_count);
    println!("   ✓ File saved successfully");

    Ok(row_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_headers() {
        let headers = generate_headers();

        
        
        assert_eq!(headers.len(), 43);

        
        assert_eq!(headers[0], "row_index");
        assert_eq!(headers[1], "timestamp_us");
        assert_eq!(headers[2], "datetime");

        
        assert_eq!(headers[3], "bid_price_1");
        assert_eq!(headers[4], "bid_qty_1");

        
        assert_eq!(headers[23], "ask_price_1");
        assert_eq!(headers[24], "ask_qty_1");
    }
}
