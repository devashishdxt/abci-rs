var searchIndex = JSON.parse('{\
"abci":{"doc":"A Rust crate for creating ABCI applications.","i":[[23,"async_trait","abci","Utility macro for implementing `Consensus`, `Mempool` and…",null,null],[3,"Server","","ABCI Server",null,null],[4,"Address","","Address of ABCI Server",null,null],[13,"Tcp","","TCP Address",0,null],[13,"Uds","","UDS Address",0,null],[11,"new","","Creates a new instance of `Server`",1,[[],["result",6]]],[11,"run","","Starts ABCI server",1,[[]]],[0,"types","","Types used in ABCI",null,null],[3,"BeginBlockRequest","abci::types","",null,null],[12,"hash","","Block\'s hash. This can be derived from the block header",2,null],[12,"header","","Block header",2,null],[12,"last_commit_info","","Info about the last commit, including the round, and the…",2,null],[12,"byzantine_validators","","List of evidence of validators that acted maliciously",2,null],[3,"BeginBlockResponse","","",null,null],[12,"events","","Events for filtering and indexing",3,null],[3,"CheckTxRequest","","",null,null],[12,"tx","","The request transaction bytes",4,null],[12,"check_type","","Denotes if this is a new request of a re-check request",4,null],[3,"CheckTxResponse","","",null,null],[12,"data","","Result bytes, if any.",5,null],[12,"log","","Output of application\'s logger (may be non-deterministic)",5,null],[12,"info","","Additional information (may be non-deterministic)",5,null],[12,"gas_wanted","","Amount of gas requested for transaction",5,null],[12,"gas_used","","Amount of gas consumed by transaction",5,null],[12,"events","","Events for filtering and indexing",5,null],[3,"CommitResponse","","",null,null],[12,"data","","The Merkle root hash of the application state",6,null],[12,"retain_height","","Automatically remove blocks below this height",6,null],[3,"DeliverTxRequest","","",null,null],[12,"tx","","The request transaction bytes",7,null],[3,"DeliverTxResponse","","",null,null],[12,"data","","Result bytes, if any.",8,null],[12,"log","","Output of application\'s logger (may be non-deterministic)",8,null],[12,"info","","Additional information (may be non-deterministic)",8,null],[12,"gas_wanted","","Amount of gas requested for transaction",8,null],[12,"gas_used","","Amount of gas consumed by transaction",8,null],[12,"events","","Events for filtering and indexing",8,null],[3,"EndBlockRequest","","",null,null],[12,"height","","Height of the block just executed",9,null],[3,"EndBlockResponse","","",null,null],[12,"validator_updates","","Changes to validator set (set voting power to 0 to remove)",10,null],[12,"consensus_param_updates","","Changes to consensus-critical time, size, and other…",10,null],[12,"events","","Events for filtering and indexing",10,null],[3,"InfoRequest","","",null,null],[12,"version","","Tendermint software semantic version",11,null],[12,"block_version","","Tendermint block protocol version",11,null],[12,"p2p_version","","Tendermint P2P protocol version",11,null],[3,"InfoResponse","","",null,null],[12,"data","","Some arbitrary information",12,null],[12,"version","","Application software semantic version",12,null],[12,"app_version","","Application protocol version",12,null],[12,"last_block_height","","Latest block for which the app has called Commit",12,null],[12,"last_block_app_hash","","Latest result of Commit",12,null],[3,"InitChainRequest","","",null,null],[12,"time","","Genesis time (duration since epoch)",13,null],[12,"chain_id","","ID of blockchain",13,null],[12,"consensus_params","","Initial consensus-critical parameters",13,null],[12,"validators","","Initial genesis validators",13,null],[12,"app_state_bytes","","Serialized initial application state (amino-encoded JSON…",13,null],[3,"InitChainResponse","","",null,null],[12,"consensus_params","","Initial consensus-critical parameters",14,null],[12,"validators","","Initial validator set (if non empty)",14,null],[3,"ConsensusParams","","",null,null],[12,"block","","Parameters limiting the size of a block and time between…",15,null],[12,"evidence","","Parameters limiting the validity of evidence of byzantine…",15,null],[12,"validator","","Parameters limiting the types of pubkeys validators can use",15,null],[3,"BlockParams","","",null,null],[12,"max_bytes","","Max size of a block, in bytes",16,null],[12,"max_gas","","Max sum of GasWanted in a proposed block",16,null],[3,"EvidenceParams","","Tendermint adopts a hybrid approach to check validity of…",null,null],[12,"max_age_num_blocks","","Max age of evidence, in blocks. Evidence older than this…",17,null],[12,"max_age_duration","","Max age of evidence, in time duration. Evidence older than…",17,null],[3,"ValidatorParams","","",null,null],[12,"public_key_types","","List of accepted public key types (uses same naming as…",18,null],[3,"ValidatorUpdate","","",null,null],[12,"public_key","","Public key of the validator",19,null],[12,"power","","Voting power of the validator",19,null],[3,"PublicKey","","",null,null],[12,"public_key_type","","Type of the public key. A simple string like \\\"ed25519\\\" (in…",20,null],[12,"data","","Public key data. For a simple public key, it\'s just the…",20,null],[3,"Proof","","",null,null],[12,"ops","","List of chained Merkle proofs, of possibly different types",21,null],[3,"ProofOp","","",null,null],[12,"proof_op_type","","Type of Merkle proof and how it\'s encoded",22,null],[12,"key","","Key in the Merkle tree that this proof is for",22,null],[12,"data","","Encoded Merkle proof for the key",22,null],[3,"Version","","",null,null],[12,"block","","Protocol version of the blockchain data structures",23,null],[12,"app","","Protocol version of the application",23,null],[3,"PartSetHeader","","",null,null],[12,"total","","",24,null],[12,"hash","","",24,null],[3,"BlockId","","",null,null],[12,"hash","","",25,null],[12,"parts_header","","",25,null],[3,"Header","","",null,null],[12,"version","","Version of the blockchain and the application",26,null],[12,"chain_id","","ID of the blockchain",26,null],[12,"height","","Height of the block in the chain",26,null],[12,"time","","Time of the previous block. For heights > 1, it\'s the…",26,null],[12,"last_block_id","","Hash of the previous (parent) block",26,null],[12,"last_commit_hash","","Hash of the previous block\'s commit",26,null],[12,"data_hash","","Hash if data in the block",26,null],[12,"validators_hash","","Hash of the validator set for this block",26,null],[12,"next_validators_hash","","Hash of the validator set for the next block",26,null],[12,"consensus_hash","","Hash of the consensus parameters for this block",26,null],[12,"app_hash","","Data returned by the last call to `Commit` - typically the…",26,null],[12,"last_results_hash","","Hash of the ABCI results returned by the last block",26,null],[12,"evidence_hash","","Hash of the evidence included in this block",26,null],[12,"proposer_address","","Original proposer for the block",26,null],[3,"Validator","","",null,null],[12,"address","","Address of the validator (hash of the public key)",27,null],[12,"power","","Voting power of the validator",27,null],[3,"VoteInfo","","",null,null],[12,"validator","","A validator",28,null],[12,"signed_last_block","","Indicates whether or not the validator signed the last block",28,null],[3,"LastCommitInfo","","",null,null],[12,"round","","Commit round",29,null],[12,"votes","","List of validators addresses in the last validator set…",29,null],[3,"Evidence","","",null,null],[12,"evidence_type","","Type of the evidence. A hierarchical path like…",30,null],[12,"validator","","The offending validator",30,null],[12,"height","","Height when the offense was committed",30,null],[12,"time","","Time of the block at height Height. It is the proposer\'s…",30,null],[12,"total_voting_power","","Total voting power of the validator set at `height`",30,null],[3,"Pair","","",null,null],[12,"key","","Key",31,null],[12,"value","","Value",31,null],[3,"Event","","",null,null],[12,"event_type","","Event type",32,null],[12,"attributes","","Attributes",32,null],[3,"QueryRequest","","",null,null],[12,"data","","Raw query bytes (can be used with or in lieu of `path`)",33,null],[12,"path","","Path of request, like an HTTP GET path (can be used with…",33,null],[12,"height","","Block height for which you want the query (default=0…",33,null],[12,"prove","","Return Merkle proof with response if possible",33,null],[3,"QueryResponse","","",null,null],[12,"log","","Output of application\'s logger (may be non-deterministic)",34,null],[12,"info","","Additional information (may be non-deterministic)",34,null],[12,"index","","Index of the key in the tree",34,null],[12,"key","","Key of the matching data",34,null],[12,"value","","Value of the matching data",34,null],[12,"proof","","Serialized proof for the value data, if requested, to be…",34,null],[12,"height","","Block height from which data was derived",34,null],[3,"SetOptionRequest","","",null,null],[12,"key","","Key to set",35,null],[12,"value","","Value to set for key",35,null],[3,"SetOptionResponse","","",null,null],[12,"log","","Output of application\'s logger (may be non-deterministic)",36,null],[12,"info","","Additional information (may be non-deterministic)",36,null],[3,"Error","","ABCI Error",null,null],[12,"code","","Error code",37,null],[12,"codespace","","Namespace for error code",37,null],[12,"log","","Output of application\'s logger (may be non-deterministic)",37,null],[12,"info","","Additional information (may be non-deterministic)",37,null],[4,"CheckTxType","","",null,null],[13,"New","","Denotes that the transaction has never been checked",38,null],[13,"Recheck","","Denotes that the transaction was already checked and…",38,null],[6,"Result","","ABCI Result",null,null],[8,"Consensus","abci","Trait for managing consensus of blockchain.",null,null],[10,"init_chain","","Called once upon genesis. Usually used to establish…",39,[[["initchainrequest",3]],[["box",3],["pin",3]]]],[10,"begin_block","","Signals the beginning of a new block. Called prior to any…",39,[[["beginblockrequest",3]],[["pin",3],["box",3]]]],[10,"deliver_tx","","Execute the transaction in full. The workhorse of the…",39,[[["delivertxrequest",3]],[["box",3],["pin",3]]]],[10,"end_block","","Signals the end of a block. Called after all transactions,…",39,[[["endblockrequest",3]],[["pin",3],["box",3]]]],[10,"commit","","Persist the application state.",39,[[],[["pin",3],["box",3]]]],[11,"flush","","Signals that messages queued on the client should be…",39,[[],[["pin",3],["box",3]]]],[8,"Info","","Trait for initialization and for queries from the user.",null,null],[11,"echo","","Echo a string to test abci client/server implementation.",40,[[["string",3]],[["box",3],["pin",3]]]],[10,"info","","Return information about the application state.",40,[[["inforequest",3]],[["pin",3],["box",3]]]],[11,"set_option","","Set non-consensus critical application specific options.",40,[[["setoptionrequest",3]],[["pin",3],["box",3]]]],[11,"query","","Query for data from the application at current or past…",40,[[["queryrequest",3]],[["pin",3],["box",3]]]],[8,"Mempool","","Trait for managing tendermint\'s mempool.",null,null],[10,"check_tx","","Guardian of the mempool: every node runs CheckTx before…",41,[[["checktxrequest",3]],[["box",3],["pin",3]]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","abci::types","",2,[[]]],[11,"into","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"try_into","","",2,[[],["result",4]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"from","","",3,[[]]],[11,"into","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"try_into","","",3,[[],["result",4]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"from","","",5,[[]]],[11,"into","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"try_into","","",5,[[],["result",4]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"from","","",6,[[]]],[11,"into","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"try_into","","",6,[[],["result",4]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"from","","",7,[[]]],[11,"into","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"try_into","","",7,[[],["result",4]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"from","","",8,[[]]],[11,"into","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"try_into","","",8,[[],["result",4]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"from","","",9,[[]]],[11,"into","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"try_into","","",9,[[],["result",4]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"from","","",10,[[]]],[11,"into","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"try_into","","",10,[[],["result",4]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"from","","",11,[[]]],[11,"into","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"try_into","","",11,[[],["result",4]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"from","","",12,[[]]],[11,"into","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"try_into","","",12,[[],["result",4]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"from","","",13,[[]]],[11,"into","","",13,[[]]],[11,"try_from","","",13,[[],["result",4]]],[11,"try_into","","",13,[[],["result",4]]],[11,"borrow","","",13,[[]]],[11,"borrow_mut","","",13,[[]]],[11,"type_id","","",13,[[],["typeid",3]]],[11,"from","","",14,[[]]],[11,"into","","",14,[[]]],[11,"try_from","","",14,[[],["result",4]]],[11,"try_into","","",14,[[],["result",4]]],[11,"borrow","","",14,[[]]],[11,"borrow_mut","","",14,[[]]],[11,"type_id","","",14,[[],["typeid",3]]],[11,"from","","",15,[[]]],[11,"into","","",15,[[]]],[11,"try_from","","",15,[[],["result",4]]],[11,"try_into","","",15,[[],["result",4]]],[11,"borrow","","",15,[[]]],[11,"borrow_mut","","",15,[[]]],[11,"type_id","","",15,[[],["typeid",3]]],[11,"from","","",16,[[]]],[11,"into","","",16,[[]]],[11,"try_from","","",16,[[],["result",4]]],[11,"try_into","","",16,[[],["result",4]]],[11,"borrow","","",16,[[]]],[11,"borrow_mut","","",16,[[]]],[11,"type_id","","",16,[[],["typeid",3]]],[11,"from","","",17,[[]]],[11,"into","","",17,[[]]],[11,"try_from","","",17,[[],["result",4]]],[11,"try_into","","",17,[[],["result",4]]],[11,"borrow","","",17,[[]]],[11,"borrow_mut","","",17,[[]]],[11,"type_id","","",17,[[],["typeid",3]]],[11,"from","","",18,[[]]],[11,"into","","",18,[[]]],[11,"try_from","","",18,[[],["result",4]]],[11,"try_into","","",18,[[],["result",4]]],[11,"borrow","","",18,[[]]],[11,"borrow_mut","","",18,[[]]],[11,"type_id","","",18,[[],["typeid",3]]],[11,"from","","",19,[[]]],[11,"into","","",19,[[]]],[11,"try_from","","",19,[[],["result",4]]],[11,"try_into","","",19,[[],["result",4]]],[11,"borrow","","",19,[[]]],[11,"borrow_mut","","",19,[[]]],[11,"type_id","","",19,[[],["typeid",3]]],[11,"from","","",20,[[]]],[11,"into","","",20,[[]]],[11,"try_from","","",20,[[],["result",4]]],[11,"try_into","","",20,[[],["result",4]]],[11,"borrow","","",20,[[]]],[11,"borrow_mut","","",20,[[]]],[11,"type_id","","",20,[[],["typeid",3]]],[11,"from","","",21,[[]]],[11,"into","","",21,[[]]],[11,"try_from","","",21,[[],["result",4]]],[11,"try_into","","",21,[[],["result",4]]],[11,"borrow","","",21,[[]]],[11,"borrow_mut","","",21,[[]]],[11,"type_id","","",21,[[],["typeid",3]]],[11,"from","","",22,[[]]],[11,"into","","",22,[[]]],[11,"try_from","","",22,[[],["result",4]]],[11,"try_into","","",22,[[],["result",4]]],[11,"borrow","","",22,[[]]],[11,"borrow_mut","","",22,[[]]],[11,"type_id","","",22,[[],["typeid",3]]],[11,"from","","",23,[[]]],[11,"into","","",23,[[]]],[11,"try_from","","",23,[[],["result",4]]],[11,"try_into","","",23,[[],["result",4]]],[11,"borrow","","",23,[[]]],[11,"borrow_mut","","",23,[[]]],[11,"type_id","","",23,[[],["typeid",3]]],[11,"from","","",24,[[]]],[11,"into","","",24,[[]]],[11,"try_from","","",24,[[],["result",4]]],[11,"try_into","","",24,[[],["result",4]]],[11,"borrow","","",24,[[]]],[11,"borrow_mut","","",24,[[]]],[11,"type_id","","",24,[[],["typeid",3]]],[11,"from","","",25,[[]]],[11,"into","","",25,[[]]],[11,"try_from","","",25,[[],["result",4]]],[11,"try_into","","",25,[[],["result",4]]],[11,"borrow","","",25,[[]]],[11,"borrow_mut","","",25,[[]]],[11,"type_id","","",25,[[],["typeid",3]]],[11,"from","","",26,[[]]],[11,"into","","",26,[[]]],[11,"try_from","","",26,[[],["result",4]]],[11,"try_into","","",26,[[],["result",4]]],[11,"borrow","","",26,[[]]],[11,"borrow_mut","","",26,[[]]],[11,"type_id","","",26,[[],["typeid",3]]],[11,"from","","",27,[[]]],[11,"into","","",27,[[]]],[11,"try_from","","",27,[[],["result",4]]],[11,"try_into","","",27,[[],["result",4]]],[11,"borrow","","",27,[[]]],[11,"borrow_mut","","",27,[[]]],[11,"type_id","","",27,[[],["typeid",3]]],[11,"from","","",28,[[]]],[11,"into","","",28,[[]]],[11,"try_from","","",28,[[],["result",4]]],[11,"try_into","","",28,[[],["result",4]]],[11,"borrow","","",28,[[]]],[11,"borrow_mut","","",28,[[]]],[11,"type_id","","",28,[[],["typeid",3]]],[11,"from","","",29,[[]]],[11,"into","","",29,[[]]],[11,"try_from","","",29,[[],["result",4]]],[11,"try_into","","",29,[[],["result",4]]],[11,"borrow","","",29,[[]]],[11,"borrow_mut","","",29,[[]]],[11,"type_id","","",29,[[],["typeid",3]]],[11,"from","","",30,[[]]],[11,"into","","",30,[[]]],[11,"try_from","","",30,[[],["result",4]]],[11,"try_into","","",30,[[],["result",4]]],[11,"borrow","","",30,[[]]],[11,"borrow_mut","","",30,[[]]],[11,"type_id","","",30,[[],["typeid",3]]],[11,"from","","",31,[[]]],[11,"into","","",31,[[]]],[11,"try_from","","",31,[[],["result",4]]],[11,"try_into","","",31,[[],["result",4]]],[11,"borrow","","",31,[[]]],[11,"borrow_mut","","",31,[[]]],[11,"type_id","","",31,[[],["typeid",3]]],[11,"from","","",32,[[]]],[11,"into","","",32,[[]]],[11,"try_from","","",32,[[],["result",4]]],[11,"try_into","","",32,[[],["result",4]]],[11,"borrow","","",32,[[]]],[11,"borrow_mut","","",32,[[]]],[11,"type_id","","",32,[[],["typeid",3]]],[11,"from","","",33,[[]]],[11,"into","","",33,[[]]],[11,"try_from","","",33,[[],["result",4]]],[11,"try_into","","",33,[[],["result",4]]],[11,"borrow","","",33,[[]]],[11,"borrow_mut","","",33,[[]]],[11,"type_id","","",33,[[],["typeid",3]]],[11,"from","","",34,[[]]],[11,"into","","",34,[[]]],[11,"try_from","","",34,[[],["result",4]]],[11,"try_into","","",34,[[],["result",4]]],[11,"borrow","","",34,[[]]],[11,"borrow_mut","","",34,[[]]],[11,"type_id","","",34,[[],["typeid",3]]],[11,"from","","",35,[[]]],[11,"into","","",35,[[]]],[11,"try_from","","",35,[[],["result",4]]],[11,"try_into","","",35,[[],["result",4]]],[11,"borrow","","",35,[[]]],[11,"borrow_mut","","",35,[[]]],[11,"type_id","","",35,[[],["typeid",3]]],[11,"from","","",36,[[]]],[11,"into","","",36,[[]]],[11,"try_from","","",36,[[],["result",4]]],[11,"try_into","","",36,[[],["result",4]]],[11,"borrow","","",36,[[]]],[11,"borrow_mut","","",36,[[]]],[11,"type_id","","",36,[[],["typeid",3]]],[11,"from","","",37,[[]]],[11,"into","","",37,[[]]],[11,"to_string","","",37,[[],["string",3]]],[11,"try_from","","",37,[[],["result",4]]],[11,"try_into","","",37,[[],["result",4]]],[11,"borrow","","",37,[[]]],[11,"borrow_mut","","",37,[[]]],[11,"type_id","","",37,[[],["typeid",3]]],[11,"from","","",38,[[]]],[11,"into","","",38,[[]]],[11,"try_from","","",38,[[],["result",4]]],[11,"try_into","","",38,[[],["result",4]]],[11,"borrow","","",38,[[]]],[11,"borrow_mut","","",38,[[]]],[11,"type_id","","",38,[[],["typeid",3]]],[11,"from","abci","",0,[[["socketaddr",4]]]],[11,"from","","",0,[[["pathbuf",3]]]],[11,"default","abci::types","",2,[[],["beginblockrequest",3]]],[11,"default","","",3,[[],["beginblockresponse",3]]],[11,"default","","",4,[[],["checktxrequest",3]]],[11,"default","","",38,[[]]],[11,"default","","",5,[[],["checktxresponse",3]]],[11,"default","","",6,[[],["commitresponse",3]]],[11,"default","","",7,[[],["delivertxrequest",3]]],[11,"default","","",8,[[],["delivertxresponse",3]]],[11,"default","","",9,[[],["endblockrequest",3]]],[11,"default","","",10,[[],["endblockresponse",3]]],[11,"default","","",11,[[],["inforequest",3]]],[11,"default","","",12,[[],["inforesponse",3]]],[11,"default","","",13,[[],["initchainrequest",3]]],[11,"default","","",14,[[],["initchainresponse",3]]],[11,"default","","",15,[[],["consensusparams",3]]],[11,"default","","",16,[[],["blockparams",3]]],[11,"default","","",17,[[],["evidenceparams",3]]],[11,"default","","",18,[[],["validatorparams",3]]],[11,"default","","",19,[[],["validatorupdate",3]]],[11,"default","","",20,[[],["publickey",3]]],[11,"default","","",21,[[],["proof",3]]],[11,"default","","",22,[[],["proofop",3]]],[11,"default","","",23,[[],["version",3]]],[11,"default","","",24,[[],["partsetheader",3]]],[11,"default","","",25,[[],["blockid",3]]],[11,"default","","",26,[[],["header",3]]],[11,"default","","",27,[[],["validator",3]]],[11,"default","","",28,[[],["voteinfo",3]]],[11,"default","","",29,[[],["lastcommitinfo",3]]],[11,"default","","",30,[[],["evidence",3]]],[11,"default","","",31,[[],["pair",3]]],[11,"default","","",32,[[],["event",3]]],[11,"default","","",33,[[],["queryrequest",3]]],[11,"default","","",34,[[],["queryresponse",3]]],[11,"default","","",35,[[],["setoptionrequest",3]]],[11,"default","","",36,[[],["setoptionresponse",3]]],[11,"fmt","abci","",0,[[["formatter",3]],["result",6]]],[11,"fmt","abci::types","",2,[[["formatter",3]],["result",6]]],[11,"fmt","","",3,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"fmt","","",38,[[["formatter",3]],["result",6]]],[11,"fmt","","",5,[[["formatter",3]],["result",6]]],[11,"fmt","","",6,[[["formatter",3]],["result",6]]],[11,"fmt","","",7,[[["formatter",3]],["result",6]]],[11,"fmt","","",8,[[["formatter",3]],["result",6]]],[11,"fmt","","",9,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"fmt","","",11,[[["formatter",3]],["result",6]]],[11,"fmt","","",12,[[["formatter",3]],["result",6]]],[11,"fmt","","",13,[[["formatter",3]],["result",6]]],[11,"fmt","","",14,[[["formatter",3]],["result",6]]],[11,"fmt","","",15,[[["formatter",3]],["result",6]]],[11,"fmt","","",16,[[["formatter",3]],["result",6]]],[11,"fmt","","",17,[[["formatter",3]],["result",6]]],[11,"fmt","","",18,[[["formatter",3]],["result",6]]],[11,"fmt","","",19,[[["formatter",3]],["result",6]]],[11,"fmt","","",20,[[["formatter",3]],["result",6]]],[11,"fmt","","",21,[[["formatter",3]],["result",6]]],[11,"fmt","","",22,[[["formatter",3]],["result",6]]],[11,"fmt","","",23,[[["formatter",3]],["result",6]]],[11,"fmt","","",24,[[["formatter",3]],["result",6]]],[11,"fmt","","",25,[[["formatter",3]],["result",6]]],[11,"fmt","","",26,[[["formatter",3]],["result",6]]],[11,"fmt","","",27,[[["formatter",3]],["result",6]]],[11,"fmt","","",28,[[["formatter",3]],["result",6]]],[11,"fmt","","",29,[[["formatter",3]],["result",6]]],[11,"fmt","","",30,[[["formatter",3]],["result",6]]],[11,"fmt","","",31,[[["formatter",3]],["result",6]]],[11,"fmt","","",32,[[["formatter",3]],["result",6]]],[11,"fmt","","",33,[[["formatter",3]],["result",6]]],[11,"fmt","","",34,[[["formatter",3]],["result",6]]],[11,"fmt","","",35,[[["formatter",3]],["result",6]]],[11,"fmt","","",36,[[["formatter",3]],["result",6]]],[11,"fmt","","",37,[[["formatter",3]],["result",6]]],[11,"fmt","","",37,[[["formatter",3]],["result",6]]],[11,"echo","abci","Echo a string to test abci client/server implementation.",40,[[["string",3]],[["box",3],["pin",3]]]],[11,"set_option","","Set non-consensus critical application specific options.",40,[[["setoptionrequest",3]],[["pin",3],["box",3]]]],[11,"query","","Query for data from the application at current or past…",40,[[["queryrequest",3]],[["pin",3],["box",3]]]],[11,"flush","","Signals that messages queued on the client should be…",39,[[],[["pin",3],["box",3]]]]],"p":[[4,"Address"],[3,"Server"],[3,"BeginBlockRequest"],[3,"BeginBlockResponse"],[3,"CheckTxRequest"],[3,"CheckTxResponse"],[3,"CommitResponse"],[3,"DeliverTxRequest"],[3,"DeliverTxResponse"],[3,"EndBlockRequest"],[3,"EndBlockResponse"],[3,"InfoRequest"],[3,"InfoResponse"],[3,"InitChainRequest"],[3,"InitChainResponse"],[3,"ConsensusParams"],[3,"BlockParams"],[3,"EvidenceParams"],[3,"ValidatorParams"],[3,"ValidatorUpdate"],[3,"PublicKey"],[3,"Proof"],[3,"ProofOp"],[3,"Version"],[3,"PartSetHeader"],[3,"BlockId"],[3,"Header"],[3,"Validator"],[3,"VoteInfo"],[3,"LastCommitInfo"],[3,"Evidence"],[3,"Pair"],[3,"Event"],[3,"QueryRequest"],[3,"QueryResponse"],[3,"SetOptionRequest"],[3,"SetOptionResponse"],[3,"Error"],[4,"CheckTxType"],[8,"Consensus"],[8,"Info"],[8,"Mempool"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);