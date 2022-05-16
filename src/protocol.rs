use async_trait::async_trait;

pub enum PixelflutCommand {
    Size,
    /// Layout of rgb: 8 bits padding, 8 bits r, 8 bits g, 8 bits green
    SetPixel {
        x: u16,
        y: u16,
        rgb: u32,
    },
    GetPixel {
        x: u16,
        y: u16,
    },
}

impl PixelflutCommand {
    #[inline(always)]
    pub fn write_to_vec(&self, vec: &mut Vec<u8>) {
        match self {
            PixelflutCommand::Size => vec.extend_from_slice("SIZE\n".as_bytes()),
            PixelflutCommand::SetPixel { x, y, rgb } => {
                vec.extend_from_slice(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
            }
            PixelflutCommand::GetPixel { x, y } => {
                vec.extend_from_slice(format!("PX {x} {y}\n").as_bytes())
            }
        }
    }
}

#[async_trait]
pub trait Draw {
    async fn draw(&self, client: &mut crate::client::Client) -> std::io::Result<()>;
}
