extern crate base64;
extern crate ndarray_npy;

use crate::ndarray_npy::WriteNpyExt;
use ndarray::{Array, ArrayView};
use std::fs::{read_dir, File};
use std::io::BufWriter;

static INPUT_SIZE: usize = 1024;

// encoding for seed files
// seeds get base64-encoded, then base64 bytes are put in a vec
// filled with zeros up to INPUT_SIZE
fn encode(data: Vec<u8>, input_size: usize) -> Vec<u8> {
    let mut v = vec![0; input_size];

    let encoded = base64::encode(data);
    for (i, byte) in encoded.as_bytes().iter().enumerate() {
        if i == input_size {
            break;
        }
        v[i] = *byte;
    }

    let row = v;

    return row;
}

#[test]
fn test_encode() {
    let data = vec![65, 66, 67];
    let ret = encode(data, 6);
    assert_eq!(ret, vec![81, 85, 74, 68, 0, 0]);
}

#[tokio::main]
async fn main() {
    let mut array = Array::zeros((0, INPUT_SIZE));

    // spawn futures
    println!("Spawning futures");
    let futures = read_dir("corpus")
        .unwrap()
        .map(|entry| {
            tokio::spawn(async {
                let path = entry.unwrap().path();
                let data = tokio::fs::read(path).await.unwrap();
                if data.len() > INPUT_SIZE {
                    return None; // if input is too long
                }
                let row = encode(data, INPUT_SIZE);
                return Some(row);
            })
        })
        .collect::<Vec<_>>();

    // consume futures
    println!("Consuming futures");
    for future in futures {
        let ret = future.await.unwrap();
        match ret {
            Some(v) => {
                let row = ArrayView::from(&v);
                array.push_row(row).unwrap()
            }
            None => {}
        };
    }

    // save to file
    let writer = BufWriter::new(File::create("out.npy").unwrap());
    array.write_npy(writer).unwrap();
    println!("Done");
}
