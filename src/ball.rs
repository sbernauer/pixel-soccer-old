use std::f32::consts::PI;

use image::{io::Reader as ImageReader, DynamicImage};

use crate::image_helpers;

pub struct Ball {
    image: DynamicImage,
    draw_commands: Vec<u8>,

    center_x: f32,
    center_y: f32,
    radius: f32,
    dir: f32,
    speed: f32,

    screen_width: u16,
    screen_height: u16,
}

const SPEED: f32 = 2_f32;

// Measure the following variables with GIMP
const BALL_IMAGE_DIAMETER: u16 = 80; // Assuming quadratic image the width and height of the image

impl Ball {
    pub fn new(screen_width: u16, screen_height: u16) -> Self {
        let image = ImageReader::open("images/ball_v1.png")
            .unwrap()
            .decode()
            .unwrap();

        let mut ball = Ball {
            image,
            draw_commands: vec![],
            center_x: ((screen_width - BALL_IMAGE_DIAMETER) / 2) as f32,
            center_y: ((screen_height - BALL_IMAGE_DIAMETER) / 2) as f32,
            radius: (BALL_IMAGE_DIAMETER / 2) as f32,
            dir: PI / 4_f32,
            speed: SPEED,
            screen_width,
            screen_height,
        };
        ball.update_draw_commands();
        ball
    }

    pub fn tick(&mut self) {
        self.center_x += self.speed * self.dir.cos();
        self.center_y += self.speed * self.dir.sin();

        if self.center_x - self.radius <= 0_f32
            || self.center_y - self.radius <= 0_f32
            || self.center_x + self.radius >= self.screen_width as f32
            || self.center_y + self.radius >= self.screen_height as f32
        {
            self.dir += PI / 4_f32;
        }

        self.update_draw_commands();
    }

    fn update_draw_commands(&mut self) {
        self.draw_commands = image_helpers::draw_image(
            &self.image,
            (self.center_x - self.radius) as u16,
            (self.center_y - self.radius) as u16,
        );
    }

    pub async fn draw(&self, client: &mut crate::network::Client) -> std::io::Result<()> {
        client.write_bytes(&self.draw_commands).await
    }
}
