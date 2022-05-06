use args::Args;
use ball::Ball;
use clap::StructOpt;
use goal::Goal;
use network::Client;
use tokio::io::Result;

mod args;
mod ball;
mod goal;
mod image_helpers;
mod network;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut threads = vec![];

    let server_address = args.server_address.clone();
    threads.push(tokio::spawn(async move {
        let (mut client, _, screen_height) = setup_thread(&server_address).await;
        let goal = Goal::left(screen_height);
        loop {
            goal.draw(&mut client).await.unwrap();
        }
    }));

    let server_address = args.server_address.clone();
    threads.push(tokio::spawn(async move {
        let (mut client, screen_width, screen_height) = setup_thread(&server_address).await;
        let goal = Goal::right(screen_width, screen_height);
        loop {
            goal.draw(&mut client).await.unwrap();
        }
    }));

    let server_address = args.server_address.clone();
    threads.push(tokio::spawn(async move {
        let (mut client, screen_width, screen_height) = setup_thread(&server_address).await;
        let mut ball = Ball::new(screen_width, screen_height);
        loop {
            ball.tick();
            ball.draw(&mut client).await.unwrap();
        }
    }));

    for thread in threads {
        thread.await?;
    }

    Ok(())
}

async fn setup_thread(server_address: &str) -> (Client, u16, u16) {
    let mut client = Client::new(server_address).await.unwrap();
    let (screen_width, screen_height): (u16, u16) = client.read_screen_size().await.unwrap();

    (client, screen_width, screen_height)
}
