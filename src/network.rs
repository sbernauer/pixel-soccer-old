use std::io::{Error, ErrorKind};

use lazy_static::lazy_static;
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream, Result},
    net::TcpStream,
};

use crate::protocol::PixelflutCommand;

pub const AVG_PIXELS_PER_COMMAND: usize = "PX 123 123 ffffff\n".len();
const READ_BUFFER_SIZE: usize = 64;

lazy_static! {
    pub static ref SIZE_COMMAND_REGEX: Regex =
        Regex::new(r"^(?i)\s*SIZE\s+([[:digit:]]+)\s+([[:digit:]]+)\s*$").unwrap();
}

pub struct Client {
    stream: BufStream<TcpStream>,
}

impl Client {
    pub async fn new(server_address: &str) -> Result<Self> {
        let stream = TcpStream::connect(server_address).await?;
        Ok(Client {
            stream: BufStream::new(stream),
        })
    }

    pub async fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn write_command(&mut self, cmd: &PixelflutCommand) -> Result<()> {
        // let mut bytes = Vec::new();
        // cmd.write_to_vec(&mut bytes);
        // self.stream.write_all(&bytes).await?;
        match cmd {
            PixelflutCommand::Size => self.stream.write_all("SIZE\n".as_bytes()).await?,
            PixelflutCommand::SetPixel { x, y, rgb } => {
                self.stream
                    .write_all(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
                    .await?
            }
            PixelflutCommand::GetPixel { .. } => (),
        }

        self.stream.flush().await?;

        Ok(())
    }

    pub async fn write_command_and_read(&mut self, cmd: &PixelflutCommand) -> Result<String> {
        self.write_command(cmd).await?;
        let mut buffer = String::with_capacity(READ_BUFFER_SIZE);
        self.stream.read_line(&mut buffer).await?;

        Ok(buffer)
    }

    pub async fn read_screen_size(&mut self) -> Result<(u16, u16)> {
        let response = self.write_command_and_read(&PixelflutCommand::Size).await?;

        // Find captures in the data, return the result
        match SIZE_COMMAND_REGEX.captures(&response) {
            Some(matches) => Ok((
                matches[1]
                    .parse::<u16>()
                    .expect("Failed to parse screen width, received malformed data"),
                matches[2]
                    .parse::<u16>()
                    .expect("Failed to parse screen height, received malformed data"),
            )),
            None => Err(Error::new(
                ErrorKind::Other,
                "Failed to parse screen size, received malformed data",
            )),
        }
    }
}
