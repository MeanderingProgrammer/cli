use std::path::PathBuf;

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

pub fn dir(title: &str) -> PathBuf {
    FileDialog::new()
        .set_title(title)
        .set_directory("/")
        .pick_folder()
        .unwrap_or_else(|| panic!("must select directory for: {title}"))
}

pub fn cancel(description: String) -> bool {
    let result = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("confirm")
        .set_buttons(MessageButtons::YesNo)
        .set_description(description)
        .show();
    !matches!(result, MessageDialogResult::Yes)
}
