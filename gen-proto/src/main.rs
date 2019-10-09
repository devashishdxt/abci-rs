use protobuf_codegen_pure::{run, Args};

fn main() {
    let args = Args {
        out_dir: "src/proto",
        includes: &["gen-proto/assets"],
        input: &[
            "gen-proto/assets/abci.proto",
            "gen-proto/assets/github.com/tendermint/tendermint/libs/common/types.proto",
            "gen-proto/assets/github.com/tendermint/tendermint/crypto/merkle/merkle.proto",
        ],
        customize: Default::default(),
    };

    run(args).expect("Unable to build protobuf files");
}
