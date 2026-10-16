//! OS input only. Focus, repetition and UI actions stay in the shared frontend adapter.
use gilrs::{Axis, Button, EventType, GilrsBuilder};
use serde::Serialize;
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};

#[derive(Clone, Serialize)]
pub struct Device { id: String, name: String }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame { id: String, at: u64, buttons: Vec<&'static str>, left_x: f32, left_y: f32, right_y: f32 }
#[derive(Default, Clone, Serialize)]
pub struct Batch { devices: Vec<Device>, frames: Vec<Frame>, states: Vec<Frame>, overflow: bool, suspended: bool, error: Option<String> }
#[derive(Default)]
struct Buffer { batch: Batch }
impl Buffer {
    fn push(&mut self, frame: Frame) {
        if self.batch.frames.len() >= 256 { self.batch.frames.clear(); self.batch.overflow = true; }
        self.batch.frames.push(frame);
    }
    fn drain(&mut self, suspended: bool, now: u64) -> Batch {
        let mut batch = self.batch.clone();
        self.batch.frames.clear(); self.batch.overflow = false;
        batch.suspended = suspended;
        if batch.frames.iter().any(|f| now.saturating_sub(f.at) > 250) { batch.overflow = true; }
        if suspended || batch.overflow { batch.frames.clear(); }
        batch
    }
}
pub struct NativeGamepads { shared: Arc<Mutex<Buffer>>, stop: Arc<AtomicBool>, start: Instant }
impl Drop for NativeGamepads { fn drop(&mut self) { self.stop.store(true, Ordering::Relaxed); } }
fn snapshot(gamepad: &gilrs::Gamepad<'_>, id: String, at: u64) -> Frame {
    let buttons = [
        (Button::DPadUp,"Up"), (Button::DPadDown,"Down"), (Button::DPadLeft,"Left"), (Button::DPadRight,"Right"),
        (Button::South,"South"), (Button::East,"East"), (Button::LeftTrigger,"LB"), (Button::RightTrigger,"RB"), (Button::Start,"Menu"),
    ].into_iter().filter_map(|(button, name)| gamepad.is_pressed(button).then_some(name)).collect();
    // Gilrs Y is positive upwards; our UI coordinate system is positive downwards.
    Frame { id, at, buttons, left_x: gamepad.value(Axis::LeftStickX), left_y: -gamepad.value(Axis::LeftStickY), right_y: -gamepad.value(Axis::RightStickY) }
}
impl NativeGamepads {
    pub fn start() -> Self {
        let shared = Arc::new(Mutex::new(Buffer::default())); let stop = Arc::new(AtomicBool::new(false)); let start = Instant::now();
        let buffer = shared.clone(); let ending = stop.clone();
        std::thread::spawn(move || {
            let mut gilrs = match GilrsBuilder::new().add_env_mappings(false).build() {
                Ok(g) => g,
                Err(e) => { if let Ok(mut b) = buffer.lock() { b.batch.error = Some(format!("Native controller initialization failed: {e}")); } return; }
            };
            while !ending.load(Ordering::Relaxed) {
                let mut edges = vec![];
                let mut connection_changed = false;
                while let Some(event) = gilrs.next_event() {
                    if matches!(event.event, EventType::Connected | EventType::Disconnected) { connection_changed = true; }
                    let gamepad = gilrs.gamepad(event.id);
                    if gamepad.is_connected() { edges.push(snapshot(&gamepad, usize::from(event.id).to_string(), start.elapsed().as_millis() as u64)); }
                    if edges.len() >= 256 { break; }
                }
                if let Ok(mut b) = buffer.lock() {
                    // Re-arm even if disconnect/reconnect happened between two frontend polls.
                    if connection_changed { b.batch.overflow = true; }
                    for frame in edges { b.push(frame); }
                    let now = start.elapsed().as_millis() as u64;
                    b.batch.devices = gilrs.gamepads().map(|(id,g)| Device{id:usize::from(id).to_string(),name:g.name().into()}).collect();
                    b.batch.states = gilrs.gamepads().map(|(id,g)| snapshot(&g,usize::from(id).to_string(),now)).collect();
                }
                gilrs.inc();
                std::thread::sleep(Duration::from_millis(8));
            }
        });
        Self {shared, stop, start}
    }
}
#[tauri::command]
pub fn poll_gamepads(window: tauri::WebviewWindow, gamepads: tauri::State<'_, NativeGamepads>) -> Result<Batch, String> {
    let suspended = !window.is_focused().map_err(|e| e.to_string())?;
    Ok(gamepads.shared.lock().map_err(|_| "Controller state is unavailable")?.drain(suspended, gamepads.start.elapsed().as_millis() as u64))
}
#[cfg(test)] mod tests {
    use super::*;
    fn frame(at:u64)->Frame {Frame{id:"0".into(),at,buttons:vec!["South"],left_x:0.,left_y:0.,right_y:0.}}
    #[test] fn queue_preserves_taps_and_discards_stale_or_background_edges() {
        let mut b=Buffer::default();b.push(frame(1));let mut released=frame(2);released.buttons.clear();b.push(released);
        let batch=b.drain(false,3);assert_eq!(batch.frames.len(),2);assert!(batch.frames[1].buttons.is_empty());assert!(b.drain(false,4).frames.is_empty());
        b.push(frame(1));assert!(b.drain(false,1000).overflow);
        b.push(frame(1001));assert!(b.drain(true,1002).frames.is_empty());assert!(b.drain(false,1003).frames.is_empty());
    }
    #[test] #[ignore = "requires host input devices; run explicitly for hardware diagnostics"]
    fn hardware_inventory() {
        let gilrs = GilrsBuilder::new().add_env_mappings(false).build().expect("native input initialization");
        let devices: Vec<_> = gilrs.gamepads().map(|(id,g)| (usize::from(id),g.name().to_owned())).collect();
        println!("Native gamepads: {devices:?}");
    }
    #[test] fn queue_overflow_cannot_replay_confirmations() {let mut b=Buffer::default();for _ in 0..300 {b.push(frame(0));}let batch=b.drain(false,1);assert!(batch.overflow);assert!(batch.frames.is_empty());}
}
