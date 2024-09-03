pub mod pipeline_basic;

#[derive(Debug, Clone)]
pub struct Magic2DPipelineBasicParams {}

#[derive(Debug, Clone)]
pub enum Magic2DPipelineParams
{
    Basic(Magic2DPipelineBasicParams), // Most basic model with only direct light and no occlusion.
}
