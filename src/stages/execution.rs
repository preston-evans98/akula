use crate::{
    accessors,
    stagedsync::stage::{ExecOutput, Stage, StageInput},
    MutableTransaction, PlainStateReader, PlainStateWriter, StateReader, StateWriter,
};
use anyhow::bail;
use async_trait::async_trait;
use ethereum_types::H160;
use evmodin::Output;

#[derive(Debug)]
pub struct Execution;

#[async_trait]
impl<'db, RwTx: MutableTransaction<'db>> Stage<'db, RwTx> for Execution {
    fn id(&self) -> crate::StageId {
        todo!()
    }

    fn description(&self) -> &'static str {
        todo!()
    }

    async fn execute<'tx>(&self, tx: &'tx mut RwTx, input: StageInput) -> anyhow::Result<ExecOutput>
    where
        'db: 'tx,
    {
        let mut stage_progress = input.stage_progress.unwrap_or(0);

        for block_number in stage_progress
            ..input
                .previous_stage
                .map(|(_, b)| b)
                .unwrap_or(stage_progress)
        {
            let receipts = execute_block(tx, block_number).await?;
        }

        Ok(ExecOutput::Progress {
            stage_progress,
            done: true,
            must_commit: true,
        })
    }

    async fn unwind<'tx>(
        &self,
        tx: &'tx mut RwTx,
        input: crate::stagedsync::stage::UnwindInput,
    ) -> anyhow::Result<()>
    where
        'db: 'tx,
    {
        todo!()
    }
}

async fn execute_block<'db: 'tx, 'tx, RwTx: MutableTransaction<'db>>(
    tx: &'tx RwTx,
    block_number: u64,
) -> anyhow::Result<Vec<()>> {
    let block_hash = accessors::chain::canonical_hash::read(tx, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no canonical block hash"))?;
    let block_header = accessors::chain::header::read(tx, block_hash, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no block header"))?;

    let block_body_info = accessors::chain::storage_body::read(tx, block_hash, block_number)
        .await?
        .ok_or_else(|| anyhow::Error::msg("no block body"))?;

    let block_body =
        accessors::chain::tx::read(tx, block_body_info.base_tx_id, block_body_info.tx_amount)
            .await?;

    if block_body.len() != block_body_info.tx_amount as usize {
        bail!("block body len mismatch");
    }

    let senders = accessors::chain::tx_sender::read(
        tx,
        block_body_info.base_tx_id,
        block_body_info.tx_amount,
    )
    .await?;

    if senders.len() != block_body_info.tx_amount as usize {
        bail!("senders len mismatch");
    }

    let w = PlainStateWriter::new(tx, block_number);
    let r = PlainStateReader::new(tx);

    let mut gas_pool = block_header.gas_limit;
    for (ethtx, sender) in block_body.into_iter().zip(senders) {
        let execution_result = execute_transaction(&w, &r, ethtx, sender).await?;
    }

    let mut receipts = vec![];

    Ok(receipts)
}

async fn execute_transaction<'storage>(
    w: &impl StateWriter,
    r: &impl StateReader<'storage>,
    tx: ethereum::Transaction,
    sender: H160,
) -> anyhow::Result<Output> {
    todo!()
}
