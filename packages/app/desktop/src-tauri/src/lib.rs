mod vst3;

use std::sync::Mutex;
use vst3::Vst3Service;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Vst3Service::default()))
        .invoke_handler(tauri::generate_handler![
            vst3::vst3_scan,
            vst3::vst3_load,
            vst3::vst3_unload,
            vst3::vst3_set_parameter,
            vst3::vst3_save_state,
            vst3::vst3_load_state,
            vst3::vst3_send_midi,
            vst3::vst3_set_transport,
            vst3::vst3_process_block,
        ])
        .run(tauri::generate_context!())
        .expect("error while running openDAW desktop");
}
