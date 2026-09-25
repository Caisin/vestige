//! Bounded, local-only screenplay text extraction. Original bytes stay in SQLite.
use std::io::{Cursor, Read};

use quick_xml::{Reader, events::Event};
use serde::Serialize;

use super::{WriterError, WriterResult};

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct Segment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct ParsedSource {
    pub format: String,
    pub text: String,
    pub segments: Vec<Segment>,
    pub metadata: serde_json::Value,
}

pub fn parse(filename: &str, bytes: &[u8]) -> WriterResult<ParsedSource> {
    if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
        return Err(WriterError::new(
            "UPLOAD_LIMIT",
            "文件为空或超过 50 MiB 上传限制",
        ));
    }
    let format = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let text = match format.as_str() {
        "txt" | "md" | "fountain" => std::str::from_utf8(bytes)
            .map_err(|_| WriterError::new("TEXT_ENCODING", "请将文件转换为 UTF-8 编码后重试"))?
            .trim_start_matches('\u{feff}')
            .to_string(),
        "fdx" => xml_text(bytes, false)?,
        "docx" => {
            let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
                .map_err(|_| WriterError::new("INVALID_DOCX", "无法读取 DOCX 压缩包"))?;
            if archive.len() > 10000 {
                return Err(WriterError::new("PARSE_LIMIT", "DOCX 包含过多文件"));
            }
            let mut total = 0_u64;
            for index in 0..archive.len() {
                let file = archive
                    .by_index(index)
                    .map_err(|_| WriterError::new("INVALID_DOCX", "DOCX 文件目录损坏"))?;
                total = total.saturating_add(file.size());
                if total > 100 * 1024 * 1024 {
                    return Err(WriterError::new("PARSE_LIMIT", "DOCX 解压总量超过 100 MiB"));
                }
            }
            let document = archive
                .by_name("word/document.xml")
                .map_err(|_| WriterError::new("INVALID_DOCX", "DOCX 缺少正文文件"))?;
            let mut xml = Vec::new();
            document
                .take((MAX_TEXT_BYTES + 1) as u64)
                .read_to_end(&mut xml)
                .map_err(|_| WriterError::new("INVALID_DOCX", "无法读取 DOCX 正文"))?;
            if xml.len() > MAX_TEXT_BYTES {
                return Err(WriterError::new("PARSE_LIMIT", "DOCX 正文超出解析限制"));
            }
            xml_text(&xml, true)?
        }
        "pdf" => {
            if !bytes.starts_with(b"%PDF-") {
                return Err(WriterError::new("INVALID_PDF", "文件不是有效 PDF"));
            }
            let document = lopdf::Document::load_mem(bytes)
                .map_err(|_| WriterError::new("INVALID_PDF", "PDF 已损坏、加密或无法解析"))?;
            let pages: Vec<u32> = document.get_pages().keys().copied().collect();
            if pages.len() > 2000 {
                return Err(WriterError::new(
                    "PARSE_LIMIT",
                    "PDF 超过 2000 页，请分卷上传",
                ));
            }
            let mut result = String::new();
            for page in pages {
                let extracted = document.extract_text(&[page]).map_err(|_| {
                    WriterError::new("INVALID_PDF", "PDF 文字提取失败，请导出文本后上传")
                })?;
                if result.len() + extracted.len() > MAX_TEXT_BYTES {
                    return Err(WriterError::new(
                        "PARSE_LIMIT",
                        "提取文本超过 8 MiB，请分卷上传",
                    ));
                }
                result.push_str(&extracted);
                result.push('\n');
            }
            result
        }
        _ => {
            return Err(WriterError::new(
                "UNSUPPORTED_FORMAT",
                "支持 TXT、Markdown、Fountain、FDX、DOCX 和文本 PDF",
            ));
        }
    };
    let text = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(WriterError::new(
            "NO_TEXT",
            "没有可提取的文字；扫描 PDF 请先 OCR 后重新上传",
        ));
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err(WriterError::new(
            "PARSE_LIMIT",
            "正文超过 8 MiB，请分卷上传",
        ));
    }
    let segments = segment(&text);
    Ok(ParsedSource {
        format,
        text,
        segments,
        metadata: serde_json::json!({"parser_version":"writer-import-v1","normalization":"unicode_line_endings"}),
    })
}

pub fn attribution(parsed: &mut ParsedSource, input: &serde_json::Value) -> WriterResult<()> {
    let fields = input
        .as_object()
        .ok_or_else(|| WriterError::new("INVALID_METADATA", "作品元数据必须是对象"))?;
    for (key, value) in fields {
        if !["author", "episode"].contains(&key.as_str()) {
            return Err(WriterError::new(
                "INVALID_METADATA",
                format!("不支持的作品元数据：{key}"),
            ));
        }
        let text = value
            .as_str()
            .filter(|text| text.len() <= 300)
            .ok_or_else(|| {
                WriterError::new(
                    "INVALID_METADATA",
                    "作者和篇章信息必须是最多 300 字节的文本",
                )
            })?;
        if !text.trim().is_empty() {
            parsed.metadata[key] = serde_json::json!(text.trim());
        }
    }
    parsed.metadata["attribution"] = serde_json::json!("user_supplied");
    Ok(())
}

fn xml_text(bytes: &[u8], docx: bool) -> WriterResult<String> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().check_end_names = true;
    let mut depth = 0usize;
    let mut in_text = false;
    let mut output = String::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|_| WriterError::new("INVALID_XML", "剧本文档 XML 格式错误"))?;
        match event {
            Event::DocType(_) => {
                return Err(WriterError::new(
                    "UNSAFE_XML",
                    "不接受含外部实体或 DTD 的文档",
                ));
            }
            Event::Start(tag) => {
                depth += 1;
                if depth > 128 {
                    return Err(WriterError::new("PARSE_LIMIT", "XML 嵌套过深"));
                }
                in_text = if docx {
                    tag.local_name().as_ref() == b"t"
                } else {
                    tag.local_name().as_ref() == b"Text"
                };
            }
            Event::End(tag) => {
                depth = depth.saturating_sub(1);
                in_text = false;
                if tag.local_name().as_ref() == b"p" || tag.local_name().as_ref() == b"Paragraph" {
                    output.push('\n');
                }
            }
            Event::Text(text) if in_text => {
                let decoded = text
                    .decode()
                    .map_err(|_| WriterError::new("TEXT_ENCODING", "XML 文字编码无法读取"))?;
                let decoded = quick_xml::escape::unescape(&decoded)
                    .map_err(|_| WriterError::new("INVALID_XML", "XML 文本转义错误"))?;
                output.push_str(&decoded);
            }
            Event::GeneralRef(reference) if in_text => {
                let entity = format!("&{};", String::from_utf8_lossy(reference.as_ref()));
                output.push_str(
                    &quick_xml::escape::unescape(&entity)
                        .map_err(|_| WriterError::new("INVALID_XML", "不支持的 XML 实体"))?,
                );
            }
            Event::CData(text) if in_text => {
                output.push_str(
                    &text
                        .decode()
                        .map_err(|_| WriterError::new("TEXT_ENCODING", "XML 文字编码无法读取"))?,
                );
            }
            Event::Eof => break,
            _ => {}
        }
        if output.len() > MAX_TEXT_BYTES {
            return Err(WriterError::new("PARSE_LIMIT", "提取文字超出限制"));
        }
    }
    Ok(output)
}

fn segment(text: &str) -> Vec<Segment> {
    let mut result = Vec::new();
    let (mut start, mut start_line, mut line, mut length) = (0, 1, 1, 0);
    for (offset, character) in text.char_indices() {
        length += 1;
        let end_line = line;
        if character == '\n' {
            line += 1;
        }
        if length >= 2200 || (length >= 1600 && character == '\n') {
            let end = offset + character.len_utf8();
            result.push(Segment {
                text: text[start..end].to_string(),
                start_line,
                end_line,
            });
            start = end;
            start_line = line;
            length = 0;
        }
    }
    if start < text.len() {
        result.push(Segment {
            text: text[start..].to_string(),
            start_line,
            end_line: line,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_segments_preserve_script_and_lines() {
        let original = format!("第一场 内景\n{}\n结尾", "林夏：我要知道真相。".repeat(500));
        let parsed = parse("故事.fountain", original.as_bytes()).unwrap();
        assert!(parsed.segments.len() > 1);
        assert!(
            parsed
                .segments
                .iter()
                .all(|part| part.text.chars().count() <= 2201)
        );
        assert!(parsed.text.contains("结尾"));
        assert_eq!(
            parsed
                .segments
                .iter()
                .map(|part| part.text.as_str())
                .collect::<String>(),
            parsed.text,
            "segment boundaries must not insert characters into source evidence"
        );
        assert_eq!(
            parse("bad.txt", &[0xff]).err().unwrap().code,
            "TEXT_ENCODING"
        );
    }
    #[test]
    fn fdx_extracts_dialogue_and_rejects_dtd() {
        let parsed = parse("scene.fdx", br#"<FinalDraft><Content><Paragraph><Text>Hello &amp; goodbye.</Text></Paragraph></Content></FinalDraft>"#).unwrap();
        assert_eq!(parsed.text, "Hello & goodbye.");
        assert_eq!(
            parse("scene.fdx", br#"<!DOCTYPE x SYSTEM "file:///fiction"><x/>"#)
                .err()
                .unwrap()
                .code,
            "UNSAFE_XML"
        );
    }
    #[test]
    fn unknown_and_empty_uploads_are_explicit_errors() {
        assert_eq!(
            parse("scene.exe", b"hello").err().unwrap().code,
            "UNSUPPORTED_FORMAT"
        );
        assert_eq!(parse("scene.txt", b"   ").err().unwrap().code, "NO_TEXT");
    }
}
