pub mod pipeline_basic;

pub struct Magic2DPipelineSdfGiParams {}
pub struct Magic2DPipelineBasicPlusParams {}
pub struct Magic2DPipelineBasicParams {}
pub struct Magic2DPipelineRadianceCascadeGiParams {}

pub enum Magic2DPipelineParams
{
    Basic(Magic2DPipelineBasicParams), // Only direct light and no shadows.
    BasicPlus(Magic2DPipelineBasicPlusParams), // Direct light + SDF-based shadows.
    SdfGi(Magic2DPipelineSdfGiParams), // Direct light + SDF-based GI.
    RadianceCascadeGi(Magic2DPipelineRadianceCascadeGiParams), // Direct light + RC-based GI.
}
