use clap::Parser;
use lopdf::Document;
use std::path::PathBuf;
use std::fs;

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

    /// Splitting method: specify number of pages per chunk (e.g., -s 30) or number of equal chunks (e.g., -s c5)
    #[arg(short, long)]
    split: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open the input PDF file using lopdf
    let doc = Document::load(&args.input)?;
    let num_pages = doc.get_pages().len();

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;

    let chunk_size = if args.split.starts_with('c') {
        let num_chunks = args.split[1..].parse::<usize>()?;
        if num_chunks == 0 {
            return Err("Number of chunks cannot be zero.".into());
        }
        (num_pages + num_chunks - 1) / num_chunks // Calculate chunk size
    } else {
        args.split.parse::<usize>()? // Parse as page size
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

    // Helper function to call main with specific arguments for testing
    fn main_with_args(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
        let doc = Document::load(&args.input)?;
        let num_pages = doc.get_pages().len();
        fs::create_dir_all(&args.output)?;

        let chunk_size = if args.split.starts_with('c') {
            let num_chunks = args.split[1..].parse::<usize>()?;
            if num_chunks == 0 {
                return Err("Number of chunks cannot be zero.".into());
            }
            (num_pages + num_chunks - 1) / num_chunks
        } else {
            args.split.parse::<usize>()?
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

    #[test]
    fn test_pdf_chunking_page_size() -> Result<(), Box<dyn std::error::Error>> {
        let dummy_pdf_path = Path::new("test.pdf");
        let output_dir = Path::new("test_output_page_size");

        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }

        let args = Args {
            input: dummy_pdf_path.to_path_buf(),
            output: output_dir.to_path_buf(),
            split: "2".to_string(), // Page size of 2
        };

        main_with_args(&args)?;

        let doc = Document::load(dummy_pdf_path)?;
        let num_pages = doc.get_pages().len();
        let chunk_size = args.split.parse::<usize>()?;
        let expected_chunks = (num_pages + chunk_size - 1) / chunk_size;

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

        Ok(())
    }

    #[test]
    fn test_pdf_chunking_num_chunks() -> Result<(), Box<dyn std::error::Error>> {
        let dummy_pdf_path = Path::new("test.pdf");
        let output_dir = Path::new("test_output_num_chunks");

        if output_dir.exists() {
            fs::remove_dir_all(output_dir)?;
        }

        let args = Args {
            input: dummy_pdf_path.to_path_buf(),
            output: output_dir.to_path_buf(),
            split: "c3".to_string(), // Split into 3 chunks
        };

        main_with_args(&args)?;

        let doc = Document::load(dummy_pdf_path)?;
        let num_pages = doc.get_pages().len();
        let num_chunks = args.split[1..].parse::<usize>()?;
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

        Ok(())
    }
}
