use serde::Deserialize;
use serde::Serialize;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialOrd, PartialEq)]
pub enum TgMsgFormat {
    MarkdownV2,
    HTML,
    Markdown,
    #[serde(other)]
    PlainText,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct TgMessage {
    pub text: String,
    pub format: TgMsgFormat,
    pub show_notification: bool,
}

impl TgMessage {
    pub fn build_text(text: String) -> Self {
        TgMessage {
            text,
            format: TgMsgFormat::PlainText,
            show_notification: true,
        }
    }
}
