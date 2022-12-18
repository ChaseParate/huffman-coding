use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};

type ChildNode<T> = Box<Node<T>>;

#[derive(Debug)]
struct Node<T> {
    data: Option<T>,
    weight: u32,
    left: Option<ChildNode<T>>,
    right: Option<ChildNode<T>>,
}
impl<T> Node<T> {
    fn new_leaf(data: T, weight: u32) -> Self {
        Node {
            data: Some(data),
            weight,
            left: None,
            right: None,
        }
    }

    fn get_leftmost_child(&self) -> &Self {
        let mut node = self;
        while let Some(next_node) = &node.left {
            node = next_node.as_ref();
        }

        node
    }

    fn combine(self, other: Self) -> Self {
        Node {
            data: None,
            weight: self.weight + other.weight,
            left: Some(Box::new(self)),
            right: Some(Box::new(other)),
        }
    }
}
impl Ord for Node<u8> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let weight_order = self.weight.cmp(&other.weight);
        match weight_order {
            Ordering::Equal => {
                // Tie-Breaker
                let self_data = self.get_leftmost_child().data.unwrap();
                let other_data = other.get_leftmost_child().data.unwrap();
                self_data.cmp(&other_data)
            }
            _ => weight_order,
        }
    }
}
impl PartialOrd for Node<u8> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl<T> Eq for Node<T> {}

fn count_bytes(data: &[u8]) -> HashMap<u8, u32> {
    let mut counter = HashMap::new();
    for byte in data {
        if let Some(count) = counter.get_mut(byte) {
            *count += 1;
        } else {
            counter.insert(*byte, 1);
        }
    }

    counter
}

fn build_huffman_tree(counter: &HashMap<u8, u32>) -> Node<u8> {
    let nodes: Vec<Node<u8>> = counter
        .iter()
        .map(|(byte, count)| Node::new_leaf(*byte, *count))
        .collect();

    let mut heap = BinaryHeap::new();
    for node in nodes {
        // Binary Heaps in Rust are Max-Heaps by default, so we have to reverse the Nodes to make it a min-heap.
        heap.push(Reverse(node));
    }

    // Combine nodes into trees until only one remains.
    while heap.len() > 1 {
        let left = heap.pop().unwrap().0;
        let right = heap.pop().unwrap().0;

        let new_node = left.combine(right);

        heap.push(Reverse(new_node));
    }

    heap.pop().unwrap().0
}

fn build_encoding_map(huffman_tree: &Node<u8>) -> HashMap<u8, String> {
    let mut encoding_map = HashMap::new();

    build_encoding_map_recursive(&mut encoding_map, huffman_tree, String::from(""));

    encoding_map
}

fn build_encoding_map_recursive(
    encoding_map: &mut HashMap<u8, String>,
    root: &Node<u8>,
    path: String,
) {
    // Traverse the entire tree, inserting the "path" to each node into the map.
    if let Some(data) = root.data {
        encoding_map.insert(data, path);
    } else {
        build_encoding_map_recursive(
            encoding_map,
            root.left.as_ref().unwrap(),
            path.clone() + "0",
        );
        build_encoding_map_recursive(encoding_map, root.right.as_ref().unwrap(), path + "1");
    }
}

const EOF_CHARACTER: u8 = 0x00;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut data = Vec::from(data);

    // Add an EOF character to end of data.
    data.push(EOF_CHARACTER);

    let counter = count_bytes(&data);
    let huffman_tree = build_huffman_tree(&counter);
    let encoding_map = build_encoding_map(&huffman_tree);

    // Build the header.
    let mut header: Vec<(&u8, &u32)> = counter.iter().collect();
    header.sort();

    let mut header: Vec<u8> = header
        .into_iter()
        .flat_map(|(byte, count)| {
            let mut vec = vec![*byte];
            let mut count_vec = count.to_be_bytes().to_vec();
            vec.append(&mut count_vec);
            vec
        })
        .collect();
    let mut header_terminator = vec![0; 5];
    header.append(&mut header_terminator);

    // Encode the data.
    let encoded_chunks: Vec<String> = data
        .iter()
        .map(|byte| encoding_map.get(byte).unwrap().to_owned())
        .collect();
    let encoded_bits = encoded_chunks.join("");

    let mut encoded_bytes = Vec::new();
    let mut byte = String::new();
    for (i, char) in encoded_bits.char_indices() {
        if i != 0 && i % 8 == 0 {
            encoded_bytes.push(u8::from_str_radix(&byte, 2).unwrap());
            byte.clear();
        }

        byte.push(char);
    }

    // Pad the rest of the byte and add it to the encoded bytes.
    for _ in 0..8 - byte.len() {
        byte.push('0')
    }
    encoded_bytes.push(u8::from_str_radix(&byte, 2).unwrap());

    let mut compressed_data = header;
    compressed_data.append(&mut encoded_bytes);

    compressed_data
}

pub fn decompress(compressed_data: &[u8]) -> Vec<u8> {
    let mut iter = compressed_data.iter();

    // Parse the header to build the counters.
    let mut counter = HashMap::new();
    loop {
        let byte = *iter.next().unwrap();
        let mut count = 0u32;
        for _ in 0..3 {
            count |= *iter.next().unwrap() as u32;
            count <<= 8;
        }
        count |= *iter.next().unwrap() as u32;

        // Header Termination
        if byte == 0 && count == 0 {
            break;
        }

        counter.insert(byte, count);
    }

    let huffman_tree = build_huffman_tree(&counter);
    let encoding_map = build_encoding_map(&huffman_tree);
    let decoding_map: HashMap<String, u8> = encoding_map
        .into_iter()
        .map(|(byte, encoded_bits)| (encoded_bits, byte))
        .collect();

    // Move encoded data into a separate container.
    let mut encoded_data = Vec::new();
    for byte in iter {
        encoded_data.push(*byte);
    }

    // Convert integers into a vector of bits.
    let bits: Vec<char> = encoded_data
        .into_iter()
        .flat_map(|byte| {
            let mut bits = Vec::new();
            for i in 0..8 {
                bits.push(if (byte & (1 << (7 - i))) > 0 {
                    '1'
                } else {
                    '0'
                });
            }
            bits
        })
        .collect();

    // Parse bits based on the decoding map.
    let mut decompressed_data = Vec::new();
    let mut pattern = String::new();
    for bit in bits {
        pattern.push(bit);

        if let Some(byte) = decoding_map.get(&pattern) {
            if *byte == EOF_CHARACTER {
                break;
            }

            decompressed_data.push(*byte);
            pattern.clear();
        }
    }

    decompressed_data
}
