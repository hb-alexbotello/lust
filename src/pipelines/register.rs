use crate::config::ImageKind;
use crate::pipelines::PipelineResult;
use bytes::Bytes;
use enum_dispatch::enum_dispatch;

use super::aot::AheadOfTimePipeline;
use super::jit::JustInTimePipeline;
use super::realtime::RealtimePipeline;

/// Pipelines are dynamically selected here.
///
/// This is not a Box<dyn Trait> due to this being rather
/// performance critical and this approach allows for more
/// room for the compiler to optimise.
#[allow(clippy::enum_variant_names)]
#[enum_dispatch(Pipeline)]
pub enum PipelineSelector {
    RealtimePipeline,
    AheadOfTimePipeline,
    JustInTimePipeline,
}

#[enum_dispatch]
pub trait Pipeline: Sync + Send + 'static {
    fn on_upload(&self, kind: ImageKind, data: Vec<u8>) -> anyhow::Result<PipelineResult>;

    fn on_fetch(
        &self,
        desired_kind: ImageKind,
        data_kind: ImageKind,
        data: Bytes,
        sizing_id: u32,
        custom_size: Option<(u32, u32)>,
    ) -> anyhow::Result<PipelineResult>;
}
