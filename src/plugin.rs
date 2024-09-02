use bevy::app::App;
use bevy::prelude::Plugin;

use crate::pipelines::Magic2DPipelineParams;

pub struct Magic2DPluginConfig
{
    pub pipeline: Magic2DPipelineParams,
}

pub struct Magic2DPlugin
{
    pub config: Magic2DPluginConfig,
}

impl Plugin for Magic2DPlugin
{
    fn build(&self, _app: &mut App)
    {
        //
    }
}
