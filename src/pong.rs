use nalgebra::Vector2;
use rand::Rng;
use rapier2d::prelude::*;
use rapier2d::{na::vector, pipeline::PhysicsPipeline};

pub const PONG_WIDTH: f32 = 100.0;
pub const PONG_HEIGHT: f32 = PONG_WIDTH / 3.0 * 2.0;
pub const BALL_RADIUS: f32 = 1.2;
pub const BALL_START: (f32, f32) = (PONG_WIDTH / 2.0, PONG_HEIGHT / 2.0);
pub const PLAYER_WIDTH: f32 = 1.3;
pub const PLAYER_HEIGHT: f32 = 8.0;
pub const P1_START: (f32, f32) = (0.0, PONG_HEIGHT / 2.0 - PLAYER_HEIGHT / 2.0);
pub const P2_START: (f32, f32) = (
    PONG_WIDTH - PLAYER_WIDTH,
    PONG_HEIGHT / 2.0 - PLAYER_HEIGHT / 2.0,
);

fn create_wall_collider(h_width: f32, h_height: f32, x: f32, y: f32) -> Collider {
    ColliderBuilder::cuboid(h_width, h_height)
        .friction(0.0)
        .translation(Vector2::new(x, y))
        .build()
}

fn create_player_collider(h_width: f32, h_height: f32) -> Collider {
    ColliderBuilder::cuboid(h_width, h_height)
        .friction(0.0)
        .restitution(1.2) // increase by 20% per shot
        .restitution_combine_rule(CoefficientCombineRule::Max)
        .build()
}

struct PongPhysicsHooks;

impl PhysicsHooks for PongPhysicsHooks {
    fn modify_solver_contacts(&self, context: &mut ContactModificationContext) {
        for solver_contact in &mut *context.solver_contacts {}
    }
}

pub struct Pong {
    execute_step: Box<dyn FnMut(Option<(f32, f32)>) -> (f32, f32, bool, bool) + Send>,
}

impl Pong {
    pub fn new(speed_multiplier: f32) -> Self {
        let h_width = PONG_WIDTH / 2.0;
        let h_height = PONG_HEIGHT / 2.0;
        let h_wall_thickness = 25.0;
        let h_player_width = PLAYER_WIDTH / 2.0;
        let h_player_height = PLAYER_HEIGHT / 2.0;

        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // left wall
        let left_wall_collider_handle = collider_set.insert(create_wall_collider(
            h_wall_thickness,
            h_height,
            0.0 - h_wall_thickness,
            h_height,
        ));

        // right wall
        let right_wall_collider_handle = collider_set.insert(create_wall_collider(
            h_wall_thickness,
            h_height,
            PONG_WIDTH + h_wall_thickness,
            h_height,
        ));

        // top wall
        collider_set.insert(create_wall_collider(
            h_width,
            h_wall_thickness,
            h_width,
            0.0 - h_wall_thickness,
        ));

        // bottom wall
        collider_set.insert(create_wall_collider(
            h_width,
            h_wall_thickness,
            h_width,
            PONG_HEIGHT + h_wall_thickness,
        ));

        let p1_body_handle = rigid_body_set.insert(
            RigidBodyBuilder::kinematic_position_based()
                .translation(vector![
                    P1_START.0 + h_player_width,
                    P1_START.1 + h_player_height
                ])
                .build(),
        );

        let p1_collider_handle = collider_set.insert_with_parent(
            create_player_collider(h_player_width, h_player_height),
            p1_body_handle,
            &mut rigid_body_set,
        );

        let p2_body_handle = rigid_body_set.insert(
            RigidBodyBuilder::kinematic_position_based()
                .translation(vector![
                    P2_START.0 - h_player_width,
                    P2_START.1 + h_player_height
                ])
                .build(),
        );

        let p2_collider_handle = collider_set.insert_with_parent(
            create_player_collider(h_player_width, h_player_height),
            p2_body_handle,
            &mut rigid_body_set,
        );

        let ball_body_handle = rigid_body_set.insert(
            RigidBodyBuilder::dynamic()
                .ccd_enabled(true)
                .translation(vector![BALL_START.0, BALL_START.1])
                .build(),
        );

        let ball_collider_handle = collider_set.insert_with_parent(
            ColliderBuilder::ball(BALL_RADIUS)
                .restitution(1.0)
                .restitution_combine_rule(CoefficientCombineRule::Max)
                .friction(0.0)
                .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS)
                .build(),
            ball_body_handle,
            &mut rigid_body_set,
        );

        let mut rng = rand::thread_rng();
        let initial_velocity = Vector2::new(
            rng.gen_range(7.0..10.0) * (if rng.gen() { 1.0 } else { -1.0 }) * speed_multiplier,
            rng.gen_range(2.5..4.0) * (if rng.gen() { 1.0 } else { -1.0 }) * speed_multiplier,
        );
        rigid_body_set[ball_body_handle].set_linvel(initial_velocity, true);

        let integration_parameters = IntegrationParameters::default();
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = BroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut physics_hooks = PongPhysicsHooks {};

        let execute_step = move |updated_position: Option<(f32, f32)>| {
            if let Some((p1_y, p2_y)) = updated_position {
                let p1_body = &mut rigid_body_set[p1_body_handle];
                p1_body.set_next_kinematic_translation(vector![
                    p1_body.translation().x,
                    p1_y + h_player_height
                ]);

                let p2_body = &mut rigid_body_set[p2_body_handle];
                p2_body.set_next_kinematic_translation(vector![
                    p2_body.translation().x,
                    p2_y + h_player_height
                ]);
            }

            physics_pipeline.step(
                &Vector::zeros(),
                &integration_parameters,
                &mut island_manager,
                &mut broad_phase,
                &mut narrow_phase,
                &mut rigid_body_set,
                &mut collider_set,
                &mut impulse_joint_set,
                &mut multibody_joint_set,
                &mut ccd_solver,
                None,
                &mut physics_hooks,
                &(),
            );

            let left_wall_contact = narrow_phase
                .contact_pair(ball_collider_handle, left_wall_collider_handle)
                .is_some_and(|cp| cp.has_any_active_contact);

            let right_wall_contact = narrow_phase
                .contact_pair(ball_collider_handle, right_wall_collider_handle)
                .is_some_and(|cp| cp.has_any_active_contact);

            let p1_contact = narrow_phase
                .contact_pair(ball_collider_handle, p1_collider_handle)
                .is_some_and(|cp| cp.has_any_active_contact);

            let p2_contact = narrow_phase
                .contact_pair(ball_collider_handle, p2_collider_handle)
                .is_some_and(|cp| cp.has_any_active_contact);

            if p1_contact {
                eprintln!("p1 contact!");
            }

            if p2_contact {
                eprintln!("p2 contact!");
            }

            let ball_body = &rigid_body_set[ball_body_handle];

            (
                ball_body.translation().x,
                ball_body.translation().y,
                left_wall_contact,
                right_wall_contact,
            )
        };

        Self {
            execute_step: Box::new(execute_step),
        }
    }

    pub fn next(&mut self, updated_position: Option<(f32, f32)>) -> (f32, f32, bool, bool) {
        (self.execute_step)(updated_position)
    }
}
