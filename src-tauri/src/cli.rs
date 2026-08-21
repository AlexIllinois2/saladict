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
    Help,
}

const HELP: &str = r#"Saladict - 划词翻译 / OCR 工具

Usage:
  saladict [OPTIONS]

Options:
  -s, --selection-translate   翻译选中的文本 (translate selected text)
  -i, --input-translate       打开输入翻译窗口 (open the input translate window)
      --translate             同 --input-translate (alias of --input-translate)
  -r, --ocr-recognize         截图识别文本 (screenshot text recognition)
  -t, --ocr-translate         截图识别并翻译 (screenshot OCR and translate)
  -c, --config                打开偏好设置 (open preferences)
      --settings              同 --config (alias of --config)
  -h, --help                  显示帮助信息 (print help)

不带参数启动时默认打开输入翻译窗口。
Without options the input translate window is opened."#;

pub fn print_help() {
    println!("{HELP}");
}

pub fn parse_from(args: &[String]) -> CliAction {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return CliAction::Help;
    }
    if args.iter().any(|a| a == "--selection-translate" || a == "-s") {
        return CliAction::SelectionTranslate;
    }
    if args.iter().any(|a| a == "--input-translate" || a == "-i" || a == "--translate") {
        return CliAction::InputTranslate;
    }
    if args.iter().any(|a| a == "--ocr-recognize" || a == "-r") {
        return CliAction::OcrRecognize;
    }
    if args.iter().any(|a| a == "--ocr-translate" || a == "-t") {
        return CliAction::OcrTranslate;
    }
    if args.iter().any(|a| a == "--config" || a == "-c" || a == "--settings") {
        return CliAction::Config;
    }
    // Default action: open the input translate window, so clicking the
    // desktop launcher icon brings up the translate popup.
    CliAction::InputTranslate
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
            CliAction::Help => print_help(),
        }
    });
}
