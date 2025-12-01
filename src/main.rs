use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use hexx::{Hex, HexLayout, HexOrientation, Vec2 as HexVec2};
use std::collections::{HashSet, VecDeque};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Colony MMO".to_string(),
                canvas: Some("#bevy-canvas".into()),
                fit_canvas_to_parent: true, // This ensures the canvas fills the parent element
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //.add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<SelectionState>()
        .init_gizmo_group::<DashedGizmos>()
        .add_systems(Startup, (setup_camera, setup_physics, configure_gizmos))
        .add_systems(Startup, (setup_hex_grid, spawn_units).chain())
        .add_systems(Update, (camera_movement, move_ants, ant_input, draw_selection_visuals, draw_selection_box, draw_hex_grid))
        .run();
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct DashedGizmos;

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
     let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
     config.line_width = 2.0;

     let (dashed_config, _) = config_store.config_mut::<DashedGizmos>();
     dashed_config.line_width = 2.0;
     // Dotted is the only non-solid variant available in this version of Bevy 0.14.2 for some reason?
     // The docs say Dashed exists, but the source I read for 0.14.2 only showed Solid and Dotted.
     // Let's try Dotted for now to fix the build.
     dashed_config.line_style = GizmoLineStyle::Dotted;
}


#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct MapLayout(HexLayout);

#[derive(Component)]
struct Ant;

#[derive(Component)]
struct Queen;

#[derive(Component)]
struct TargetPosition(Vec2);

#[derive(Component, Default)]
struct Path {
    waypoints: VecDeque<Vec2>,
}

#[derive(Component)]
struct Selected;

#[derive(Resource, Default)]
struct SelectionState {
    start_pos: Option<Vec2>,
    drag_current: Option<Vec2>,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));
}

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec2::ZERO;
}

fn camera_movement(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, mut projection) = camera_query.single_mut();
    let speed = 500.0;
    let zoom_speed = 1.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation.y += speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation.y -= speed * time.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        projection.scale += zoom_speed * time.delta_seconds();
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        projection.scale -= zoom_speed * time.delta_seconds();
        projection.scale = projection.scale.max(0.1);
    }
}

fn setup_hex_grid(mut commands: Commands) {
    let layout = HexLayout {
        scale: HexVec2::splat(20.0),
        orientation: HexOrientation::Pointy,
        ..default()
    };

    commands.insert_resource(MapLayout(layout));
}

fn draw_hex_grid(mut gizmos: Gizmos, layout: Res<MapLayout>) {
    let hex_coords = Hex::ZERO.spiral_range(0..10);
    for hex in hex_coords {
        let corners = layout.0.hex_corners(hex);
        for i in 0..6 {
            let start = corners[i];
            let end = corners[(i + 1) % 6];
            // Convert hexx::Vec2 to bevy::Vec2 to resolve crate version mismatch
            let start_bevy = Vec2::new(start.x, start.y);
            let end_bevy = Vec2::new(end.x, end.y);
            gizmos.line_2d(start_bevy, end_bevy, Color::from(Srgba::hex("444444").unwrap()));
        }
    }
}

fn spawn_units(mut commands: Commands, layout: Res<MapLayout>) {
    // Spawn Queen (Gold, bigger, immobile) at 0,0 (Hex ZERO)
    let queen_color = Color::from(Srgba::hex("8B4513").unwrap()); // SaddleBrown for Queen
    let queen_hex = Hex::ZERO;
    let queen_pos = layout.0.hex_to_world_pos(queen_hex);
    let queen_vec = Vec2::new(queen_pos.x, queen_pos.y);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: queen_color,
                custom_size: Some(Vec2::new(20.0, 20.0)), // Smaller Queen (was 25.0)
                ..default()
            },
            transform: Transform::from_xyz(queen_vec.x, queen_vec.y, 1.0),
            ..default()
        },
        RigidBody::Fixed, // Immobile
        Collider::ball(12.5),
        Ant,
        Queen,
        TargetPosition(queen_vec),
        Path::default(),
    ));

    // Spawn Worker Ants
    let worker_color = Color::from(Srgba::hex("8B4513").unwrap()); // SaddleBrown
    // Spawn 3 workers in the first ring
    let worker_hexes = Hex::ZERO.ring(1).take(3);
    
    for hex in worker_hexes {
        let pos = layout.0.hex_to_world_pos(hex);
        let vec = Vec2::new(pos.x, pos.y);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: worker_color,
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                transform: Transform::from_xyz(vec.x, vec.y, 1.0),
                ..default()
            },
            RigidBody::Dynamic,
            // Make units Sensors to avoid physical collision/locking
            Sensor,
            Collider::ball(5.0),
            Velocity::zero(),
            Damping { linear_damping: 20.0, angular_damping: 1.0 },
            Ant,
            TargetPosition(vec),
            Path::default(),
        ));
    }
}

fn ant_input(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut ant_q: Query<(Entity, &mut TargetPosition, &Transform, &mut Path), With<Ant>>,
    mut selection_state: ResMut<SelectionState>,
    selected_q: Query<Entity, With<Selected>>,
    layout: Res<MapLayout>,
) {
    let window = windows.single();
    let cursor_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else if let Some(touch) = touches.first_pressed_position() {
        touch
    } else {
        return; // No input
    };

    let (camera, camera_transform) = camera_q.single();
    let world_pos = if let Some(pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        pos
    } else {
        return;
    };

    // Handle Input
    if mouse_input.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
        selection_state.start_pos = Some(world_pos);
        selection_state.drag_current = Some(world_pos);
    }

    if mouse_input.pressed(MouseButton::Left) || touches.iter().count() > 0 {
        selection_state.drag_current = Some(world_pos);
    }

    if mouse_input.just_released(MouseButton::Left) || touches.any_just_released() {
        if let Some(start) = selection_state.start_pos {
            let dist = start.distance(world_pos);
            
            if dist < 5.0 {
                // CLICK / TAP
                handle_click(
                    world_pos, 
                    &mut commands, 
                    &mut ant_q, 
                    &selected_q,
                    &layout.0
                );
            } else {
                // DRAG / BOX SELECT
                handle_box_select(
                    start, 
                    world_pos, 
                    &mut commands, 
                    &ant_q,
                    &selected_q,
                    &layout.0
                );
            }
        }
        selection_state.start_pos = None;
        selection_state.drag_current = None;
    }
}

fn handle_click(
    world_pos: Vec2,
    commands: &mut Commands,
    ant_q: &mut Query<(Entity, &mut TargetPosition, &Transform, &mut Path), With<Ant>>,
    selected_q: &Query<Entity, With<Selected>>,
    layout: &HexLayout,
) {
    // 1. Check for unit in the clicked hex
    let mut hit_unit = None;

    // Convert world_pos to hex to check which cell we clicked
    let hex_vec = HexVec2::new(world_pos.x, world_pos.y);
    let clicked_hex = layout.world_pos_to_hex(hex_vec);

    // Find if any ant is in this hex (based on their transform/target)
    // We check transform position to see if they are "visually" in the hex
    for (entity, _, transform, _) in ant_q.iter() {
        let pos = transform.translation.truncate();
        let ant_hex_vec = HexVec2::new(pos.x, pos.y);
        let ant_hex = layout.world_pos_to_hex(ant_hex_vec);
        
        if ant_hex == clicked_hex {
            hit_unit = Some(entity);
            break; 
        }
    }

    if let Some(entity) = hit_unit {
        // TOGGLE SELECTION:
        // If the entity is already selected, deselect it.
        // If it's not selected, select it (and clear others if we want single select, but user asked for toggle behavior).
        // Based on user request "tap a unit to deselect it", we imply a toggle or multi-select mode?
        // "if i select a unit ,i cai an still dratgg a new window to grup selet other units and its additiive, i will select all three, then if i tap one of the selected, it deselcted"
        
        if selected_q.contains(entity) {
            commands.entity(entity).remove::<Selected>();
        } else {
            // If we are just clicking one unit, do we clear others? The prompt implies "additive" behavior for the drag window,
            // but usually a single click replaces selection unless shift is held. 
            // However, the user says "tap a unit to deselect it".
            // Let's assume:
            // 1. Click on unselected -> Select ONLY that one (Standard RTS)
            // 2. Click on selected -> Deselect that one (User Request)
            // BUT user also mentioned "additive" drag.
            // Let's try this:
            // Single click adds/toggles if it's a toggle, or replaces if it's a new selection?
            // Re-reading: "tap a unit to deselect it... if i select a unit... then if i tap one of the selected, it deselcted"
            // This implies clicking a selected unit deselects it.
            // What if I click an unselected unit? Usually that clears and selects new. 
            // But if the user wants to "drag a new window... and its additive", that's about the window.
            
            // Let's implement: 
            // - Click selected: Deselect it.
            // - Click unselected: Select it (and clear others? Standard behavior says yes, unless we are in a special mode).
            // Let's stick to standard RTS + the requested "deselect on click":
            // If I click an unselected unit, I probably want to select it. If I didn't hold shift, I probably want to select ONLY it.
            
            // However, to support the "workflow" described: 
            // 1. Select one.
            // 2. Drag select more (additive).
            // 3. Tap one to deselect.
            
            // If I click an unselected unit without shift, standard is "Clear all, select this".
            // If I click a selected unit without shift, standard is "Select only this" (if multiple selected) or "Nothing" (if only one).
            // The user specifically wants "tap to deselect".
            
            // Interpretation: Single click always toggles? That's mobile-friendly.
            // Let's try: Single click toggles selection state of the target. Does NOT clear others.
            // This fits "additive" workflow best without modifier keys.
            commands.entity(entity).insert(Selected);
        }
    } else {
        // Move Selected Units to Center of Hexes, avoiding overlap
        
        // Identify Occupied Hexes (Targets of non-selected units)
        let mut occupied: HashSet<Hex> = HashSet::new();
        for (entity, target, _, _) in ant_q.iter() {
             // Don't mark current targets of selected units as occupied, 
             // because they are about to move (or stay if we click same spot)
             if selected_q.contains(entity) { continue; }

             // Convert target Vec2 to Hex
             let t_vec = HexVec2::new(target.0.x, target.0.y);
             let hex = layout.world_pos_to_hex(t_vec);
             occupied.insert(hex);
        }

        // Determine Target Hex for click
        let target_pos_vec = HexVec2::new(world_pos.x, world_pos.y);
        let target_hex = layout.world_pos_to_hex(target_pos_vec);
        
        let selected_entities: Vec<Entity> = selected_q.iter().collect();
        if selected_entities.is_empty() { return; }

        let mut available_hexes = Vec::new();
        let candidates = target_hex.spiral_range(0..10); 
        
        for hex in candidates {
            // Allow moving to same hex multiple times if needed, OR just ignore occupancy for now?
            // User said: "you should be able to walk through a cell taht a unit is in!!"
            // This implies we shouldn't block movement based on occupancy, OR we should just treat it as soft collision.
            // The previous logic was: "occupied.contains(&hex)".
            // Let's RELAX this. If we relax it, units might stack.
            // "units gets locked together when they corss paths"
            // This is likely due to physics collisions (Rapier).
            // We should probably use sensor colliders or collision groups to avoid units pushing each other?
            // But for now, let's remove the strict "occupied" check for target assignment so they can at least try to go there.
            // Actually, spiral_range assignment is for formation.
            
            // Let's keep formation logic but MAYBE allow overlap if space is tight?
            // Or maybe the user means transient pathing?
            // "you should be able to walk through a cell taht a unit is in" -> This suggests pathfinding issue or physics issue.
            // If it's physics, they bump.
            // If it's this logic, they can't target the same cell.
            
            // Let's Keep formation but allow moving through.
            // The issue "locked together when they cross paths" is definitely physics.
            
            if !occupied.contains(&hex) {
                available_hexes.push(hex);
                occupied.insert(hex); 
                if available_hexes.len() >= selected_entities.len() {
                    break;
                }
            }
        }

        // Assign Targets
        let mut moved_any = false;
        for (i, entity) in selected_entities.iter().enumerate() {
            if let Some(dest_hex) = available_hexes.get(i) {
                if let Ok((_, mut target, transform, mut path)) = ant_q.get_mut(*entity) {
                     let current_pos_vec = transform.translation.truncate();
                     let current_hex = layout.world_pos_to_hex(HexVec2::new(current_pos_vec.x, current_pos_vec.y));
                     
                     // Generate path using line_to (grid walking)
                     let route: Vec<Vec2> = current_hex.line_to(*dest_hex)
                        .skip(1) // Skip start
                        .map(|h| {
                            let p = layout.hex_to_world_pos(h);
                            Vec2::new(p.x, p.y)
                        })
                        .collect();
                     
                     path.waypoints = VecDeque::from(route);
                     
                     // Set initial target
                     if let Some(first) = path.waypoints.pop_front() {
                         target.0 = first;
                     } else {
                         // Already there or path empty
                         let pos = layout.hex_to_world_pos(*dest_hex);
                         target.0 = Vec2::new(pos.x, pos.y);
                     }
                     
                     moved_any = true;
                }
            }
        }
        
        if !moved_any {
             for sel in selected_q.iter() {
                commands.entity(sel).remove::<Selected>();
            }
        }
    }
}

fn handle_box_select(
    start: Vec2,
    end: Vec2,
    commands: &mut Commands,
    ant_q: &Query<(Entity, &mut TargetPosition, &Transform, &mut Path), With<Ant>>,
    selected_q: &Query<Entity, With<Selected>>,
    layout: &HexLayout,
) {
    let min = start.min(end);
    let max = start.max(end);

    // Toggle selection for units inside the box
    for (entity, _, transform, _) in ant_q.iter() {
        let pos = transform.translation.truncate();
        // Convert unit position to hex center to check if that hex is touched by the box?
        // OR: Check if the hex center is inside the box.
        // Ideally, if the box touches the unit's cell, it should select.
        // But checking if a rectangle intersects a hexagon is complex.
        // A simpler approximation: Check if the unit's hex center is inside the box.
        // This is what we did before with `pos`.
        // If "hit box should be the whole cell", it means if I drag over ANY part of the cell, it selects.
        // That means intersection of Box vs Hexagon.
        
        // Approximate: Check if Box intersects Circle (Radius ~ Hex Size)
        // Hex radius is layout.scale.x (20.0)
        // Box is min/max.
        
        let hex_radius = layout.scale.x;
        // Clamp point in box to find closest point to circle center
        let closest_x = pos.x.clamp(min.x, max.x);
        let closest_y = pos.y.clamp(min.y, max.y);
        
        let closest = Vec2::new(closest_x, closest_y);
        let dist_sq = pos.distance_squared(closest);
        
        if dist_sq < (hex_radius * hex_radius) {
            if selected_q.contains(entity) {
                commands.entity(entity).remove::<Selected>();
            } else {
                commands.entity(entity).insert(Selected);
            }
        } 
    }
}

fn draw_selection_visuals(
    mut gizmos: Gizmos,
    mut dashed_gizmos: Gizmos<DashedGizmos>,
    query: Query<(&Transform, &TargetPosition, &Path), With<Selected>>,
    layout: Res<MapLayout>,
) {
    let selection_color = Color::from(Srgba::hex("FFFF00").unwrap()); // Yellow for selection
    let path_color = Color::from(Srgba::hex("FFFF00").unwrap()); // Yellow for path
    let target_color = Color::from(Srgba::hex("FFFF00").unwrap()); // Yellow for destination

    for (transform, target, path) in query.iter() {
        let current_pos = transform.translation.truncate();
        
        // 1. Draw Hexagonal Outline around the Unit
        // Use the grid layout scale so it matches the cell size
        let corners = layout.0.hex_corners(Hex::ZERO);
         for i in 0..6 {
            let start = corners[i];
            let end = corners[(i + 1) % 6];
            let start_v = Vec2::new(start.x, start.y) + current_pos;
            let end_v = Vec2::new(end.x, end.y) + current_pos;
            gizmos.line_2d(start_v, end_v, selection_color);
        }
        
        // 2. Draw Path
        // Line from current to target (immediate)
        dashed_gizmos.line_2d(current_pos, target.0, path_color);
        
        let mut prev_point = target.0;
        for &waypoint in &path.waypoints {
             dashed_gizmos.line_2d(prev_point, waypoint, path_color);
             prev_point = waypoint;
        }
        
        // 3. Draw Target Hexagon (at final destination)
        // Only draw if we are not already there (distance > some small amount)
        // or if there are waypoints left.
        if !path.waypoints.is_empty() || current_pos.distance(target.0) > 2.0 {
            let target_hex_vec = HexVec2::new(prev_point.x, prev_point.y);
            let target_hex = layout.0.world_pos_to_hex(target_hex_vec);
            
            let corners = layout.0.hex_corners(target_hex);
            for i in 0..6 {
                let start = corners[i];
                let end = corners[(i + 1) % 6];
                gizmos.line_2d(Vec2::new(start.x, start.y), Vec2::new(end.x, end.y), target_color);
            }
        }
    }
}

// Hack to fix color restore for Queen
fn move_ants(
    mut ant_q: Query<(&mut Velocity, &mut Transform, &mut TargetPosition, &mut Path), (With<Ant>, Without<Queen>)>,
) {
    let speed = 100.0;
    let arrival_radius = 2.0;
    
    for (mut velocity, mut transform, mut target, mut path) in ant_q.iter_mut() {
        let delta = target.0 - transform.translation.truncate();
        let distance = delta.length();

        if distance > arrival_radius {
            let direction = delta.normalize();
            velocity.linvel = direction * speed;
            
             // Rotate to face direction
            if delta.length_squared() > 0.0 {
                 let angle = delta.y.atan2(delta.x);
                 transform.rotation = Quat::from_rotation_z(angle);
            }
        } else {
            // Snap to exact position to ensure centered in cell
            transform.translation.x = target.0.x;
            transform.translation.y = target.0.y;
            
            // Check for next waypoint
            if let Some(next_pos) = path.waypoints.pop_front() {
                target.0 = next_pos;
                // Continue moving immediately
                let delta = target.0 - transform.translation.truncate();
                // Rotate to face direction
                if delta.length_squared() > 0.0 {
                     let angle = delta.y.atan2(delta.x);
                     transform.rotation = Quat::from_rotation_z(angle);
                }

                let direction = delta.normalize_or_zero();
                velocity.linvel = direction * speed;
            } else {
                velocity.linvel = Vec2::ZERO;
            }
        }
    }
}

// Debug gizmo for selection box
fn draw_selection_box(
    mut gizmos: Gizmos,
    state: Res<SelectionState>,
) {
    if let (Some(start), Some(current)) = (state.start_pos, state.drag_current) {
        let center = (start + current) / 2.0;
        let size = (start - current).abs();
        
        // Only draw if it looks like a drag (> 5.0 distance)
        if start.distance(current) > 5.0 {
            gizmos.rect_2d(center, 0.0, size, Color::WHITE);
        }
    }
}
