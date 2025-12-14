use wasm_bindgen::prelude::*;
use lopdf::{Document, Object};

// Browser console logging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Initialize panic hook for better WASM error messages
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Compression levels for PDF optimization
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    Light = 0,
    Medium = 1,
    High = 2,
}

/// Convert string to compression level
#[wasm_bindgen]
pub fn compression_level_from_string(level: &str) -> Result<CompressionLevel, JsValue> {
    match level.to_lowercase().as_str() {
        "light" => Ok(CompressionLevel::Light),
        "medium" => Ok(CompressionLevel::Medium),
        "high" => Ok(CompressionLevel::High),
        _ => Err(JsValue::from_str("Invalid compression level. Use 'light', 'medium', or 'high'.")),
    }
}

/// Get PDF info without compression
#[wasm_bindgen]
pub fn get_pdf_info(pdf_data: &[u8]) -> Result<JsValue, JsValue> {
    let doc = Document::load_mem(pdf_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {}", e)))?;

    let pages = doc.get_pages();
    let total_objects = doc.objects.len();

    let mut image_count = 0;
    let mut jpeg_images = 0;

    for (_id, obj) in doc.objects.iter() {
        if let Object::Stream(stream) = obj {
            let is_image = stream
                .dict
                .get(b"Subtype")
                .ok()
                .and_then(|v| v.as_name().ok())
                .map(|n| n == b"Image")
                .unwrap_or(false);

            if is_image {
                image_count += 1;
                if let Ok(filter) = stream.dict.get(b"Filter") {
                    if let Ok(name) = filter.as_name() {
                        if name == b"DCTDecode" || name == b"DCT" {
                            jpeg_images += 1;
                        }
                    }
                }
            }
        }
    }

    let info = PdfInfo {
        page_count: pages.len(),
        total_size: pdf_data.len(),
        total_objects,
        image_count,
        jpeg_images,
    };

    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Extract JPEG images from PDF for parallel worker compression
#[wasm_bindgen]
pub fn extract_images(pdf_data: &[u8]) -> Result<JsValue, JsValue> {
    let doc = Document::load_mem(pdf_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {}", e)))?;

    let mut images = Vec::new();

    console_log!("🔍 Scanning PDF for images...");

    for (obj_id, obj) in doc.objects.iter() {
        if let Object::Stream(stream) = obj {
            let is_image = stream
                .dict
                .get(b"Subtype")
                .ok()
                .and_then(|v| v.as_name().ok())
                .map(|n| n == b"Image")
                .unwrap_or(false);

            if !is_image {
                continue;
            }

            let is_jpeg = stream
                .dict
                .get(b"Filter")
                .ok()
                .and_then(|v| v.as_name().ok())
                .map(|f| f == b"DCTDecode" || f == b"DCT")
                .unwrap_or(false);

            if !is_jpeg {
                continue;
            }

            let width = stream.dict.get(b"Width")
                .ok()
                .and_then(|v| v.as_i64().ok())
                .unwrap_or(0);

            let height = stream.dict.get(b"Height")
                .ok()
                .and_then(|v| v.as_i64().ok())
                .unwrap_or(0);

            images.push(ImageInfo {
                object_id: format!("{}-{}", obj_id.0, obj_id.1),
                data: stream.content.clone(),
                width: width as u32,
                height: height as u32,
            });

            console_log!("  ✓ Found image at {:?}: {}x{} ({} KB)", 
                obj_id, width, height, stream.content.len() / 1024);
        }
    }

    console_log!("✓ Extracted {} images", images.len());

    serde_wasm_bindgen::to_value(&images)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Reinject compressed images back into PDF preserving ALL metadata and references
#[wasm_bindgen]
pub fn reinject_images(pdf_data: &[u8], compressed_images: JsValue) -> Result<Vec<u8>, JsValue> {
    let images: Vec<CompressedImage> = serde_wasm_bindgen::from_value(compressed_images)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize images: {}", e)))?;

    console_log!("🔄 Reinjecting {} compressed images", images.len());

    let mut doc = Document::load_mem(pdf_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {}", e)))?;

    let mut updated_count = 0;

    for compressed in &images {
        let parts: Vec<&str> = compressed.object_id.split('-').collect();
        if parts.len() != 2 { 
            console_log!("⚠️ Invalid object_id format: {}", compressed.object_id);
            continue; 
        }

        let id = match parts[0].parse::<u32>() { 
            Ok(id) => id, 
            Err(_) => {
                console_log!("⚠️ Invalid object ID: {}", parts[0]);
                continue;
            }
        };
        
        let gen_num = match parts[1].parse::<u16>() { 
            Ok(g) => g, 
            Err(_) => {
                console_log!("⚠️ Invalid generation number: {}", parts[1]);
                continue;
            }
        };
        
        let obj_id = (id, gen_num);

        if let Ok(obj) = doc.get_object_mut(obj_id) {
            if let Object::Stream(stream) = obj {
                // CRITICAL: Backup ALL original metadata before modification
                let original_colorspace = stream.dict.get(b"ColorSpace").ok().cloned();
                let original_bpc = stream.dict.get(b"BitsPerComponent").ok().cloned();
                let original_filter = stream.dict.get(b"Filter").ok().cloned();
                let original_decode_parms = stream.dict.get(b"DecodeParms").ok().cloned();
                let original_smask = stream.dict.get(b"SMask").ok().cloned();
                let original_mask = stream.dict.get(b"Mask").ok().cloned();
                let original_intent = stream.dict.get(b"Intent").ok().cloned();
                let original_interpolate = stream.dict.get(b"Interpolate").ok().cloned();
                let original_decode = stream.dict.get(b"Decode").ok().cloned();
                let original_image_mask = stream.dict.get(b"ImageMask").ok().cloned();
                let original_struct_parent = stream.dict.get(b"StructParent").ok().cloned();
                let original_id = stream.dict.get(b"ID").ok().cloned();
                let original_oc = stream.dict.get(b"OC").ok().cloned();
                let original_metadata = stream.dict.get(b"Metadata").ok().cloned();

                // Update image content and dimensions
                let new_len = compressed.data.len();
                //stream.content = compressed.data.clone();
                stream.set_content(compressed.data.clone());
                stream.dict.set("Width", Object::Integer(compressed.width as i64));
                stream.dict.set("Height", Object::Integer(compressed.height as i64));
                stream.dict.set("Length", Object::Integer(new_len as i64));

                // Restore Filter (should always be DCTDecode for JPEG)
                if let Some(filter) = original_filter {
                    stream.dict.set("Filter", filter);
                } else {
                    stream.dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                }

                // Restore ALL original metadata to preserve PDF structure
                if let Some(cs) = original_colorspace {
                    stream.dict.set("ColorSpace", cs);
                }
                if let Some(bpc) = original_bpc {
                    stream.dict.set("BitsPerComponent", bpc);
                }
                if let Some(dp) = original_decode_parms {
                    stream.dict.set("DecodeParms", dp);
                }
                if let Some(sm) = original_smask {
                    stream.dict.set("SMask", sm);
                }
                if let Some(m) = original_mask {
                    stream.dict.set("Mask", m);
                }
                if let Some(i) = original_intent {
                    stream.dict.set("Intent", i);
                }
                if let Some(interp) = original_interpolate {
                    stream.dict.set("Interpolate", interp);
                }
                if let Some(dec) = original_decode {
                    stream.dict.set("Decode", dec);
                }
                if let Some(im) = original_image_mask {
                    stream.dict.set("ImageMask", im);
                }
                if let Some(sp) = original_struct_parent {
                    stream.dict.set("StructParent", sp);
                }
                if let Some(id_obj) = original_id {
                    stream.dict.set("ID", id_obj);
                }
                if let Some(oc) = original_oc {
                    stream.dict.set("OC", oc);
                }
                if let Some(meta) = original_metadata {
                    stream.dict.set("Metadata", meta);
                }

                updated_count += 1;
                console_log!("  ✓ Updated {:?}: {} bytes", obj_id, new_len);
            } else {
                console_log!("  ⚠️ Object {:?} is not a stream", obj_id);
            }
        } else {
            console_log!("  ❌ Object {:?} not found in document", obj_id);
        }
    }

    console_log!("✓ Successfully reinjected {}/{} images", updated_count, images.len());

    // Save document WITHOUT calling compress() to preserve all references
    let mut output = Vec::new();
    doc.save_to(&mut output)
        .map_err(|e| JsValue::from_str(&format!("Failed to save PDF: {}", e)))?;

    console_log!("✓ PDF saved: {} bytes", output.len());
    Ok(output)
}

#[derive(serde::Serialize)]
struct PdfInfo {
    page_count: usize,
    total_size: usize,
    total_objects: usize,
    image_count: usize,
    jpeg_images: usize,
}

#[derive(serde::Serialize)]
struct ImageInfo {
    object_id: String,
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(serde::Deserialize)]
struct CompressedImage {
    object_id: String,
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
