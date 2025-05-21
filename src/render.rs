use macroquad::prelude::*;

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn clear(&self) {
        clear_background(DARKGRAY);
    }

    pub fn set_camera(&self, camera: &Camera3D) {
        set_camera(camera);
    }

    pub fn draw_player(
        &self,
        x: f32,
        y: f32,
        color: Color,
    ) {
        draw_cube(
            vec3(x, 0.5, y),
            vec3(1., 1., 1.),
            None,
            color,
        );
    }
}