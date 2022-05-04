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
    // /// Write yourself to the stream regardless if it's just a buffer or a tcpstream
    // #[inline(always)]
    // pub async fn write_to_stream(&self, stream: &mut BufStream<TcpStream>) -> Result<()> {
    //     match self {
    //         PixelflutCommand::Size => stream.write_all("SIZE\n".as_bytes()),
    //         PixelflutCommand::SetPixel { x, y, rgb } => {
    //             stream
    //                 .write_all(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
    //         }
    //         PixelflutCommand::SetPixelWithAlpha { x, y, rgba } => {
    //             stream
    //                 .write_all(format!("PX {x} {y} {rgba:08x}\n").as_bytes())
    //         }
    //         PixelflutCommand::GetPixel { .. } => Ok(()),
    //     }
    // }

    #[inline(always)]
    pub fn write_to_vec(&self, vec: &mut Vec<u8>) {
        match self {
            PixelflutCommand::Size => vec.extend_from_slice("SIZE\n".as_bytes()),
            PixelflutCommand::SetPixel { x, y, rgb } => {
                vec.extend_from_slice(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
            }
            PixelflutCommand::GetPixel { .. } => (),
        }
    }
}
