use std::sync::mpsc::{self, Receiver};

use tokio::runtime::{Builder, Runtime};
use tokio_util::sync::CancellationToken;

use crate::resolver::{AnalysisReport, Resolver, ResolverConfig};

#[derive(Debug)]
pub enum JobEvent {
    Started,
    Completed(AnalysisReport),
    Failed(String),
    Cancelled,
}

pub struct JobHandle {
    pub receiver: Receiver<JobEvent>,
    pub cancel: CancellationToken,
}

pub struct JobManager {
    runtime: Runtime,
}

impl JobManager {
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = Builder::new_multi_thread().enable_all().build()?;
        Ok(Self { runtime })
    }

    pub fn start_analysis(&self, url: String, config: ResolverConfig) -> JobHandle {
        let (sender, receiver) = mpsc::channel();
        let cancel = CancellationToken::new();
        let child = cancel.clone();
        self.runtime.spawn(async move {
            let _ = sender.send(JobEvent::Started);
            let resolver = Resolver::new(reqwest::Client::new(), config);
            match resolver.analyze(&url, child.clone()).await {
                Ok(report) => {
                    let _ = sender.send(JobEvent::Completed(report));
                }
                Err(crate::error::SubLensError::Cancelled) => {
                    let _ = sender.send(JobEvent::Cancelled);
                }
                Err(error) => {
                    let _ = sender.send(JobEvent::Failed(error.to_string()));
                }
            }
        });
        JobHandle { receiver, cancel }
    }
}
