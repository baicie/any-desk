pub mod network;

use bytes::Bytes;
use enigo::{Enigo, KeyboardControllable, MouseControllable};
use ffmpeg_next as ffmpeg;
use scrap::{Capturer, Display};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::APIBuilder;
use webrtc::peer_connection::RTCPeerConnection;

pub use network::{ConnectionInfo, NetworkManager};

pub struct RemoteDesktop {
    peer_connection: Arc<RTCPeerConnection>,
    encoder: ffmpeg::encoder::Encoder,
    network: Arc<Mutex<NetworkManager>>,
}

impl RemoteDesktop {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        ffmpeg::init()?;

        let api = APIBuilder::new().build();
        let peer_connection = api.new_peer_connection().await?;
        let encoder = Self::setup_encoder()?;
        let network = Arc::new(Mutex::new(NetworkManager::new().await?));

        Ok(Self {
            peer_connection: Arc::new(peer_connection),
            encoder,
            network,
        })
    }

    fn setup_encoder() -> Result<ffmpeg::encoder::Encoder, ffmpeg::Error> {
        // 尝试使用硬件编码器
        let encoder_names = ["hevc_nvenc", "hevc_vaapi", "libx265"];

        for name in encoder_names {
            if let Ok(codec) = ffmpeg::encoder::find_encoder_by_name(name) {
                println!("Using encoder: {}", name);
                return Ok(codec);
            }
        }

        Err(ffmpeg::Error::EncoderNotFound)
    }

    pub async fn start_streaming(&self) -> Result<(), Box<dyn std::error::Error>> {
        let display = Display::primary()?;
        let mut capturer = Capturer::new(display)?;
        let network = self.network.clone();

        // 使用线程安全的通道
        let (tx, rx) = std::sync::mpsc::channel();

        // 在单独的线程中捕获帧
        std::thread::spawn(move || loop {
            if let Ok(frame) = capturer.frame() {
                let _ = tx.send(frame);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        });

        // 在异步任务中处理帧
        tokio::spawn(async move {
            while let Ok(frame) = rx.recv() {
                if let Ok(encoded) = self.encode_frame(&frame) {
                    let bytes = Bytes::copy_from_slice(&encoded);
                    let _ = network.lock().await.send_video_frame(&bytes).await;
                }
            }
        });

        Ok(())
    }

    pub fn handle_input(&self, event: InputEvent) -> Result<(), Box<dyn std::error::Error>> {
        let mut enigo = Enigo::new();
        match event {
            InputEvent::MouseMove { x, y } => {
                enigo.mouse_move_to(x, y);
            }
            InputEvent::MouseClick { button } => {
                enigo.mouse_click(button);
            }
            InputEvent::KeyPress { key } => {
                enigo.key_click(key);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: enigo::MouseButton },
    KeyPress { key: enigo::Key },
}
