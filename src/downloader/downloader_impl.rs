use crate::downloader::{
    chain_config::{ChainConfig, ChainsConfig},
    headers::{header_merge_stream::HeaderMergeStream, header_slices::HeaderSlices},
    opts::Opts,
    sentry_client,
    sentry_client::SentryClient,
    sentry_client_impl::SentryClientImpl,
    sentry_client_reactor::SentryClientReactor,
};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio_stream::StreamExt;

pub struct Downloader {
    opts: Opts,
    chain_config: ChainConfig,
}

impl Downloader {
    pub fn new(opts: Opts, chains_config: ChainsConfig) -> Downloader {
        let chain_config = chains_config.0[&opts.chain_name].clone();

        Downloader { opts, chain_config }
    }

    pub async fn run(
        &self,
        sentry_client_opt: Option<Box<dyn SentryClient>>,
    ) -> anyhow::Result<()> {
        let status = sentry_client::Status {
            total_difficulty: ethereum_types::U256::zero(),
            best_hash: ethereum_types::H256::zero(),
            chain_fork_config: self.chain_config.clone(),
            max_block: 0,
        };

        let mut sentry_client = match sentry_client_opt {
            Some(v) => v,
            None => Box::new(SentryClientImpl::new(self.opts.sentry_api_addr.clone()).await?),
        };

        sentry_client.set_status(status).await?;

        let mut sentry_reactor = SentryClientReactor::new(sentry_client);
        sentry_reactor.start();

        let mut ui_system = crate::downloader::ui_system::UISystem::new();
        ui_system.start();

        let header_slices = Arc::new(HeaderSlices::new(100 << 20 /* 100 Mb */));
        let sentry = Arc::new(RwLock::new(sentry_reactor));

        let header_slices_view =
            crate::downloader::headers::HeaderSlicesView::new(Arc::clone(&header_slices));
        ui_system.set_view(Some(Box::new(header_slices_view)));

        let headers_stream =
            HeaderMergeStream::new(Arc::clone(&header_slices), Arc::clone(&sentry));
        let mut stream = headers_stream.stream()?;

        while let Some(header) = stream.next().await {
            tracing::info!("next header: {:?}", header.number);
        }

        ui_system.stop().await?;

        {
            let mut sentry_reactor = sentry.write();
            sentry_reactor.stop().await?;
        }

        Ok(())
    }
}
