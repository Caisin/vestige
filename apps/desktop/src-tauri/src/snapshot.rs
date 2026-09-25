//! Capture only our own WKWebView on an explicit local CLI request. No screen
//! recording permission, browser IPC command or access to other windows.
use block2::RcBlock;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSImage};
use objc2_foundation::{NSDictionary, NSError};
use objc2_web_kit::WKWebView;
use std::path::PathBuf;

pub fn capture(window: &tauri::WebviewWindow, output: PathBuf) {
    if output.extension().and_then(|part| part.to_str()) != Some("png") {
        eprintln!("Snapshot output must end in .png");
        return;
    }
    let _ = window.with_webview(move |view| {
        // Tauri provides its live WKWebView pointer on the main thread. The
        // framework retains the completion block until the snapshot finishes.
        let webview = unsafe { &*(view.inner() as *const WKWebView) };
        let completion = RcBlock::new(move |image: *mut NSImage, error: *mut NSError| {
            if !error.is_null() || image.is_null() {
                eprintln!("WKWebView snapshot failed");
                return;
            }
            let image = unsafe { &*image };
            if let Some(tiff) = image.TIFFRepresentation()
                && let Some(bitmap) = NSBitmapImageRep::imageRepWithData(&tiff)
                && let Some(png) = unsafe {
                    bitmap.representationUsingType_properties(
                        NSBitmapImageFileType::PNG,
                        &NSDictionary::new(),
                    )
                }
            {
                if let Err(error) = std::fs::write(&output, png.to_vec()) {
                    eprintln!("Snapshot write failed: {error}");
                }
            }
        });
        unsafe {
            webview.takeSnapshotWithConfiguration_completionHandler(None, &completion);
        }
    });
}
