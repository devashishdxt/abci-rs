use protobuf_codegen_pure::{run, Args};

fn main() {
    let args = Args {
        out_dir: "src/proto",
        includes: &["protobuf"],
        input: &[
            "protobuf/abci.proto",
            "protobuf/github.com/tendermint/tendermint/libs/common/types.proto",
            "protobuf/github.com/tendermint/tendermint/crypto/merkle/merkle.proto",
        ],
        customize: Default::default(),
    };

    run(args).expect("Unable to build protobuf files");
}
