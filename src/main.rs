use std::sync::Arc;

use args::Args;
use ball::Ball;
use clap::StructOpt;
use client::Client;
use goal::Goal;
use tokio::io::Result;
use tokio::task::JoinHandle;

use crate::protocol::Draw;

mod args;
mod ball;
mod client;
mod goal;
mod image_helpers;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut client = Client::new(&args.server_address).await.unwrap();
    let (screen_width, screen_height) = client.read_screen_size().await.unwrap();

    let ball = Arc::new(Ball::new(screen_width, screen_height).await);
    let goal_left = Arc::new(Goal::left(screen_height));
    let goal_right = Arc::new(Goal::right(screen_width, screen_height));

    let mut threads = vec![ball::start_main_thread(Arc::clone(&ball), client, 10)];
    threads.extend(start_drawing(ball, &args.server_address, 5).await);
    threads.extend(start_drawing(goal_left, &args.server_address, 1).await);
    threads.extend(start_drawing(goal_right, &args.server_address, 1).await);

    for thread in threads {
        thread.await?;
    }

    Ok(())
}

async fn start_drawing(
    object: Arc<impl Draw + std::marker::Send + std::marker::Sync + 'static>,
    server_address: &str,
    num_threads: u16,
) -> Vec<JoinHandle<()>> {
    let mut threads = vec![];

    for _ in 0..num_threads {
        let mut client = Client::new(server_address).await.unwrap();
        let object_clone = object.clone();

        let thread = tokio::spawn(async move {
            loop {
                object_clone.draw(&mut client).await.unwrap();
            }
        });
        threads.push(thread);
    }

    threads
}
