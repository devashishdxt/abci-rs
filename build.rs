use std::{fs::OpenOptions, io::Write};

use bytes::Bytes;
use prost_build::compile_protos;
use reqwest::Result;

const TENDERMINT_URL: &str = "https://raw.githubusercontent.com/tendermint/tendermint/v0.33.6/";

const FILES_TO_DOWNLOAD: [(&str, &str); 3] = [
    ("libs/kv/types.proto", "src/proto/libs/kv/types.proto"),
    ("abci/types/types.proto", "src/proto/abci/types/abci.proto"),
    (
        "crypto/merkle/merkle.proto",
        "src/proto/crypto/merkle/merkle.proto",
    ),
];

#[tokio::main]
async fn main() {
    download_proto_files().await;
    std::env::set_var("OUT_DIR", "src/proto");
    compile_protos(
        &[
            "src/proto/abci/types/abci.proto",
            "src/proto/libs/kv/types.proto",
            "src/proto/crypto/merkle/merkle.proto",
        ],
        &["src/proto"],
    )
    .unwrap()
}

async fn download_proto_files() {
    for (file, destination) in FILES_TO_DOWNLOAD.iter() {
        let bytes = get_bytes(&format!("{}{}", TENDERMINT_URL, file))
            .await
            .unwrap_or_else(|_| panic!("Unable to download [{}]", file));

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(destination)
            .unwrap_or_else(|_| panic!("Unable to open file [{}]", destination));

        file.write_all(&bytes)
            .unwrap_or_else(|_| panic!("Unable to write to [{}]", destination));
    }
}

async fn get_bytes(url: &str) -> Result<Bytes> {
    let response = reqwest::get(url).await?;
    response.bytes().await
}
