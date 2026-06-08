/* TEMPORARY STUB — replace with real rodio when libasound2-dev is installed.
 * This lets the Rust code compile and type-check on machines without ALSA.
 */

pub struct Decoder;
#[derive(Clone)]
pub struct MixerDeviceSink;
impl MixerDeviceSink {
    pub fn mixer(&self) -> Mixer { Mixer }
}
pub struct Mixer;

#[derive(Clone, Copy)]
pub struct Player;
impl Player {
    pub fn connect_new(_mixer: &Mixer) -> Self { Player }
    pub fn pause(&self) {}
    pub fn play(&self) {}
    pub fn stop(&self) {}
    pub fn skip_one(&self) {}
    pub fn try_seek(&self, _dur: std::time::Duration) {}
    pub fn get_pos(&self) -> std::time::Duration { std::time::Duration::from_secs(0) }
    pub fn len(&self) -> std::time::Duration { std::time::Duration::from_secs(0) }
}

pub struct DeviceSinkBuilder;
impl DeviceSinkBuilder {
    pub fn open_default_sink() -> Result<MixerDeviceSink, Box<dyn std::error::Error>> {
        Ok(MixerDeviceSink)
    }
}

pub mod source {
    pub trait Source {}
}
