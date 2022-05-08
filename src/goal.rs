use async_trait::async_trait;
use tokio::io::Result;

use crate::{image_helpers, protocol::Draw};
use image::io::Reader as ImageReader;

pub struct Goal {
    draw_commands: Vec<u8>,

    hitbox_x: u16,
    hitbox_y: u16,
}

// Measure the following variables with GIMP
const GOAL_IMAGE_WIDTH: u16 = 338;
const GOAL_IMAGE_HEIGHT: u16 = 600;
const HITBOX_WIDTH: u16 = 117;
const HITBOX_HEIGHT: u16 = 310;

impl Goal {
    fn new(image_x: u16, image_y: u16, hitbox_x: u16, hitbox_y: u16, fliph: bool) -> Self {
        let mut image = ImageReader::open("images/goal_v1_transparent.png")
            .unwrap()
            .decode()
            .unwrap();

        if fliph {
            image = image.fliph();
        }

        let draw_commands = image_helpers::draw_image(&image, image_x, image_y);
        // draw_commands.extend_from_slice(&image_helpers::draw_rect(
        //     hitbox_x,
        //     hitbox_y,
        //     HITBOX_WIDTH,
        //     HITBOX_HEIGHT,
        //     0xff000000,
        // ));

        Goal {
            draw_commands,
            hitbox_x,
            hitbox_y,
        }
    }

    pub fn left(screen_height: u16) -> Self {
        let image_x = 0;
        let image_y = (screen_height - GOAL_IMAGE_HEIGHT as u16) / 2;
        let hitbox_x = 0;
        let hitbox_y = image_y + (GOAL_IMAGE_HEIGHT - HITBOX_HEIGHT) / 2;

        Goal::new(image_x, image_y, hitbox_x, hitbox_y, false)
    }

    pub fn right(screen_width: u16, screen_height: u16) -> Self {
        let image_x = screen_width - GOAL_IMAGE_WIDTH;
        let image_y = (screen_height - GOAL_IMAGE_HEIGHT as u16) / 2;
        let hitbox_x = screen_width - HITBOX_WIDTH;
        let hitbox_y = image_y + (GOAL_IMAGE_HEIGHT - HITBOX_HEIGHT) / 2;

        Goal::new(image_x, image_y, hitbox_x, hitbox_y, true)
    }
}

#[async_trait]
impl Draw for Goal {
    async fn draw(&self, client: &mut crate::network::Client) -> Result<()> {
        client.write_bytes(&self.draw_commands).await
    }
}
