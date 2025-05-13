use clap::{Parser, ValueEnum};
use lopdf::Document;
use std::path::PathBuf;
use std::fs;

/// Defines the splitting mode for the PDF.
#[derive(ValueEnum, Clone, Debug)]
enum SplitMode {
    /// Split the PDF into a specified number of equal chunks.
    NumChunks,
    /// Split the PDF into chunks of a specified number of pages.
    PageSize,
}

/// A command line tool to chunk and save a given pdf file into a new folder.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input PDF file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the output directory
    #[arg(short, long)]
    output: PathBuf,

    /// Splitting mode
    #[arg(short, long, value_enum, default_value_t = SplitMode::PageSize)]
    mode: SplitMode,

    /// Number of pages per chunk (used with --mode page-size)
    #[arg(short, long, default_value_t = 10)]
    chunk_size: usize,

    /// Number of equal chunks (used with --mode num-chunks)
    #[arg(long)]
    num_chunks: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open the input PDF file using lopdf
    let doc = Document::load(&args.input)?;
    let num_pages = doc.get_pages().len();

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;

    let chunk_size = match args.mode {
        SplitMode::PageSize => args.chunk_size,
        SplitMode::NumChunks => {
            let num_chunks = args.num_chunks.unwrap_or(3); // Default to 3 chunks if not specified
            if num_chunks == 0 {
                return Err("Number of chunks cannot be zero.".into());
            }
            (num_pages + num_chunks - 1) / num_chunks // Calculate chunk size
        }
    };

    // Chunk and save the PDF
    for (chunk_index, start_page) in (0..num_pages).step_by(chunk_size).enumerate() {
        let end_page = (start_page + chunk_size).min(num_pages);
        let output_path = args.output.join(format!("chunk_{}.pdf", chunk_index + 1));

        // Clone the original document for the current chunk
        let mut chunk_doc = doc.clone();

        // Determine pages to keep (1-based)
        let pages_to_keep: Vec<u32> = (start_page as u32 + 1..=end_page as u32).collect();
        let all_pages: Vec<u32> = (1..=num_pages as u32).collect();

        // Determine pages to delete
        let pages_to_delete: Vec<u32> = all_pages.into_iter().filter(|p| !pages_to_keep.contains(p)).collect();

        // Delete unwanted pages
        // lopdf delete_pages expects 1-based page numbers
        chunk_doc.delete_pages(&pages_to_delete);

        // Save the chunk document
        chunk_doc.save(&output_path)?;

        println!("Saved chunk {} (pages {} to {}) to {:?}", chunk_index + 1, start_page + 1, end_page, output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_pdf_chunking_page_size() -> Result<(), Box<dyn std::error::Error>> {
        // Using the provided test.pdf
        let dummy_pdf_path = Path::new("test.pdf");
        let output_dir = Path::new("test_output_page_size");

        // Ensure the output directory is clean before testing
        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }

        // Create dummy arguments for page size mode
        let args = Args {
            input: dummy_pdf_path.to_path_buf(),
            output: output_dir.to_path_buf(),
            mode: SplitMode::PageSize,
            chunk_size: 2, // Chunk size of 2 for testing
            num_chunks: None,
        };

        // Run the main logic with dummy arguments
        main_with_args(&args)?; // Pass a reference

        // Verify the output files
        let doc = Document::load(dummy_pdf_path)?;
        let num_pages = doc.get_pages().len();
        let expected_chunks = (num_pages + args.chunk_size - 1) / args.chunk_size;

        let mut chunk_count = 0;
        if output_dir.exists() {
            for entry in fs::read_dir(output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    chunk_count += 1;
                }
            }
        }

        assert_eq!(chunk_count, expected_chunks, "Incorrect number of chunks created for page size mode");

        // TODO: Add more assertions, e.g., check file sizes or attempt to read chunks

        Ok(())
    }

    #[test]
    fn test_pdf_chunking_num_chunks() -> Result<(), Box<dyn std::error::Error>> {
        // Using the provided test.pdf
        let dummy_pdf_path = Path::new("test.pdf");
        let output_dir = Path::new("test_output_num_chunks");

        // Ensure the output directory is clean before testing
        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }

        // Create dummy arguments for number of chunks mode
        let args = Args {
            input: dummy_pdf_path.to_path_buf(),
            output: output_dir.to_path_buf(),
            mode: SplitMode::NumChunks,
            chunk_size: 10, // Default chunk size (not used in this mode)
            num_chunks: Some(3), // Split into 3 chunks
        };

        // Run the main logic with dummy arguments
        main_with_args(&args)?; // Pass a reference

        // Verify the output files
        let doc = Document::load(dummy_pdf_path)?;
        let num_pages = doc.get_pages().len();
        let num_chunks = args.num_chunks.unwrap_or(3);
        let expected_chunks = num_chunks;

        let mut chunk_count = 0;
        if output_dir.exists() {
            for entry in fs::read_dir(output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    chunk_count += 1;
                }
            }
        }

        assert_eq!(chunk_count, expected_chunks, "Incorrect number of chunks created for number of chunks mode");

        // TODO: Add more assertions, e.g., check file sizes or attempt to read chunks

        Ok(())
    }


    // Helper function to call main with specific arguments for testing
    fn main_with_args(args: &Args) -> Result<(), Box<dyn std::error::Error>> { // Accept a reference
        let doc = Document::load(&args.input)?;
        let num_pages = doc.get_pages().len();
        fs::create_dir_all(&args.output)?;

        let chunk_size = match args.mode {
            SplitMode::PageSize => args.chunk_size,
            SplitMode::NumChunks => {
                let num_chunks = args.num_chunks.unwrap_or(3);
                if num_chunks == 0 {
                    return Err("Number of chunks cannot be zero.".into());
                }
                (num_pages + num_chunks - 1) / num_chunks
            }
        };

        for (chunk_index, start_page) in (0..num_pages).step_by(chunk_size).enumerate() {
            let end_page = (start_page + chunk_size).min(num_pages);
            let output_path = args.output.join(format!("chunk_{}.pdf", chunk_index + 1));

            let mut chunk_doc = doc.clone();
            let pages_to_keep: Vec<u32> = (start_page as u32 + 1..=end_page as u32).collect();
            let all_pages: Vec<u32> = (1..=num_pages as u32).collect();
            let pages_to_delete: Vec<u32> = all_pages.into_iter().filter(|p| !pages_to_keep.contains(p)).collect();
            chunk_doc.delete_pages(&pages_to_delete);
            chunk_doc.save(&output_path)?;
        }
        Ok(())
    }
}
