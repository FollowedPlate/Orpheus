mod commands;
mod models;
mod parsers;

use commands::{files, library, llm, settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            files::parse_book,
            library::load_library,
            library::save_library,
            library::update_progress,
            library::add_book_to_library,
            library::remove_book,
            library::start_session,
            library::update_book_metadata,
            library::update_book_file_path,
            settings::load_settings,
            settings::save_settings,
            llm::generate_questions,
            llm::generate_book_metadata,
            llm::evaluate_answer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
