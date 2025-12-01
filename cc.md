Character controller
Most games involve bodies behaving in ways that defy the laws of physics: floating platforms, elevators, playable characters, etc. This is why kinematic bodies exist: they offer a total control over the body’s trajectory since they are completely immune to forces or impulses (like gravity, contacts, joints).

But this control comes at a price: it is up to the user to take any obstacle into account by running custom collision-detection operations manually and update the trajectory accordingly. This can be very difficult. Detecting obstacles usually rely on ray-casting or shape-casting, used to adjust the trajectory based on the potential contact normals. Often, multiple ray or shape-casts are needed, and the trajectory adjustment code isn’t straightforward.

The Kinematic Character Controller (which we will abbreviate to character controller) is a higher-level tool that will emit the proper ray-casts and shape-casts to adjust the user-defined trajectory based on obstacles. The well-known move-and-slide operation is the main feature of a character controller.

note
Despite its name, a character controller can also be used for moving objects that are not characters. For example, a character controller may be used to move a platform. In the rest of this guide, we will use the word character to designate whatever you would like to move using the character controller.

Rapier provides a built-in general-purpose character controller implementation. It allows you to easily:

Stop at obstacles.
Slide on slopes that are not to steep.
Climb stairs automatically.
Walk over small obstacles.
Interact with moving platforms.
Despite the fact that this built-in character controller is designed to be generic enough to serve as a good starting point for many common use-cases, character-control (especially for the player’s character itself) is often very game-specific. Therefore the builtin character controller may not work perfectly out-of-the-box for all game types. Don’t hesitate to copy and customize it to fit your particular needs.

Setup and usage
The character controller implementation is exposed as the KinematicCharacterController structure. This structure only contains information about the character controller’s behavior. It does not contain any collider-specific or rigid-body-specific information like handles, velocities, positions, etc. Therefore, the same instance of KinematicCharacterController can be used to control multiple rigid-bodies/colliders if they rely on the same set of parameters. The KinematicCharacterController exposes only two methods:

move_shape is responsible for calculating the possible movement of a character based on the desired movement, obstacles, and character controller options.
solve_character_collision_impulses is detailed in the collisions section.
Example 2D
Example 3D
The recommended way to update the character’s position depends on its representation:

A collider not attached to any rigid-body: set the collider’s position directly to the corrected movement added to its current position.
A velocity-based kinematic rigid-body: set its velocity to the computed movement divided by the timestep length.
A position-based kinematic rigid-body: set its next kinematic position to the corrected movement added to its current position.
info
The character’s shape may be any shape supported by Rapier. However, it is recommended to either use a cuboid, a ball, or a capsule since they involve less computations and less numerical approximations.

warning
The built-in character controller does not support rotational movement. It only supports translations.

Character offset
For performance and numerical stability reasons, the character controller will attempt to preserve a small gap between the character shape and the environment. This small gap is named offset and acts as a small margin around the character shape. A good value for this offset is something sufficiently small to make the gap unnoticeable, but sufficiently large to avoid numerical issues (if the character seems to get stuck inexplicably, try increasing the offset).

character offset

// The character offset is set to 0.01.
character_controller.offset = CharacterLength::Absolute(0.01);
// The character offset is set to 0.01 multiplied by the shape’s height.
character_controller.offset = CharacterLength::Relative(0.01);

warning
It is not recommended to change the offset after the creation of the character controller.

Up vector
The up vector instructs the character controller of what direction should be considered vertical. The horizontal plane is the plane orthogonal to this up vector. There are two equivalent ways to evaluate the slope of the floor: by taking the angle between the floor and the horizontal plane (in 2D), or by taking the angle between the up-vector and the normal of the floor (in 2D and 3D). By default, the up vector is the positive y axis, but it can be modified to be any (unit) vector that suits the application.

up vector and slope angles

// Set the up-vector to the positive X axis.
character_controller.up = Vector::x_axis();

Slopes
If sliding is enabled, the character can automatically climb slopes if they are not too steep, or slide down slopes if they are too steep. Sliding is configured by the following parameters:

The max slope climb angle: if the angle between the slope to climb and the horizontal floor is larger than this value, then the character won’t be able to slide up this slope.
The min slope slide angle: if the angle between the slope and the horizontal floor is smaller than this value, then the vertical component of the character’s movement won’t result in any sliding.
info
As always in Rapier, angles are specified in radians.

// Don’t allow climbing slopes larger than 45 degrees.
character_controller.max_slope_climb_angle = 45_f32.to_radians();
// Automatically slide down on slopes smaller than 30 degrees.
character_controller.min_slope_slide_angle = 30_f32.to_radians();

Stairs and small obstacles
If enabled, the autostep setting allows the character to climb stairs automatically and walk over small obstacles. Autostepping requires the following parameters:

The maximum height the character can step over. If the vertical movement needed to step over this obstacle is larger than this value, then the character will be stopped by the obstacle.
The minimum (horizontal) width available on top of the obstacle. If, after the character is teleported on top of the obstacle, it cannot move forward by a distance larger than this minimum width, then the character will just be stopped by the obstacle (without being moved to the top of the obstacle).
Whether or not autostepping is enabled for dynamic bodies. If it is not enabled for dynamic bodies, the character won’t attempt to automatically step over small dynamic bodies. Disabling this can be useful if we want the character to push these small objects (see collisions) instead of just stepping over them.
The following depicts (top) one configuration where all the autostepping conditions are satisfied, and, (bottom) two configurations where these conditions are not all satisfied (left: because the width of the step is too small, right: because the height of the step is too large):

autostepping

info
Autostepping will only activate if the character is touching the floor right before the obstacle. This prevents the player from being teleported on to of a platform while it is in the air.

// Set autostep to None to disable it.
character_controller.autostep = None;
// Autostep if the step height is smaller than 0.5, and its width larger than 0.2.
character_controller.autostep = Some(CharacterAutostep {
    max_height: CharacterLength::Absolute(0.5),
    min_width: CharacterLength::Absolute(0.2),
    include_dynamic_bodies: true,
});
// Autostep if the step height is smaller than 0.5 multiplied by the character’s height,
// and its width larger than 0.5 multiplied by the character’s width (i.e. half the character’s
// width).
character_controller.autostep = Some(CharacterAutostep {
    max_height: CharacterLength::Relative(0.3),
    min_width: CharacterLength::Relative(0.5),
    include_dynamic_bodies: true,
});

Snap-to-ground
If enabled, snap-to-ground will force the character to stick to the ground if the following conditions are met simultaneously:

At the start of the movement, the character touches the ground.
The movement has a slight downward component.
At the end of the desired movement, the character would be separated from the ground by a distance smaller than the distance provided by the snap-to-ground parameter.
If these conditions are met, the character is automatically teleported down to the ground at the end of its motion. Typical usages of snap-to-ground include going downstairs or remaining in contact with the floor when moving downhill.

snap-to-ground

// Set snap-to-ground to None to disable it.
character_controller.snap_to_ground = None;
// Snap to the ground if the vertical distance to the ground is smaller than 0.5.
character_controller.snap_to_ground = Some(CharacterLength::Absolute(0.5));
// Snap to the ground if the vertical distance to the ground is smaller than 0.2 times the character’s height.
character_controller.snap_to_ground = Some(CharacterLength::Relative(0.2));

Filtering
It is possible to let the character controller ignore some obstacles. This is achieved by configuring the filter argument of the KinematicCharacterController::move_shape method. This QueryFilter structure is detailed in the scene query filters section.

warning
If the character-controller is used to move a collider (and the rigid-body it may be attached to) that is present in the physics scene, the filters must be used to exclude that collider (and that rigid-body) from the set of obstacles (with QueryFilter::exclude_collider and QueryFilter::exclude_rigid_body) to prevent the character from colliding with itself.

Collisions
As the character moves along its path, it will hit grounds and obstacles before sliding or stepping on them. Knowing what collider was hit on this path, and where the hit took place, can be valuable to apply various logic (custom forces, sound effects, etc.) This is why a set of character collision events are collected during the calculation of its trajectory.

info
The character collision events are given in chronological order. For example, if, during the resolution of the character motion, the character hits an obstacle A, then slides against it, and then hits another obstacle B. The collision with A will be reported first, and the collision with B will be reported second.

let character_controller = KinematicCharacterController::default();
// Use a closure to handle or collect the collisions while
// the character is being moved.
character_controller.move_shape(
    dt,
    &query_pipeline,
    character_shape,
    character_pos,
    desired_translation,
    |collision| { /* Handle or collect the collision in this closure. */ },
);

Unless dynamic bodies are filtered-out by the character controller’s filters, they may be hit during the resolution of the character movement. If that happens, these dynamic bodies will generally not react to (i.e. not be pushed by) the character because the character controller’s offset prevents actual contacts from happening.

In these situations forces need to be applied manually to this rigid-bodies. The character controller can apply these forces for you if needed:

// First, collect all the collisions.
let mut collisions = vec![];
character_controller.move_shape(
    dt,
    &query_pipeline,
    character_shape,
    &character_pos,
    desired_translation,
    |collision| collisions.push(collision),
);
// Then, let the character controller solve (and apply) the collision impulses
// to the dynamic rigid-bodies hit along its path.
// Note that we need to init a QueryPipelineMut here (because the impulse
// application will modify rigid-bodies.
let mut query_pipeline_mut = broad_phase.as_query_pipeline_mut(
    narrow_phase.query_dispatcher(),
    &mut bodies,
    &mut colliders,
    filter,
);
character_controller.solve_character_collision_impulses(
    dt,
    &mut query_pipeline_mut,
    character_shape,
    character_mass,
    &collisions,
);

Gravity
Since you are responsible for providing the movement vector to the character controller at each frame, it is up to you to emulate gravity by adding a downward component to that movement vector.