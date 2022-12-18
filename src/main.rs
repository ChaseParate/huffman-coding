use huffman_coding::{compress, decompress};

fn calculate_compression_ratio(original_size: usize, compressed_size: usize) -> f32 {
    (compressed_size as f32) / (original_size as f32)
}

fn main() {
    let data = String::from("piazza");
    let bytes = data.as_bytes();

    let compressed_data = compress(bytes);
    println!("{:?}", compressed_data);

    let decompressed_data: String = String::from_utf8(decompress(&compressed_data)).unwrap();
    println!("{}", decompressed_data);

    println!(
        "Compression Ratio: {}",
        calculate_compression_ratio(data.len(), compressed_data.len())
    );
}
