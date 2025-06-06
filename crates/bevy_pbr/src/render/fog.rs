use bevy_app::{App, Plugin};
use bevy_color::{ColorToComponents, LinearRgba};
use bevy_ecs::prelude::*;
use bevy_math::{Vec3, Vec4};
use bevy_render::{
    extract_component::ExtractComponentPlugin,
    load_shader_library,
    render_resource::{DynamicUniformBuffer, ShaderType},
    renderer::{RenderDevice, RenderQueue},
    view::ExtractedView,
    Render, RenderApp, RenderSystems,
};

use crate::{DistanceFog, FogFalloff};

/// The GPU-side representation of the fog configuration that's sent as a uniform to the shader
#[derive(Copy, Clone, ShaderType, Default, Debug)]
pub struct GpuFog {
    /// Fog color
    base_color: Vec4,
    /// The color used for the fog where the view direction aligns with directional lights
    directional_light_color: Vec4,
    /// Allocated differently depending on fog mode.
    /// See `mesh_view_types.wgsl` for a detailed explanation
    be: Vec3,
    /// The exponent applied to the directional light alignment calculation
    directional_light_exponent: f32,
    /// Allocated differently depending on fog mode.
    /// See `mesh_view_types.wgsl` for a detailed explanation
    bi: Vec3,
    /// Unsigned int representation of the active fog falloff mode
    mode: u32,
}

// Important: These must be kept in sync with `mesh_view_types.wgsl`
const GPU_FOG_MODE_OFF: u32 = 0;
const GPU_FOG_MODE_LINEAR: u32 = 1;
const GPU_FOG_MODE_EXPONENTIAL: u32 = 2;
const GPU_FOG_MODE_EXPONENTIAL_SQUARED: u32 = 3;
const GPU_FOG_MODE_ATMOSPHERIC: u32 = 4;

/// Metadata for fog
#[derive(Default, Resource)]
pub struct FogMeta {
    pub gpu_fogs: DynamicUniformBuffer<GpuFog>,
}

/// Prepares fog metadata and writes the fog-related uniform buffers to the GPU
pub fn prepare_fog(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut fog_meta: ResMut<FogMeta>,
    views: Query<(Entity, Option<&DistanceFog>), With<ExtractedView>>,
) {
    let views_iter = views.iter();
    let view_count = views_iter.len();
    let Some(mut writer) = fog_meta
        .gpu_fogs
        .get_writer(view_count, &render_device, &render_queue)
    else {
        return;
    };
    for (entity, fog) in views_iter {
        let gpu_fog = if let Some(fog) = fog {
            match &fog.falloff {
                FogFalloff::Linear { start, end } => GpuFog {
                    mode: GPU_FOG_MODE_LINEAR,
                    base_color: LinearRgba::from(fog.color).to_vec4(),
                    directional_light_color: LinearRgba::from(fog.directional_light_color)
                        .to_vec4(),
                    directional_light_exponent: fog.directional_light_exponent,
                    be: Vec3::new(*start, *end, 0.0),
                    ..Default::default()
                },
                FogFalloff::Exponential { density } => GpuFog {
                    mode: GPU_FOG_MODE_EXPONENTIAL,
                    base_color: LinearRgba::from(fog.color).to_vec4(),
                    directional_light_color: LinearRgba::from(fog.directional_light_color)
                        .to_vec4(),
                    directional_light_exponent: fog.directional_light_exponent,
                    be: Vec3::new(*density, 0.0, 0.0),
                    ..Default::default()
                },
                FogFalloff::ExponentialSquared { density } => GpuFog {
                    mode: GPU_FOG_MODE_EXPONENTIAL_SQUARED,
                    base_color: LinearRgba::from(fog.color).to_vec4(),
                    directional_light_color: LinearRgba::from(fog.directional_light_color)
                        .to_vec4(),
                    directional_light_exponent: fog.directional_light_exponent,
                    be: Vec3::new(*density, 0.0, 0.0),
                    ..Default::default()
                },
                FogFalloff::Atmospheric {
                    extinction,
                    inscattering,
                } => GpuFog {
                    mode: GPU_FOG_MODE_ATMOSPHERIC,
                    base_color: LinearRgba::from(fog.color).to_vec4(),
                    directional_light_color: LinearRgba::from(fog.directional_light_color)
                        .to_vec4(),
                    directional_light_exponent: fog.directional_light_exponent,
                    be: *extinction,
                    bi: *inscattering,
                },
            }
        } else {
            // If no fog is added to a camera, by default it's off
            GpuFog {
                mode: GPU_FOG_MODE_OFF,
                ..Default::default()
            }
        };

        // This is later read by `SetMeshViewBindGroup<I>`
        commands.entity(entity).insert(ViewFogUniformOffset {
            offset: writer.write(&gpu_fog),
        });
    }
}

/// Inserted on each `Entity` with an `ExtractedView` to keep track of its offset
/// in the `gpu_fogs` `DynamicUniformBuffer` within `FogMeta`
#[derive(Component)]
pub struct ViewFogUniformOffset {
    pub offset: u32,
}

/// A plugin that consolidates fog extraction, preparation and related resources/assets
pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "fog.wgsl");

        app.register_type::<DistanceFog>();
        app.add_plugins(ExtractComponentPlugin::<DistanceFog>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<FogMeta>()
                .add_systems(Render, prepare_fog.in_set(RenderSystems::PrepareResources));
        }
    }
}
