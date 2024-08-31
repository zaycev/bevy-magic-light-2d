mod pipeline_low;

pub struct Magic2DPipelineHighParams {}
pub struct Magic2DPipelineMediumParams {}
pub struct Magic2DPipelineLowParams {}

pub enum Magic2DPipelineParams
{
    High(Magic2DPipelineHighParams), // Direct + indirect light with SDF-based occlusion.
    Medium(Magic2DPipelineMediumParams), // Direct light + SDF-based occlusion.
    Low(Magic2DPipelineLowParams),   // Only direct light and no occlusion.
}
