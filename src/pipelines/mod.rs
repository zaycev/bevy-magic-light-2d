pub mod pipeline_basic;

pub struct Magic2DPipelineBasicParams {}

pub enum Magic2DPipelineParams
{
    Basic(Magic2DPipelineBasicParams), // Most basic model with only direct light and no occlusion.
}
