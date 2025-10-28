use crate::environment::UploadMediaExt;
use crate::environment::model::Instance;
use crate::view_model::AttachmentMedia;
use megalodon::entities::UploadMedia;

pub fn validate_text(_instance: Option<Instance>, text: &str) -> (bool, u32, u32) {
    // Q3: MVP simplified validation - hardcoded 500 char limit for agent chat
    let current = text.chars().count() as u32;
    let max_chars = 5000; // Generous limit for agent conversations

    if current > max_chars {
        (false, current, max_chars)
    } else {
        (true, current, max_chars)
    }
}

pub fn handle_image_upload(
    image: AttachmentMedia,
    result: Result<UploadMedia, String>,
    media: &mut Vec<AttachmentMedia>,
) -> Option<String> {
    match result {
        Ok(m) => {
            for n in media.iter_mut() {
                if n.path == image.path {
                    n.server_id = Some(m.id().to_string());
                    n.is_uploaded = true;
                    break;
                }
            }
            None
        }
        Err(error) => {
            let index = media.iter().position(|s| s.path == image.path)?;
            media.remove(index);
            Some(error)
        }
    }
}
