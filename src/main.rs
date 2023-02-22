#![allow(clippy::unnecessary_wraps)]

use ggez::{
    event,
    glam::*,
    graphics::{self, Color, DrawParam},
    Context, GameResult,
};

use std::vec::Vec;

struct VerletObjects {
    pos: Vec<Vec2>,
    old_pos: Vec<Vec2>,
    accel: Vec<Vec2>,
    link: Vec<(usize, usize, f32)>,
    count: usize,
}

impl VerletObjects {
    fn new() -> VerletObjects {
        VerletObjects {
            pos: Vec::with_capacity(100),
            old_pos: Vec::with_capacity(100),
            accel: Vec::with_capacity(100),
            link: Vec::with_capacity(100),
            count: 0
        }
    }

    fn update_positions(&mut self, dt: f32) {
        for i in 0..self.count {
            self.update_position(dt, i);
        }
    }

    fn update_position(&mut self, dt: f32, index: usize) {
        let vel = self.pos[index] - self.old_pos[index];
        self.old_pos[index] = self.pos[index];
        self.pos[index] = self.pos[index] + vel + self.accel[index] * (dt * dt);
        self.accel[index] = vec2(0., 0.);
    }

    fn get(&self, index: usize) -> Vec2 {
        return self.pos[index];
    }

    fn add(&mut self, pos: Vec2) {
        self.pos.push(pos);
        self.old_pos.push(pos);
        self.accel.push(vec2(0., 0.));

        self.count = self.count + 1;
    }

    fn add_link(&mut self, p1: usize, p2: usize) {
        let dist = self.pos[p1] - self.pos[p2];
        self.link.push((p1, p2, dist.length()))
    }

    fn apply_gravity(&mut self) {
        for i in 0..self.count {
            self.accel[i] += vec2(0., 1000.0);
        }
    }

    fn solve_links(&mut self) {
        for i in 0..self.link.len() {
            let link = self.link[i];
            let v = self.pos[link.0] - self.pos[link.1];
            let dist = v.length();

            let n = v / dist;
            let delta = link.2 - dist;
            self.pos[link.0] += 0.5 * delta * n;
            self.pos[link.1] -= 0.5 * delta * n;
        }
    }

    fn apply_constraint(&mut self) {
        let pos = vec2(400.0, 300.0);
        let radius = 300.0;

        for i in 0..self.count {
            let to_obj = self.pos[i] - pos;
            let dist = to_obj.length();

            if dist > radius - 50.0 {
                let n = to_obj / dist;
                self.pos[i] = pos + n * (300.0 - 50.0);
            }
        }
    }

    fn check_collisions(&mut self, dt: f32) {
        for i in 0..self.count {
            for k in (i+1)..self.count {
                let v = self.pos[i] - self.pos[k];
                let dist2 = v.length_squared();
                let m_dist = 100.0;

                if dist2 < (m_dist * m_dist) {
                    let dist = v.length();
                    let n = v / dist;

                    let delta = 0.5 * 0.75 * (dist - m_dist);
                    self.pos[i] -= n * (0.5 * delta);
                    self.pos[k] += n * (0.5 * delta);
                }
            }
        }
    }
}

struct MainState {
    circle_area: graphics::Mesh,
    circle: graphics::Mesh,
    verlet_state: VerletObjects,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut verlet_state = VerletObjects::new();
        verlet_state.add(vec2(300.0, 300.0)); // 0
        verlet_state.add(vec2(400.0, 500.0)); // 1
        verlet_state.add(vec2(500.0, 300.0)); // 2

        verlet_state.add_link(0, 1);
        verlet_state.add_link(0, 2);
        verlet_state.add_link(1, 2);

         let circle_area = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            vec2(0., 0.),
            300.0,
            1.0,
            Color::WHITE,
        )?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            50.0,
            1.0,
            Color::WHITE,
        )?;

        Ok(MainState { circle_area: circle_area, circle: circle, verlet_state: verlet_state })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        //self.pos_x = self.pos_x % 800.0 + 1.0;

        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            let dt = 1.0 / (DESIRED_FPS as f32);
            let dt_substep = dt / 8.0; 

            //println!("DT: {}, DT_SUBSTEP: {}", dt, dt / 8);

            for i in 0..8 {
                self.verlet_state.apply_gravity();
                self.verlet_state.check_collisions(dt_substep);
                self.verlet_state.solve_links();
                self.verlet_state.apply_constraint();
                self.verlet_state.update_positions(dt_substep);
            }
        }


        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(&self.circle_area, vec2(400.0, 300.0));
        canvas.draw(&self.circle, self.verlet_state.get(0));
        canvas.draw(&self.circle, self.verlet_state.get(1));
        canvas.draw(&self.circle, self.verlet_state.get(2));

        let line = graphics::Mesh::new_line(
            ctx,
            &[self.verlet_state.get(0), self.verlet_state.get(1)],
            2.,
            Color::WHITE)?;

        let line1 = graphics::Mesh::new_line(
            ctx,
            &[self.verlet_state.get(0), self.verlet_state.get(2)],
            2.,
            Color::WHITE)?;

        let line2 = graphics::Mesh::new_line(
            ctx,
            &[self.verlet_state.get(1), self.verlet_state.get(2)],
            2.,
            Color::WHITE)?;
        
        canvas.draw(&line, DrawParam::default());
        canvas.draw(&line1, DrawParam::default());
        canvas.draw(&line2, DrawParam::default());

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}