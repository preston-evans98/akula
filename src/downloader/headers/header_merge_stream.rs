use crate::downloader::{
    block_id,
    headers::header_slices::HeaderSlices,
    messages::{
        BlockHeadersMessage, EthMessageId, GetBlockHeadersMessage, GetBlockHeadersMessageParams,
        Message,
    },
    sentry_client::PeerFilter,
    sentry_client_reactor::SentryClientReactor,
};
use async_stream::stream;
use ethereum::Header as HeaderType;
use futures_core::Stream;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio_stream::StreamExt;

pub struct HeaderMergeStream {
    header_slices: Arc<HeaderSlices>,
    sentry: Arc<RwLock<SentryClientReactor>>,
}

impl HeaderMergeStream {
    pub fn new(header_slices: Arc<HeaderSlices>, sentry: Arc<RwLock<SentryClientReactor>>) -> Self {
        Self {
            header_slices,
            sentry,
        }
    }

    pub fn stream(&self) -> anyhow::Result<Box<dyn Stream<Item = HeaderType> + Unpin + Send>> {
        let mut message_stream = self.receive_headers()?;

        for i in 0..100 {
            let _sent_peers_count = self.request(i, 1000, 1)?;
            // TODO
            // if sent_peers_count > 0 {
            //     break;
            // }
            //tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let stream = stream! {

            while let Some(message) = message_stream.next().await {
                for header in message.headers {
                    yield header;
                }
            }

            // TODO: remove
            yield HeaderType{
                parent_hash: Default::default(),
                ommers_hash: Default::default(),
                beneficiary: Default::default(),
                state_root: Default::default(),
                transactions_root: Default::default(),
                receipts_root: Default::default(),
                logs_bloom: Default::default(),
                difficulty: Default::default(),
                number: Default::default(),
                gas_limit: Default::default(),
                gas_used: Default::default(),
                timestamp: 0,
                extra_data: vec![],
                mix_hash: Default::default(),
                nonce: Default::default()
            };
        };

        Ok(Box::new(Box::pin(stream)))
    }

    fn receive_headers(
        &self,
    ) -> anyhow::Result<Box<dyn Stream<Item = BlockHeadersMessage> + Unpin + Send>> {
        let in_stream = self
            .sentry
            .read()
            .receive_messages(EthMessageId::BlockHeaders)?;

        let out_stream = in_stream.map(|message| match message {
            Message::BlockHeaders(message) => message,
            _ => panic!("unexpected type {:?}", message.eth_id()),
        });

        Ok(Box::new(out_stream))
    }

    fn request(&self, request_id: u64, block_num: u64, limit: u64) -> anyhow::Result<()> {
        let message = GetBlockHeadersMessage {
            request_id,
            params: GetBlockHeadersMessageParams {
                start_block: block_id::BlockId::Number(block_num),
                limit,
                skip: 0,
                reverse: 0,
            },
        };
        self.sentry
            .read()
            .send_message(Message::GetBlockHeaders(message), PeerFilter::Random(1))
    }
}
