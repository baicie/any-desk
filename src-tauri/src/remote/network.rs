use bytes::Bytes;
use parking_lot;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub offer: String,               // SDP offer
    pub ice_candidates: Vec<String>, // ICE candidates
}

pub struct NetworkManager {
    peer_connection: Arc<RTCPeerConnection>,
    video_track: Option<Arc<TrackLocalStaticSample>>,
    data_channel: Option<Arc<RTCDataChannel>>,
}

impl NetworkManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let api = APIBuilder::new().build();
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        Ok(Self {
            peer_connection,
            video_track: None,
            data_channel: None,
        })
    }

    // 创建并获取视频轨道
    pub async fn create_video_track(&mut self) -> Result<Arc<TrackLocalStaticSample>, Error> {
        let track = Arc::new(TrackLocalStaticSample::new(
            webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                mime_type: "video/h264".to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "desktop".to_owned(),
        ));

        let track_clone = Arc::clone(&track);
        self.peer_connection
            .add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        self.video_track = Some(track);
        Ok(track_clone)
    }

    // 创建数据通道（用于控制信息）
    pub async fn create_data_channel(&mut self, label: &str) -> Result<Arc<RTCDataChannel>, Error> {
        let data_channel = self
            .peer_connection
            .create_data_channel(label, None)
            .await?;
        let data_channel = Arc::new(data_channel);
        self.data_channel = Some(Arc::clone(&data_channel));
        Ok(data_channel)
    }

    // 发送视频帧
    pub async fn send_video_frame(&self, frame: Bytes) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(track) = &self.video_track {
            track
                .write_sample(&webrtc::media::Sample {
                    data: frame,
                    duration: std::time::Duration::from_millis(16),
                    ..Default::default()
                })
                .await?;
        }
        Ok(())
    }

    // 发送控制数据
    pub async fn send_control_data(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dc) = &self.data_channel {
            dc.send(Bytes::copy_from_slice(data)).await?;
        }
        Ok(())
    }

    // 创建连接请求（发起方）
    pub async fn create_offer(&mut self) -> Result<ConnectionInfo, Box<dyn std::error::Error>> {
        self.create_video_track().await?;
        self.create_data_channel("control").await?;

        let offer = self.peer_connection.create_offer(None).await?;
        self.peer_connection
            .set_local_description(offer.clone())
            .await?;

        let ice_candidates = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let ice_candidates_clone = ice_candidates.clone();

        self.peer_connection.on_ice_candidate(Box::new(move |c| {
            if let Some(candidate) = c {
                ice_candidates_clone.lock().push(candidate.to_string());
            }
            Box::pin(async {})
        }));

        // 先获取 ice candidates
        let candidates = ice_candidates.lock().clone();

        Ok(ConnectionInfo {
            offer: offer.sdp,
            ice_candidates: candidates,
        })
    }

    // 接受连接（接收方）
    pub async fn accept_offer(
        &mut self,
        info: ConnectionInfo,
    ) -> Result<ConnectionInfo, Box<dyn std::error::Error>> {
        // 设置数据通道处理
        let pc = Arc::clone(&self.peer_connection);
        self.peer_connection.on_data_channel(Box::new(move |dc| {
            println!("New DataChannel: {}", dc.label());
            Box::pin(async {})
        }));

        // 设置视频轨道处理
        self.peer_connection.on_track(Box::new(move |track, _, _| {
            println!("New Track: {}", track.kind());
            Box::pin(async {})
        }));

        self.peer_connection
            .set_remote_description(info.offer)
            .await?;

        for candidate in info.ice_candidates {
            self.peer_connection.add_ice_candidate(candidate).await?;
        }

        let answer = self.peer_connection.create_answer(None).await?;
        self.peer_connection
            .set_local_description(answer.clone())
            .await?;

        let mut ice_candidates = Vec::new();
        self.peer_connection.on_ice_candidate(Box::new(move |c| {
            if let Some(candidate) = c {
                ice_candidates.push(candidate.to_string());
            }
            Box::pin(async {})
        }));

        Ok(ConnectionInfo {
            offer: answer.sdp,
            ice_candidates,
        })
    }

    // 完成连接（发起方）
    pub fn complete_connection(
        &self,
        answer_info: ConnectionInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        futures::executor::block_on(async {
            self.peer_connection
                .set_remote_description(RTCSessionDescription::new(
                    answer_info.offer,
                    RTCSdpType::Answer,
                ))
                .await?;

            for candidate in answer_info.ice_candidates {
                self.peer_connection.add_ice_candidate(candidate).await?;
            }
            Ok(())
        })
    }

    // 监听连接状态
    pub fn on_connection_state_change(&self) {
        let pc = Arc::clone(&self.peer_connection);
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Connection State has changed: {}", s);
            Box::pin(async {})
        }));
    }
}
