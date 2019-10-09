use std::time::Duration;

use protobuf::well_known_types::Timestamp;

use crate::proto::abci::{
    BlockID as ProtoBlockId, BlockParams as ProtoBlockParams,
    ConsensusParams as ProtoConsensusParams, Event as ProtoEvent, Evidence as ProtoEvidence,
    EvidenceParams as ProtoEvidenceParams, Header as ProtoHeader,
    LastCommitInfo as ProtoLastCommitInfo, PartSetHeader as ProtoPartSetHeader,
    PubKey as ProtoPublicKey, Validator as ProtoValidator, ValidatorParams as ProtoValidatorParams,
    ValidatorUpdate as ProtoValidatorUpdate, Version as ProtoVersion, VoteInfo as ProtoVoteInfo,
};
use crate::proto::merkle::{Proof as ProtoProof, ProofOp as ProtoProofOp};
use crate::proto::types::KVPair as ProtoKeyValuePair;

#[derive(Debug, Default)]
pub struct ConsensusParams {
    /// Parameters limiting the size of a block and time between consecutive blocks
    pub block: Option<BlockParams>,
    /// Parameters limiting the validity of evidence of byzantine behavior
    pub evidence: Option<EvidenceParams>,
    /// Parameters limiting the types of pubkeys validators can use
    pub validator: Option<ValidatorParams>,
}

impl From<ConsensusParams> for ProtoConsensusParams {
    fn from(consensus_params: ConsensusParams) -> ProtoConsensusParams {
        let mut proto_consensus_params = ProtoConsensusParams::new();
        proto_consensus_params.block = consensus_params.block.map(Into::into).into();
        proto_consensus_params.evidence = consensus_params.evidence.map(Into::into).into();
        proto_consensus_params.validator = consensus_params.validator.map(Into::into).into();
        proto_consensus_params
    }
}

#[derive(Debug, Default)]
pub struct BlockParams {
    /// Max size of a block, in bytes
    pub max_bytes: i64,
    /// Max sum of GasWanted in a proposed block
    ///
    /// # Note
    ///
    /// Blocks that violate this may be committed if there are Byzantine proposers. It's the application's
    /// responsibility to handle this when processing a block!
    pub max_gas: i64,
}

impl From<BlockParams> for ProtoBlockParams {
    fn from(block_params: BlockParams) -> ProtoBlockParams {
        let mut proto_block_params = ProtoBlockParams::new();
        proto_block_params.max_bytes = block_params.max_bytes;
        proto_block_params.max_gas = block_params.max_gas;
        proto_block_params
    }
}

#[derive(Debug, Default)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks. Evidence older than this is considered stale and ignored
    ///
    /// # Note
    ///
    /// - This should correspond with an app's "unbonding period" or other similar mechanism for handling
    ///   Nothing-At-Stake attacks.
    /// - This should change to time (instead of blocks)!
    pub max_age: i64,
}

impl From<EvidenceParams> for ProtoEvidenceParams {
    fn from(evidence_params: EvidenceParams) -> ProtoEvidenceParams {
        let mut proto_evidence_params = ProtoEvidenceParams::new();
        proto_evidence_params.max_age = evidence_params.max_age;
        proto_evidence_params
    }
}

#[derive(Debug, Default)]
pub struct ValidatorParams {
    /// List of accepted public key types (uses same naming as `PublicKey.public_key_type`)
    pub public_key_types: Vec<String>,
}

impl From<ValidatorParams> for ProtoValidatorParams {
    fn from(validator_params: ValidatorParams) -> ProtoValidatorParams {
        let mut proto_validator_params = ProtoValidatorParams::new();
        proto_validator_params.pub_key_types = validator_params.public_key_types.into();
        proto_validator_params
    }
}

#[derive(Debug, Default)]
pub struct ValidatorUpdate {
    /// Public key of the validator
    pub public_key: Option<PublicKey>,
    /// Voting power of the validator
    pub power: i64,
}

impl From<ValidatorUpdate> for ProtoValidatorUpdate {
    fn from(validator_update: ValidatorUpdate) -> ProtoValidatorUpdate {
        let mut proto_validator_update = ProtoValidatorUpdate::new();
        proto_validator_update.pub_key = validator_update.public_key.map(Into::into).into();
        proto_validator_update.power = validator_update.power;
        proto_validator_update
    }
}

#[derive(Debug, Default)]
pub struct PublicKey {
    /// Type of the public key. A simple string like "ed25519" (in the future, may indicate a serialization algorithm to
    /// parse the Data, for instance "amino")
    pub public_key_type: String,
    /// Public key data. For a simple public key, it's just the raw bytes. If the `public_key_type` indicates an
    /// encoding algorithm, this is the encoded public key.
    pub data: Vec<u8>,
}

impl From<PublicKey> for ProtoPublicKey {
    fn from(public_key: PublicKey) -> ProtoPublicKey {
        let mut proto_public_key = ProtoPublicKey::new();
        proto_public_key.field_type = public_key.public_key_type;
        proto_public_key.data = public_key.data;
        proto_public_key
    }
}

#[derive(Debug, Default)]
pub struct Proof {
    /// List of chained Merkle proofs, of possibly different types
    ///
    /// # Note
    ///
    /// - The Merkle root of one op is the value being proven in the next op
    /// - The Merkle root of the final op should equal the ultimate root hash being verified against
    pub ops: Vec<ProofOp>,
}

impl From<Proof> for ProtoProof {
    fn from(proof: Proof) -> ProtoProof {
        let mut proto_proof = ProtoProof::new();
        proto_proof.ops = proof
            .ops
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ProtoProofOp>>()
            .into();
        proto_proof
    }
}

#[derive(Debug, Default)]
pub struct ProofOp {
    /// Type of Merkle proof and how it's encoded
    pub proof_op_type: String,
    /// Key in the Merkle tree that this proof is for
    pub key: Vec<u8>,
    /// Encoded Merkle proof for the key
    pub data: Vec<u8>,
}

impl From<ProofOp> for ProtoProofOp {
    fn from(proof_op: ProofOp) -> ProtoProofOp {
        let mut proto_proof_op = ProtoProofOp::new();
        proto_proof_op.field_type = proof_op.proof_op_type;
        proto_proof_op.key = proof_op.key;
        proto_proof_op.data = proof_op.data;
        proto_proof_op
    }
}

#[derive(Debug, Default)]
pub struct Version {
    /// Protocol version of the blockchain data structures
    pub block: u64,
    /// Protocol version of the application
    pub app: u64,
}

impl From<Version> for ProtoVersion {
    fn from(version: Version) -> ProtoVersion {
        let mut proto_version = ProtoVersion::new();
        proto_version.Block = version.block;
        proto_version.App = version.app;
        proto_version
    }
}

#[derive(Debug, Default)]
pub struct PartSetHeader {
    pub total: i32,
    pub hash: Vec<u8>,
}

impl From<PartSetHeader> for ProtoPartSetHeader {
    fn from(part_set_header: PartSetHeader) -> ProtoPartSetHeader {
        let mut proto_part_set_header = ProtoPartSetHeader::new();
        proto_part_set_header.total = part_set_header.total;
        proto_part_set_header.hash = part_set_header.hash;
        proto_part_set_header
    }
}

#[derive(Debug, Default)]
pub struct BlockId {
    pub hash: Vec<u8>,
    pub parts_header: Option<PartSetHeader>,
}

impl From<BlockId> for ProtoBlockId {
    fn from(block_id: BlockId) -> ProtoBlockId {
        let mut proto_block_id = ProtoBlockId::new();
        proto_block_id.hash = block_id.hash;
        proto_block_id.parts_header = block_id.parts_header.map(Into::into).into();
        proto_block_id
    }
}

#[derive(Debug, Default)]
pub struct Header {
    /// Version of the blockchain and the application
    pub version: Option<Version>,
    /// ID of the blockchain
    pub chain_id: String,
    /// Height of the block in the chain
    pub height: i64,
    /// Time of the previous block. For heights > 1, it's the weighted median of the timestamps of the valid votes in
    /// the `block.last_commit`. For height == 1, it's genesis time. (duration since epoch)
    pub time: Option<Duration>,
    /// Number of transactions in the block
    pub num_txs: i64,
    /// Total number of transactions in the blockchain until now
    pub total_txs: i64,
    /// Hash of the previous (parent) block
    pub last_block_id: Option<BlockId>,
    /// Hash of the previous block's commit
    pub last_commit_hash: Vec<u8>,
    /// Hash if data in the block
    pub data_hash: Vec<u8>,
    /// Hash of the validator set for this block
    pub validators_hash: Vec<u8>,
    /// Hash of the validator set for the next block
    pub next_validators_hash: Vec<u8>,
    /// Hash of the consensus parameters for this block
    pub consensus_hash: Vec<u8>,
    /// Data returned by the last call to `Commit` - typically the Merkle root of the application state after executing
    /// the previous block's transactions
    pub app_hash: Vec<u8>,
    /// Hash of the ABCI results returned by the last block
    pub last_results_hash: Vec<u8>,
    /// Hash of the evidence included in this block
    pub evidence_hash: Vec<u8>,
    /// Original proposer for the block
    pub proposer_address: Vec<u8>,
}

impl From<Header> for ProtoHeader {
    fn from(header: Header) -> ProtoHeader {
        let mut proto_header = ProtoHeader::new();
        proto_header.version = header.version.map(Into::into).into();
        proto_header.chain_id = header.chain_id;
        proto_header.height = header.height;
        proto_header.time = header
            .time
            .map(|time| {
                let mut timestamp = Timestamp::new();
                timestamp.seconds = time.as_secs() as i64;
                timestamp.nanos = time.subsec_nanos() as i32;
                timestamp
            })
            .into();
        proto_header.num_txs = header.num_txs;
        proto_header.total_txs = header.total_txs;
        proto_header.last_block_id = header.last_block_id.map(Into::into).into();
        proto_header.last_commit_hash = header.last_commit_hash;
        proto_header.data_hash = header.data_hash;
        proto_header.validators_hash = header.validators_hash;
        proto_header.next_validators_hash = header.next_validators_hash;
        proto_header.consensus_hash = header.consensus_hash;
        proto_header.app_hash = header.app_hash;
        proto_header.last_results_hash = header.last_results_hash;
        proto_header.evidence_hash = header.evidence_hash;
        proto_header.proposer_address = header.proposer_address;
        proto_header
    }
}

#[derive(Debug, Default)]
pub struct Validator {
    /// Address of the validator (hash of the public key)
    pub address: Vec<u8>,
    /// Voting power of the validator
    pub power: i64,
}

impl From<Validator> for ProtoValidator {
    fn from(validator: Validator) -> ProtoValidator {
        let mut proto_validator = ProtoValidator::new();
        proto_validator.address = validator.address;
        proto_validator.power = validator.power;
        proto_validator
    }
}

#[derive(Debug, Default)]
pub struct VoteInfo {
    /// A validator
    pub validator: Option<Validator>,
    /// Indicates whether or not the validator signed the last block
    pub signed_last_block: bool,
}

impl From<VoteInfo> for ProtoVoteInfo {
    fn from(vote_info: VoteInfo) -> ProtoVoteInfo {
        let mut proto_vote_info = ProtoVoteInfo::new();
        proto_vote_info.validator = vote_info.validator.map(Into::into).into();
        proto_vote_info.signed_last_block = vote_info.signed_last_block;
        proto_vote_info
    }
}

#[derive(Debug, Default)]
pub struct LastCommitInfo {
    /// Commit round
    pub round: i32,
    /// List of validators addresses in the last validator set with their voting power and whether or not they signed a
    /// vote.
    pub votes: Vec<VoteInfo>,
}

impl From<LastCommitInfo> for ProtoLastCommitInfo {
    fn from(last_commit_info: LastCommitInfo) -> ProtoLastCommitInfo {
        let mut proto_last_commit_info = ProtoLastCommitInfo::new();
        proto_last_commit_info.round = last_commit_info.round;
        proto_last_commit_info.votes = last_commit_info
            .votes
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ProtoVoteInfo>>()
            .into();
        proto_last_commit_info
    }
}

#[derive(Debug, Default)]
pub struct Evidence {
    /// Type of the evidence. A hierarchical path like "duplicate/vote".
    pub evidence_type: String,
    /// The offending validator
    pub validator: Option<Validator>,
    /// Height when the offense was committed
    pub height: i64,
    /// Time of the block at height Height. It is the proposer's local time when block was created (duration since
    /// epoch)
    pub time: Option<Duration>,
    /// Total voting power of the validator set at `height`
    pub total_voting_power: i64,
}

impl From<Evidence> for ProtoEvidence {
    fn from(evidence: Evidence) -> ProtoEvidence {
        let mut proto_evidence = ProtoEvidence::new();
        proto_evidence.field_type = evidence.evidence_type;
        proto_evidence.validator = evidence.validator.map(Into::into).into();
        proto_evidence.height = evidence.height;
        proto_evidence.time = evidence
            .time
            .map(|time| {
                let mut timestamp = Timestamp::new();
                timestamp.seconds = time.as_secs() as i64;
                timestamp.nanos = time.subsec_nanos() as i32;
                timestamp
            })
            .into();
        proto_evidence.total_voting_power = evidence.total_voting_power;
        proto_evidence
    }
}

#[derive(Debug, Default)]
pub struct KeyValuePair {
    /// Key
    pub key: Vec<u8>,
    /// Value
    pub value: Vec<u8>,
}

impl From<KeyValuePair> for ProtoKeyValuePair {
    fn from(pair: KeyValuePair) -> ProtoKeyValuePair {
        let mut proto_pair = ProtoKeyValuePair::new();
        proto_pair.key = pair.key;
        proto_pair.value = pair.value;
        proto_pair
    }
}

#[derive(Debug, Default)]
pub struct Event {
    /// Event type
    pub event_type: String,
    /// Attributes
    pub attributes: Vec<KeyValuePair>,
}

impl From<Event> for ProtoEvent {
    fn from(event: Event) -> ProtoEvent {
        let mut proto_event = ProtoEvent::new();
        proto_event.field_type = event.event_type;
        proto_event.attributes = event
            .attributes
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ProtoKeyValuePair>>()
            .into();
        proto_event
    }
}
