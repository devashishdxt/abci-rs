use std::{fs::OpenOptions, io::Write};

use bytes::Bytes;
use protobuf_codegen_pure::Codegen;
use reqwest::Result;

const TENDERMINT_URL: &str = "https://raw.githubusercontent.com/tendermint/tendermint/v0.33.4/";

const FILES_TO_DOWNLOAD: [(&str, &str); 3] = [
    (
        "libs/kv/types.proto",
        "gen-proto/assets/libs/kv/types.proto",
    ),
    (
        "abci/types/types.proto",
        "gen-proto/assets/abci/types/abci.proto",
    ),
    (
        "crypto/merkle/merkle.proto",
        "gen-proto/assets/crypto/merkle/merkle.proto",
    ),
];

#[tokio::main]
async fn main() {
    for (file, destination) in FILES_TO_DOWNLOAD.iter() {
        let bytes = get_bytes(&format!("{}{}", TENDERMINT_URL, file))
            .await
            .expect(&format!("Unable to download [{}]", file));

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(destination)
            .expect(&format!("Unable to open file [{}]", destination));

        file.write_all(&bytes)
            .expect(&format!("Unable to write to [{}]", destination));
    }

    Codegen::new()
        .out_dir("src/proto")
        .include("gen-proto/assets")
        .inputs(&[
            "gen-proto/assets/abci/types/abci.proto",
            "gen-proto/assets/libs/kv/types.proto",
            "gen-proto/assets/crypto/merkle/merkle.proto",
        ])
        .run()
        .expect("Unable to build protobuf files");
}

async fn get_bytes(url: &str) -> Result<Bytes> {
    let response = reqwest::get(url).await?;
    response.bytes().await
}
