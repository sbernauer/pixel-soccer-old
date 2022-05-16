use async_trait::async_trait;
use atomic_float::AtomicF32;
use std::{
    f32::consts::PI,
    sync::{
        atomic::Ordering::{Acquire, Release},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::RwLock,
    task::JoinHandle,
    time::{self, Instant},
};

use image::{io::Reader as ImageReader, DynamicImage};

use crate::{client::Client, image_helpers, protocol::Draw};

pub struct Ball {
    image: DynamicImage,
    // If anybody has a better idea on how to share the ball between one writing and multiple reading threads please reach out to me!
    draw_commands: RwLock<Vec<u8>>,

    center_x: AtomicF32,
    center_y: AtomicF32,
    dir: AtomicF32,
    radius: f32,
    speed: f32,

    screen_width: u16,
    screen_height: u16,
}

const SPEED: f32 = 10.0_f32;
const LOOKAHEAD_LINE_LENGTH: f32 = 100_f32; // Keep in mind that for a distance of BALL_IMAGE_DIAMETER / 2 nothing will be looked at because it's inside the ball.

const TARGET_COLOR: u32 = 0x00ff0000; // red
const TARGET_COLOR_THRESHOLD: f32 = 0.99f32;

// Measure the following variables with GIMP
const BALL_IMAGE_DIAMETER: u16 = 80; // Assuming quadratic image the width and height of the image

impl Ball {
    pub async fn new(screen_width: u16, screen_height: u16) -> Self {
        let image = ImageReader::open("images/ball_v1.png")
            .unwrap()
            .decode()
            .unwrap();

        let ball = Ball {
            image,
            draw_commands: RwLock::new(vec![]),
            center_x: AtomicF32::new(((screen_width - BALL_IMAGE_DIAMETER) / 2) as f32),
            center_y: AtomicF32::new(((screen_height - BALL_IMAGE_DIAMETER) / 2) as f32),
            radius: (BALL_IMAGE_DIAMETER / 2) as f32,
            dir: AtomicF32::new(-PI / 0.9_f32),
            speed: SPEED,
            screen_width,
            screen_height,
        };
        *(ball.draw_commands.write().await) = ball.get_draw_commands();
        ball
    }

    fn get_draw_commands(&self) -> Vec<u8> {
        image_helpers::draw_image(
            &self.image,
            (self.center_x.load(Acquire) - self.radius) as u16,
            (self.center_y.load(Acquire) - self.radius) as u16,
        )
    }
}

#[async_trait]
impl Draw for Ball {
    async fn draw(&self, client: &mut crate::client::Client) -> std::io::Result<()> {
        client.write_bytes(&self.draw_commands.read().await).await
    }
}

pub fn start_main_thread(ball: Arc<Ball>, mut client: Client, fps: u16) -> JoinHandle<()> {
    let mut interval = time::interval(Duration::from_millis(1_000 / fps as u64));

    tokio::spawn(async move {
        let mut fps_counter_last_update = Instant::now();
        let mut fps_counter = 0;

        loop {
            interval.tick().await;

            if fps_counter_last_update.elapsed() >= Duration::from_secs(1) {
                println!("{} fps", fps_counter);
                fps_counter = 0;
                fps_counter_last_update = Instant::now();
            } else {
                fps_counter += 1;
            }

            let mut new_dir = ball.dir.load(Acquire);
            let mut new_center_x = ball.center_x.load(Acquire);
            let mut new_center_y = ball.center_y.load(Acquire);
            let mut movement_x = ball.speed * new_dir.cos();
            let mut movement_y = ball.speed * new_dir.sin();
            new_center_x += movement_x;
            new_center_y += movement_y;

            // Collision on left or right
            if new_center_x - ball.radius <= 0_f32
                || new_center_x + ball.radius >= ball.screen_width as f32
            {
                movement_x *= -1_f32;
            }

            // Collision on top or bottom
            if new_center_y - ball.radius <= 0_f32
                || new_center_y + ball.radius >= ball.screen_height as f32
            {
                movement_y *= -1_f32;
            }

            // If we are not to close to some edge check if we collude with enough red pixels
            let line_front_point_x = new_center_x + LOOKAHEAD_LINE_LENGTH * new_dir.cos();
            let line_front_point_y = new_center_y + LOOKAHEAD_LINE_LENGTH * new_dir.sin();

            if line_front_point_x > 0_f32
                && line_front_point_x < ball.screen_width as f32
                && line_front_point_y > 0_f32
                && line_front_point_y < ball.screen_height as f32
            {
                let line_coordinates = image_helpers::coordinates_on_line_with_dir_and_skip_offset(
                    new_center_x,
                    new_center_y,
                    new_dir,
                    LOOKAHEAD_LINE_LENGTH,
                    ball.radius,
                );
                let line_colors = client.get_color_of_pixels(&line_coordinates).await.unwrap();
                let num_line_colors = line_colors.len();
                let target_color_count = line_colors
                    .iter()
                    .filter(|color| **color == TARGET_COLOR)
                    .count() as f32;
                let target_color_percentage = target_color_count / num_line_colors as f32;

                // println! {"target_color_percentage: {target_color_percentage:?} ({target_color_count}/{num_line_colors})"};
                if target_color_percentage >= TARGET_COLOR_THRESHOLD {
                    // TODO figure out correct bouncing
                    movement_x *= -1_f32;
                    // movement_y *= -1_f32;
                }
            }

            new_dir = movement_y.atan2(movement_x);

            ball.center_x.store(new_center_x, Release);
            ball.center_y.store(new_center_y, Release);
            ball.dir.store(new_dir, Release);

            let new_draw_commands = ball.get_draw_commands();
            *ball.draw_commands.write().await = new_draw_commands;
        }
    })
}
