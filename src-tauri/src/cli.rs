use crate::window::{config_window, input_translate, ocr_recognize, ocr_translate, selection_translate};
use log::info;

// Actions that can be triggered from the command line. On GNOME Wayland the
// in-app global shortcuts are unavailable, so users can bind a system keyboard
// shortcut to `saladict --selection-translate` (or the other flags below).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliAction {
    SelectionTranslate,
    InputTranslate,
    OcrRecognize,
    OcrTranslate,
    Config,
    None,
}

pub fn parse_from(args: &[String]) -> CliAction {
    if args.iter().any(|a| a == "--selection-translate" || a == "-s") {
        return CliAction::SelectionTranslate;
    }
    if args.iter().any(|a| a == "--input-translate" || a == "-i") {
        return CliAction::InputTranslate;
    }
    if args.iter().any(|a| a == "--ocr-recognize" || a == "-r") {
        return CliAction::OcrRecognize;
    }
    if args.iter().any(|a| a == "--ocr-translate" || a == "-t") {
        return CliAction::OcrTranslate;
    }
    if args.iter().any(|a| a == "--config" || a == "-c") {
        return CliAction::Config;
    }
    CliAction::None
}

// Execute a CLI action. Runs in a background thread so it never blocks the
// event loop; a short delay lets a freshly started app initialize its windows.
pub fn run(action: CliAction) {
    info!("Execute CLI action: {:?}", action);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        match action {
            CliAction::SelectionTranslate => selection_translate(),
            CliAction::InputTranslate => input_translate(),
            CliAction::OcrRecognize => ocr_recognize(),
            CliAction::OcrTranslate => ocr_translate(),
            CliAction::Config => config_window(),
            CliAction::None => {}
        }
    });
}
