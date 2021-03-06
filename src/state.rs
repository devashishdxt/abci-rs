use crate::types::*;

#[derive(Debug, Default)]
pub struct ConsensusStateValidator {
    state: ConsensusState,
}

impl ConsensusStateValidator {
    pub fn on_info_response(&mut self, info_response: &ResponseInfo) {
        if self.state == ConsensusState::NoInfo {
            let block_height = info_response.last_block_height;

            if block_height == 0 {
                self.state = ConsensusState::NotInitialized;
            } else {
                self.state = ConsensusState::WaitingForBlock {
                    block_height: block_height + 1,
                    app_hash: info_response.last_block_app_hash.clone(),
                };
            }
        }
    }

    pub fn on_init_chain_request(&mut self) -> Result<(), String> {
        if self.state != ConsensusState::NotInitialized {
            return Err("Received `InitChain` call when chain is already initialized".to_string());
        }

        self.state = ConsensusState::InitChain;
        Ok(())
    }

    pub fn on_begin_block_request(
        &mut self,
        begin_block_request: &RequestBeginBlock,
    ) -> Result<(), String> {
        let new_state = match self.state {
            ConsensusState::InitChain => {
                let header = begin_block_request
                    .header
                    .as_ref()
                    .ok_or("`BeginBlock` request does not contain a header")?;

                ConsensusState::ExecutingBlock {
                    block_height: header.height,
                    execution_state: BlockExecutionState::BeginBlock,
                }
            }
            ConsensusState::WaitingForBlock {
                ref block_height,
                ref app_hash,
            } => {
                let block_height = *block_height;

                let header = begin_block_request
                    .header
                    .as_ref()
                    .ok_or("`BeginBlock` request does not contain a header")?;

                if header.height != block_height {
                    return Err(format!(
                        "Expected height {} in `BeginBlock` request. Got {}",
                        block_height, header.height
                    ));
                }

                if &header.app_hash != app_hash {
                    return Err(format!(
                        "Expected app hash {:?} in `BeginBlock`. Got {:?}",
                        app_hash, header.app_hash
                    ));
                }

                ConsensusState::ExecutingBlock {
                    block_height,
                    execution_state: BlockExecutionState::BeginBlock,
                }
            }
            _ => {
                return Err(format!(
                    "`BeginBlock` cannot be called after {:?}",
                    self.state
                ))
            }
        };

        self.state = new_state;

        Ok(())
    }

    pub fn on_deliver_tx_request(&mut self) -> Result<(), String> {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ..
            } => execution_state.validate(BlockExecutionState::DeliverTx),
            _ => Err(format!(
                "`DeliverTx` cannot be called after {:?}",
                self.state
            )),
        }
    }

    pub fn on_end_block_request(
        &mut self,
        end_block_request: &RequestEndBlock,
    ) -> Result<(), String> {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ref block_height,
            } => {
                let block_height = *block_height;

                if block_height != end_block_request.height {
                    return Err(format!(
                        "Expected `EndBlock` for height {}. But received for {}",
                        block_height, end_block_request.height
                    ));
                }

                execution_state.validate(BlockExecutionState::EndBlock)
            }
            _ => Err(format!(
                "`EndBlock` cannot be called after {:?}",
                self.state
            )),
        }
    }

    #[inline]
    pub fn on_commit_request(&mut self) -> Result<(), String> {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ..
            } => execution_state.validate(BlockExecutionState::Commit),
            _ => Err(format!("`Commit` cannot be called after {:?}", self.state)),
        }
    }

    pub fn on_commit_response(&mut self, commit_response: &ResponseCommit) -> Result<(), String> {
        let new_state = match self.state {
            ConsensusState::ExecutingBlock {
                execution_state: BlockExecutionState::Commit,
                block_height,
            } => ConsensusState::WaitingForBlock {
                block_height: block_height + 1,
                app_hash: commit_response.data.clone(),
            },
            _ => return Err(format!("Received `CommitResponse` after {:?}", self.state)),
        };

        self.state = new_state;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusState {
    NoInfo,
    NotInitialized,
    InitChain,
    WaitingForBlock {
        block_height: i64,
        app_hash: Vec<u8>,
    },
    ExecutingBlock {
        block_height: i64,
        execution_state: BlockExecutionState,
    },
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::NoInfo
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockExecutionState {
    BeginBlock,
    DeliverTx,
    EndBlock,
    Commit,
}

impl BlockExecutionState {
    pub fn validate(&mut self, next: Self) -> Result<(), String> {
        let is_valid = matches!(
            (*self, next),
            (Self::BeginBlock, Self::DeliverTx)
                | (Self::BeginBlock, Self::EndBlock)
                | (Self::DeliverTx, Self::DeliverTx)
                | (Self::DeliverTx, Self::EndBlock)
                | (Self::EndBlock, Self::Commit)
        );

        if is_valid {
            *self = next;
            Ok(())
        } else {
            Err(format!("{:?} cannot be called after {:?}", next, self))
        }
    }
}
