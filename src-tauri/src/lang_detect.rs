use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use once_cell::sync::Lazy;

// 只保留中英两种语言，避免 lingua 把 21 种语言的模型数据全部编入二进制
// （之前 21 种语言让 .rodata 数据段高达 ~89MB，是包体积的主要来源）
static DETECTOR: Lazy<LanguageDetector> = Lazy::new(|| {
    LanguageDetectorBuilder::from_languages(&[Language::Chinese, Language::English]).build()
});

pub fn init_lang_detect() {
    // 预热：触发 detector 构建一次，避免首次查词时卡顿
    let _ = DETECTOR.detect_language_of("Hello Language");
}

#[tauri::command]
pub fn lang_detect(text: &str) -> Result<&str, ()> {
    match DETECTOR.detect_language_of(text) {
        Some(Language::Chinese) => Ok("zh_cn"),
        // 英语及未识别均回退英文
        _ => Ok("en"),
    }
}
