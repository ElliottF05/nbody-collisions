use crate::simulation::body::{self, Body};
use crate::simulation::vec2::Vec2;

/// The main simulation struct, containing bodies and simulation parameters and responsible 
/// for updating the simulation state.
pub struct Simulation {
    grav_constant: f32,
    coeff_restitution: f32,
    bodies: Vec<Body>,
}

impl Simulation {
    /// Creates a new simulation with default parameters and bodies.
    pub fn new() -> Self {
        let num_bodies = 500;
        let mut bodies = vec![];

        let radius = 500.0;
        for _ in 0..num_bodies {
            let mut pos = Vec2::new(rand::random(), rand::random());
            pos = (pos - Vec2::new(0.5, 0.5)) * radius;
            // let vel = Vec2::new(pos.y, -pos.x).normalize() * (1.0 / (pos.length() + 0.1).sqrt()) * 800.0;
            let vel = Vec2::zero();
            bodies.push(Body::new(
                pos,
                vel,
                5.0,
                1.0,
            ));
        }

        Simulation {
            grav_constant: 600.0,
            coeff_restitution: 0.95,
            bodies,
        }
    }

    /// Updates the simulation state by a time step `dt`.
    pub fn update(&mut self, dt: f32) {

        // update acceleration
        for i in 0..self.bodies.len() {
            let mut accel = Vec2::zero();
            for j in 0..self.bodies.len() {
                if i == j {
                    continue;
                }
                let direction = self.bodies[j].position - self.bodies[i].position;
                let distance_sq = direction.length_squared().max(0.001);
                let f = self.grav_constant * self.bodies[j].mass / distance_sq;
                accel += direction.normalize() * f;
            }
            self.bodies[i].velocity += accel * dt;
        }

        // update positions
        for body in self.bodies.iter_mut() {
            body.position += body.velocity * dt;
        }

        // solve collisions
        for i in 0..self.bodies.len() {
            for j in i+1..self.bodies.len() {

                let direction = self.bodies[j].position - self.bodies[i].position;
                let distance = direction.length();
                let min_distance = self.bodies[i].radius + self.bodies[j].radius;
                let constraint_distance = distance - min_distance;

                if constraint_distance < 0.0 {
                    let normal = direction.normalize();
                    let relative_velocity = self.bodies[j].velocity - self.bodies[i].velocity;
                    let constraint_velocity = relative_velocity.dot(normal);
                    
                    if constraint_velocity < 0.0 {
                        let body_i_mass = self.bodies[i].mass;
                        let body_j_mass = self.bodies[j].mass;
                        let denom = 1.0 / body_i_mass + 1.0 / body_j_mass;
                        
                        // velocity correction (impulse)
                        let impulse = (1.0 + self.coeff_restitution) * constraint_velocity / denom;
                        let impulse_vec = normal * impulse;
                        self.bodies[i].velocity += impulse_vec / body_i_mass;
                        self.bodies[j].velocity -= impulse_vec / body_j_mass;
                    }
                    
                    // position correction
                    let correction_amount = constraint_distance.abs() * 1.0; // 80% correction
                    let correction = normal * correction_amount;
                    let total_mass = self.bodies[i].mass + self.bodies[j].mass;
                    let body_i_mass = self.bodies[i].mass;
                    let body_j_mass = self.bodies[j].mass;
                    self.bodies[i].position -= correction * (body_j_mass / total_mass);
                    self.bodies[j].position += correction * (body_i_mass / total_mass);

                    // tangent velocity correction (friction-like)
                    let tangent_velocity = relative_velocity - normal * constraint_velocity;
                    let tangent_speed = tangent_velocity.length();
                    if tangent_speed > 0.0 {
                        let friction_impulse_magnitude = tangent_speed * 0.5; // friction coefficient
                        let friction_impulse = tangent_velocity.normalize() * friction_impulse_magnitude;
                        let body_i_mass = self.bodies[i].mass;
                        let body_j_mass = self.bodies[j].mass;
                        self.bodies[i].velocity += friction_impulse / body_i_mass;
                        self.bodies[j].velocity -= friction_impulse / body_j_mass;
                    }
                
                }
            }
        }
    }

    /// Returns a reference to the bodies in the simulation.
    pub fn get_bodies(&self) -> &[Body] {
        &self.bodies
    }
}