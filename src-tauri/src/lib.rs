// Question Desk — scaffold only.
//
// Build your Tauri commands here (M2: save_question, list_questions,
// delete_question — M4: draft_answer). Register each one in the
// invoke_handler below as you add it. See:
// https://tauri.app/develop/calling-rust/

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
