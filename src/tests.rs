mod counter;
mod request_generator;

use std::time::{Duration, Instant};

use mock_io::tokio::{MockListener, MockStream};
use tendermint_proto::abci::{
    response::Value as ResponseValue, Request, Response, ResponseException,
};
use tokio::spawn;

use crate::{types::ResponseCheckTx, utils::get_stream_pair, Address};

async fn initialize_server() -> (MockStream, MockStream) {
    let server = counter::server();

    let (listener, handle) = MockListener::new();
    let address: Address = listener.into();

    spawn(async move {
        server
            .run(address)
            .await
            .expect("Unable to start ABCI server");
    });

    (
        MockStream::connect(&handle).unwrap(),
        MockStream::connect(&handle).unwrap(),
    )
}

async fn initialize_server_with_state(counter: u64, block_height: i64) -> (MockStream, MockStream) {
    let server = counter::server_with_state(counter, block_height);

    let (listener, handle) = MockListener::new();
    let address: Address = listener.into();

    spawn(async move {
        server
            .run(address)
            .await
            .expect("Unable to start ABCI server");
    });

    (
        MockStream::connect(&handle).unwrap(),
        MockStream::connect(&handle).unwrap(),
    )
}

#[tokio::test]
async fn check_concurrent_check_tx_requests() {
    let (info_stream, mempool_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut mempool_stream_reader, mut mempool_stream_writer) = get_stream_pair(mempool_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let request = request_generator::info();
    info_stream_writer.write(request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Send one `check_tx` for mempool task scheduling
    let request = request_generator::check_tx(1, false);
    mempool_stream_writer.write(request).await.unwrap();
    let response: Response = mempool_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::CheckTx(_)));

    // Sent three `check_tx` requests and check if all run concurrently and responses are received
    // in order
    let start_time = Instant::now();

    // This request will take 2 seconds to execute (see `check_tx` implementation in `counter.rs`)
    mempool_stream_writer
        .write(request_generator::check_tx(1, true))
        .await
        .unwrap();
    // This request will take 2 seconds to execute (see `check_tx` implementation in `counter.rs`)
    mempool_stream_writer
        .write(request_generator::check_tx(2, true))
        .await
        .unwrap();
    // This request will get executed immediately (see `check_tx` implementation in `counter.rs`)
    mempool_stream_writer
        .write(request_generator::check_tx(3, true))
        .await
        .unwrap();
    let response1: Response = mempool_stream_reader.read().await.unwrap().unwrap();
    let response2: Response = mempool_stream_reader.read().await.unwrap().unwrap();
    let response3: Response = mempool_stream_reader.read().await.unwrap().unwrap();

    let duration = Instant::now() - start_time;

    // To check if all the requests executed concurrently, we check if all the responses were
    // returned within 4 seconds and in order.
    assert!(duration < Duration::from_secs(4));

    assert!(response1.value.is_some());
    assert!(matches!(
        response1.value.unwrap(),
        ResponseValue::CheckTx(ResponseCheckTx { data, .. }) if data == 1u64.to_be_bytes().to_vec()
    ));
    assert!(response2.value.is_some());
    assert!(matches!(
        response2.value.unwrap(),
        ResponseValue::CheckTx(ResponseCheckTx { data, .. }) if data == 2u64.to_be_bytes().to_vec()
    ));
    assert!(response3.value.is_some());
    assert!(matches!(
        response3.value.unwrap(),
        ResponseValue::CheckTx(ResponseCheckTx { data, .. }) if data == 3u64.to_be_bytes().to_vec()
    ));
}

#[tokio::test]
async fn check_task_scheduling() {
    let (info_stream, _) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let request = request_generator::info();
    info_stream_writer.write(request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();

    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    //
    // Note: We'll use info connection to send `init_chain`. This should return an exception.
    let request = request_generator::init_chain();
    info_stream_writer.write(request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Non-info request on info connection"
    ));
}

#[tokio::test]
async fn check_valid_abci_flow() {
    let (info_stream, consensus_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let request = request_generator::info();
    info_stream_writer.write(request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();

    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let request = request_generator::init_chain();
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let request = request_generator::begin_block(1, Default::default());
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint may call multiple `deliver_tx`
    let request = request_generator::deliver_tx(1);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    let request = request_generator::deliver_tx(2);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // After all the transactions are delivered, tendermint will call `end_block`
    let request = request_generator::end_block(1);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));

    // Finally, tendermint will call `commit`
    let request = request_generator::commit();
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Commit(_)));

    // Next, tendermint will call `begin_block` with `block_height = 2`
    let request = request_generator::begin_block(2, 2u64.to_be_bytes().to_vec());
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint may call multiple `deliver_tx`
    let request = request_generator::deliver_tx(3);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    let request = request_generator::deliver_tx(4);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // After all the transactions are delivered, tendermint will call `end_block`
    let request = request_generator::end_block(2);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));

    // Finally, tendermint will call `commit`
    let request = request_generator::commit();
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Commit(_)));
}

#[tokio::test]
async fn check_valid_abci_flow_with_init_state() {
    let (info_stream, consensus_stream) = initialize_server_with_state(4, 2).await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let request = request_generator::info();
    info_stream_writer.write(request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();

    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `2`, tendermint will next call
    // `begin_block` with `block_height = 3`
    let request = request_generator::begin_block(3, 4u64.to_be_bytes().to_vec());
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint may call multiple `deliver_tx`
    let request = request_generator::deliver_tx(5);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    let request = request_generator::deliver_tx(6);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // After all the transactions are delivered, tendermint will call `end_block`
    let request = request_generator::end_block(3);
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));

    // Finally, tendermint will call `commit`
    let request = request_generator::commit();
    consensus_stream_writer.write(request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Commit(_)));
}

async fn call_after_startup(request: Request, state: Option<(u64, i64)>) -> Response {
    let (info_stream, consensus_stream) = match state {
        None => initialize_server().await,
        Some((counter, block_height)) => initialize_server_with_state(counter, block_height).await,
    };

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let info_request = request_generator::info();
    info_stream_writer.write(info_request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Send provided request
    consensus_stream_writer.write(request).await.unwrap();
    consensus_stream_reader.read().await.unwrap().unwrap()
}

#[tokio::test]
async fn can_call_init_chain_after_startup() {
    let response = call_after_startup(request_generator::init_chain(), None).await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_after_startup() {
    let response =
        call_after_startup(request_generator::begin_block(0, Default::default()), None).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`BeginBlock` cannot be called after NotInitialized"
    ));
}

#[tokio::test]
async fn cannot_call_deliver_tx_after_startup() {
    let response = call_after_startup(request_generator::deliver_tx(0), None).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`DeliverTx` cannot be called after NotInitialized"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_after_startup() {
    let response = call_after_startup(request_generator::end_block(0), None).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`EndBlock` cannot be called after NotInitialized"
    ));
}

#[tokio::test]
async fn cannot_call_commit_after_startup() {
    let response = call_after_startup(request_generator::commit(), None).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`Commit` cannot be called after NotInitialized"
    ));
}

#[tokio::test]
async fn cannot_call_init_chain_after_startup_with_state() {
    let response = call_after_startup(request_generator::init_chain(), Some((1, 1))).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Received `InitChain` call when chain is already initialized"
    ));
}

#[tokio::test]
async fn cannot_call_deliver_tx_after_startup_with_state() {
    let response = call_after_startup(request_generator::deliver_tx(0), Some((1, 1))).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`DeliverTx` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_after_startup_with_state() {
    let response = call_after_startup(request_generator::end_block(0), Some((1, 1))).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`EndBlock` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn cannot_call_commit_after_startup_with_state() {
    let response = call_after_startup(request_generator::commit(), Some((1, 1))).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`Commit` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn can_call_begin_block_after_startup_with_state() {
    let response = call_after_startup(
        request_generator::begin_block(2, 1u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_with_different_block_height_after_startup_with_state() {
    let response = call_after_startup(
        request_generator::begin_block(3, 1u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected height 2 in `BeginBlock` request. Got 3"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_with_different_app_hash_after_startup_with_state() {
    let response = call_after_startup(
        request_generator::begin_block(2, 2u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected app hash [0, 0, 0, 0, 0, 0, 0, 1] in `BeginBlock`. Got [0, 0, 0, 0, 0, 0, 0, 2]"
    ));
}

async fn call_after_begin_block(request: Request) -> Response {
    let (info_stream, consensus_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let info_request = request_generator::info();
    info_stream_writer.write(info_request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let init_chain_request = request_generator::init_chain();
    consensus_stream_writer
        .write(init_chain_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let begin_block_request = request_generator::begin_block(1, Default::default());
    consensus_stream_writer
        .write(begin_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Send provided request
    consensus_stream_writer.write(request).await.unwrap();
    consensus_stream_reader.read().await.unwrap().unwrap()
}

#[tokio::test]
async fn cannot_call_init_chain_after_begin_block() {
    let response = call_after_begin_block(request_generator::init_chain()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Received `InitChain` call when chain is already initialized"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_after_begin_block() {
    let response =
        call_after_begin_block(request_generator::begin_block(2, Default::default())).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: BeginBlock }"
    ));
}

#[tokio::test]
async fn cannot_call_commit_after_begin_block() {
    let response = call_after_begin_block(request_generator::commit()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Commit cannot be called after BeginBlock"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_with_different_block_height_after_begin_block() {
    let response = call_after_begin_block(request_generator::end_block(2)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected `EndBlock` for height 1. But received for 2"
    ));
}

#[tokio::test]
async fn can_call_deliver_tx_after_begin_block() {
    let response = call_after_begin_block(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));
}

#[tokio::test]
async fn can_call_end_block_after_begin_block() {
    let response = call_after_begin_block(request_generator::end_block(1)).await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));
}

async fn call_after_deliver_tx(request: Request) -> Response {
    let (info_stream, consensus_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let info_request = request_generator::info();
    info_stream_writer.write(info_request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let init_chain_request = request_generator::init_chain();
    consensus_stream_writer
        .write(init_chain_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let begin_block_request = request_generator::begin_block(1, Default::default());
    consensus_stream_writer
        .write(begin_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint will call `deliver_tx`
    let deliver_tx_request = request_generator::deliver_tx(1);
    consensus_stream_writer
        .write(deliver_tx_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // Send provided request
    consensus_stream_writer.write(request).await.unwrap();
    consensus_stream_reader.read().await.unwrap().unwrap()
}

#[tokio::test]
async fn cannot_call_init_chain_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::init_chain()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Received `InitChain` call when chain is already initialized"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: DeliverTx }"
    ));
}

#[tokio::test]
async fn cannot_call_commit_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::commit()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Commit cannot be called after DeliverTx"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_with_different_height_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::end_block(2)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected `EndBlock` for height 1. But received for 2"
    ));
}

#[tokio::test]
async fn can_call_deliver_tx_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));
}

#[tokio::test]
async fn can_call_end_block_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::end_block(1)).await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));
}

async fn call_after_end_block(request: Request) -> Response {
    let (info_stream, consensus_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let info_request = request_generator::info();
    info_stream_writer.write(info_request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let init_chain_request = request_generator::init_chain();
    consensus_stream_writer
        .write(init_chain_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let begin_block_request = request_generator::begin_block(1, Default::default());
    consensus_stream_writer
        .write(begin_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint will call `deliver_tx`
    let deliver_tx_request = request_generator::deliver_tx(1);
    consensus_stream_writer
        .write(deliver_tx_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // Next, tendermint will call `end_block`
    let end_block_request = request_generator::end_block(1);
    consensus_stream_writer
        .write(end_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));

    // Send provided request
    consensus_stream_writer.write(request).await.unwrap();
    consensus_stream_reader.read().await.unwrap().unwrap()
}

#[tokio::test]
async fn cannot_call_init_chain_after_end_block() {
    let response = call_after_end_block(request_generator::init_chain()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Received `InitChain` call when chain is already initialized"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_after_end_block() {
    let response = call_after_end_block(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: EndBlock }"
    ));
}

#[tokio::test]
async fn cannot_call_deliver_tx_after_end_block() {
    let response = call_after_end_block(request_generator::deliver_tx(2)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "DeliverTx cannot be called after EndBlock"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_after_end_block() {
    let response = call_after_end_block(request_generator::end_block(1)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "EndBlock cannot be called after EndBlock"
    ));
}

#[tokio::test]
async fn can_call_commit_after_end_block() {
    let response = call_after_end_block(request_generator::commit()).await;
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Commit(_)));
}

async fn call_after_commit(request: Request) -> Response {
    let (info_stream, consensus_stream) = initialize_server().await;

    let (mut info_stream_reader, mut info_stream_writer) = get_stream_pair(info_stream);
    let (mut consensus_stream_reader, mut consensus_stream_writer) =
        get_stream_pair(consensus_stream);

    // First, tendermint calls `info` to get information about ABCI application
    let info_request = request_generator::info();
    info_stream_writer.write(info_request).await.unwrap();
    let response: Response = info_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Info(_)));

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let init_chain_request = request_generator::init_chain();
    consensus_stream_writer
        .write(init_chain_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::InitChain(_)
    ));

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let begin_block_request = request_generator::begin_block(1, Default::default());
    consensus_stream_writer
        .write(begin_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));

    // Next, tendermint will call `deliver_tx`
    let deliver_tx_request = request_generator::deliver_tx(1);
    consensus_stream_writer
        .write(deliver_tx_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::DeliverTx(_)
    ));

    // Next, tendermint will call `end_block`
    let end_block_request = request_generator::end_block(1);
    consensus_stream_writer
        .write(end_block_request)
        .await
        .unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::EndBlock(_)
    ));

    // Next, tendermint will call `commit`
    let commit_request = request_generator::commit();
    consensus_stream_writer.write(commit_request).await.unwrap();
    let response: Response = consensus_stream_reader.read().await.unwrap().unwrap();
    assert!(response.value.is_some());
    assert!(matches!(response.value.unwrap(), ResponseValue::Commit(_)));

    // Send provided request
    consensus_stream_writer.write(request).await.unwrap();
    consensus_stream_reader.read().await.unwrap().unwrap()
}

#[tokio::test]
async fn cannot_call_init_chain_after_commit() {
    let response = call_after_commit(request_generator::init_chain()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Received `InitChain` call when chain is already initialized"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_with_different_height_after_commit() {
    let response = call_after_commit(request_generator::begin_block(
        3,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected height 2 in `BeginBlock` request. Got 3"
    ));
}

#[tokio::test]
async fn cannot_call_begin_block_with_different_app_hash_after_commit() {
    let response = call_after_commit(request_generator::begin_block(
        2,
        2u64.to_be_bytes().to_vec(),
    ))
    .await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "Expected app hash [0, 0, 0, 0, 0, 0, 0, 1] in `BeginBlock`. Got [0, 0, 0, 0, 0, 0, 0, 2]"
    ));
}

#[tokio::test]
async fn cannot_call_deliver_tx_after_commit() {
    let response = call_after_commit(request_generator::deliver_tx(2)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`DeliverTx` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn cannot_call_end_block_after_commit() {
    let response = call_after_commit(request_generator::end_block(2)).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`EndBlock` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn cannot_call_commit_after_commit() {
    let response = call_after_commit(request_generator::commit()).await;

    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::Exception(ResponseException { error }) if error ==
            "`Commit` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
    ));
}

#[tokio::test]
async fn can_call_begin_block_after_commit() {
    let response = call_after_commit(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;
    assert!(response.value.is_some());
    assert!(matches!(
        response.value.unwrap(),
        ResponseValue::BeginBlock(_)
    ));
}
