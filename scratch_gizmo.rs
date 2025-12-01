use bevy::prelude::*;
use bevy::gizmos::{GizmoConfigStore, GizmoLineStyle};

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DashedLineGizmos;

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
     let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
     config.line_width = 2.0;
     
     let (dashed_config, _) = config_store.config_mut::<DashedLineGizmos>();
     dashed_config.line_width = 2.0;
     dashed_config.line_style = GizmoLineStyle::Dashed {
         line_scale: 10.0,
         gap_scale: 5.0,
     };
}
