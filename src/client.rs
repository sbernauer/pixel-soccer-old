use std::io::{Error, ErrorKind};

use lazy_static::{__Deref, lazy_static};
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream, Result},
    net::TcpStream,
};

use crate::protocol::PixelflutCommand;

pub const AVG_PIXELS_PER_COMMAND: usize = "PX 123 123 ffffff\n".len();
const READ_BUFFER_SIZE: usize = 64;

lazy_static! {
    // Thanks to https://github.com/timvisee/pixelpwnr/blob/0d83b3e0b54448a59844e330a36f2e4b0e19e611/src/pix/client.rs#L19
    pub static ref SIZE_COMMAND_REGEX: Regex =
        Regex::new(r"^(?i)\s*SIZE\s+([[:digit:]]+)\s+([[:digit:]]+)\s*$").unwrap();
        pub static ref READ_PIXEL_COMMAND_REGEX: Regex = Regex::new(r"PX [0-9]+ [0-9]+ ([0-9a-fA-F]+)\s").unwrap();
        // pub static ref READ_PIXEL_COMMAND_REGEX: Regex = Regex::new(r"^PX [[:digit:]]+ [[:digit:]]+ ([0-9a-f]{6}])\s$").unwrap();
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

    /// Slow, don't use for performant drawing
    pub async fn write_command(&mut self, cmd: &PixelflutCommand) -> Result<()> {
        match cmd {
            PixelflutCommand::Size => self.stream.write_all("SIZE\n".as_bytes()).await?,
            PixelflutCommand::SetPixel { x, y, rgb } => {
                self.stream
                    .write_all(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
                    .await?
            }
            PixelflutCommand::GetPixel { x, y } => {
                self.stream
                    .write_all(format!("PX {x} {y}\n").as_bytes())
                    .await?
            }
        }

        self.stream.flush().await?;

        Ok(())
    }

    /// Slow, don't use for performant drawing
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

    pub async fn get_color_of_pixels(&mut self, pixels: &Vec<(u16, u16)>) -> Result<Vec<u32>> {
        let num_pixels = pixels.len();
        let mut request = Vec::new();
        let mut result: Vec<u32> = Vec::new();

        for (x, y) in pixels {
            PixelflutCommand::GetPixel { x: *x, y: *y }.write_to_vec(&mut request);
        }

        // Ask for pixel colors
        self.write_bytes(&request).await?;

        // Read response
        let mut buffer = String::with_capacity(READ_BUFFER_SIZE);
        for _ in 0..num_pixels {
            let read = self.stream.read_line(&mut buffer).await?;
            // if read == 0 {
            //     break;
            // }

            let cap = READ_PIXEL_COMMAND_REGEX.captures(&buffer);
            if let Some(cap) = cap {
                if let Ok(rgb) = u32::from_str_radix(cap.get(1).unwrap().as_str(), 16) {
                    result.push(rgb);
                }
            }
        }

        Ok(result)
    }
}
