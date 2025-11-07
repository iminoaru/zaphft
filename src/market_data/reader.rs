



use crate::types::L2Snapshot;
use anyhow::{Context, Result};
use csv::Reader;
use std::fs::File;
use std::path::Path;


pub struct SnapshotReader {
    reader: Reader<File>,
    snapshots_read: usize,
}

impl SnapshotReader {
    
    pub fn new(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .context(format!("Failed to open snapshot file: {}", path.display()))?;

        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        Ok(Self {
            reader,
            snapshots_read: 0,
        })
    }

    
    
    
    pub fn next_snapshot(&mut self) -> Result<Option<L2Snapshot>> {
        let mut iter = self.reader.deserialize();

        match iter.next() {
            Some(result) => {
                let snapshot: L2Snapshot = result
                    .context(format!("Failed to parse snapshot at row {}", self.snapshots_read))?;

                self.snapshots_read += 1;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    
    
    
    
    pub fn read_all(path: &Path) -> Result<Vec<L2Snapshot>> {
        let file = File::open(path)
            .context(format!("Failed to open snapshot file: {}", path.display()))?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut snapshots = Vec::new();

        for result in reader.deserialize() {
            let snapshot: L2Snapshot = result
                .context("Failed to parse snapshot")?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }

    
    pub fn count(&self) -> usize {
        self.snapshots_read
    }
}


#[derive(Debug)]
pub struct SnapshotStats {
    pub count: usize,
    pub start_time_us: u64,
    pub end_time_us: u64,
    pub duration_ms: u64,
    pub min_spread: f64,
    pub max_spread: f64,
    pub avg_spread: f64,
    pub min_price: f64,
    pub max_price: f64,
}

impl SnapshotStats {
    
    pub fn from_snapshots(snapshots: &[L2Snapshot]) -> Self {
        let count = snapshots.len();

        let start_time_us = snapshots.first().map(|s| s.timestamp_us).unwrap_or(0);
        let end_time_us = snapshots.last().map(|s| s.timestamp_us).unwrap_or(0);
        let duration_ms = (end_time_us - start_time_us) / 1000;

        let spreads: Vec<f64> = snapshots.iter().map(|s| s.spread()).collect();
        let min_spread = spreads.iter().copied().fold(f64::INFINITY, f64::min);
        let max_spread = spreads.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_spread = spreads.iter().sum::<f64>() / spreads.len() as f64;

        let min_price = snapshots.iter()
            .map(|s| s.best_bid())
            .fold(f64::INFINITY, f64::min);
        let max_price = snapshots.iter()
            .map(|s| s.best_ask())
            .fold(f64::NEG_INFINITY, f64::max);

        Self {
            count,
            start_time_us,
            end_time_us,
            duration_ms,
            min_spread,
            max_spread,
            avg_spread,
            min_price,
            max_price,
        }
    }

    
    pub fn print(&self) {
        println!("\n📊 Snapshot Statistics");
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   Total Snapshots: {}", self.count);
        println!("   Duration:        {} ms ({:.2} hours)",
                 self.duration_ms, self.duration_ms as f64 / 3_600_000.0);
        println!("   Price Range:     ${:.2} - ${:.2}", self.min_price, self.max_price);
        println!("   Spread (min):    ${:.4}", self.min_spread);
        println!("   Spread (avg):    ${:.4}", self.avg_spread);
        println!("   Spread (max):    ${:.4}", self.max_spread);
        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_reader() {
        
        let path = Path::new("data/L2_processed.csv");

        if !path.exists() {
            println!("Skipping test - data file not found");
            return;
        }

        let mut reader = SnapshotReader::new(path).expect("Failed to create reader");

        
        let snapshot = reader.next_snapshot()
            .expect("Failed to read snapshot")
            .expect("No snapshot found");

        
        assert!(snapshot.best_bid() > 0.0);
        assert!(snapshot.best_ask() > snapshot.best_bid());
        assert!(snapshot.is_valid());

        println!("✓ Successfully read snapshot: bid={}, ask={}, spread={}",
                 snapshot.best_bid(), snapshot.best_ask(), snapshot.spread());
    }
}
