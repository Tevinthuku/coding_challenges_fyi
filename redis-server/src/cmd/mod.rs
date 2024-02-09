use crate::resp::Frame;

pub fn execute_command(frame: Frame) -> Frame {
    println!("frame: {:?}", frame);
    match frame {
        Frame::Array(frames) => {
            if let Some(Frame::BulkString { content, length: _ }) = frames.first() {
                if content == "PING" {
                    return Frame::SimpleString("PONG".to_owned());
                }
            }
            Frame::Error("ERR unknown command".to_owned())
        }
        _ => unimplemented!(),
    }
}
