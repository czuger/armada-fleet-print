//! Serialization of a [`DocumentPlan`] into PDF bytes.

use std::collections::{BTreeSet, HashMap};

use miniz_oxide::deflate::{CompressionLevel, compress_to_vec_zlib};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, Str, TextStr};

use crate::card_image::CardImage;
use crate::card_name::CardName;
use crate::layout::{DocumentPlan, Element, PAGE_HEIGHT, PAGE_WIDTH, Page, TextAlign};
use crate::pdf::helvetica;

const FONT_NAME: Name<'static> = Name(b"F1");

/// Control-point distance of a cubic Bézier approximating a unit quarter circle.
const KAPPA: f32 = 0.552_284_8;

/// Error returned when the plan uses a card whose image was not provided.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// No decoded image was given for this card.
    #[error("no decoded image for card {0}")]
    MissingImage(CardName),
}

/// Renders `plan` as a PDF document.
///
/// # Arguments
/// * `plan` - Pages to draw.
/// * `images` - Decoded image of every card the plan uses; each is embedded once.
/// * `title` - Document title shown by PDF viewers.
///
/// # Errors
/// Returns [`RenderError::MissingImage`] if a card of the plan is missing from `images`.
pub fn render(
    plan: &DocumentPlan,
    images: &HashMap<CardName, CardImage>,
    title: &str,
) -> Result<Vec<u8>, RenderError> {
    let mut refs = Ref::new(1);
    let catalog_id = refs.bump();
    let ids = PageIds {
        tree: refs.bump(),
        font: refs.bump(),
    };
    let info_id = refs.bump();

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(ids.tree);
    pdf.document_info(info_id).title(TextStr(title));
    pdf.type1_font(ids.font)
        .base_font(Name(b"Helvetica"))
        .encoding_predefined(Name(b"WinAnsiEncoding"));

    let x_objects = write_images(&mut pdf, &mut refs, plan, images)?;
    let page_ids: Vec<(Ref, Ref)> = plan
        .pages
        .iter()
        .map(|_| (refs.bump(), refs.bump()))
        .collect();
    pdf.pages(ids.tree)
        .kids(page_ids.iter().map(|&(page_id, _)| page_id))
        .count(page_ids.len() as i32);
    for (page, (page_id, content_id)) in plan.pages.iter().zip(page_ids) {
        write_page(&mut pdf, page, &ids, (page_id, content_id), &x_objects)?;
    }
    Ok(pdf.finish())
}

/// Objects shared by every page.
struct PageIds {
    tree: Ref,
    font: Ref,
}

/// An embedded image and the resource name pages refer to it by.
struct XObject {
    name: String,
    id: Ref,
}

/// Embeds each distinct card image used by `plan` and returns them by card.
fn write_images<'a>(
    pdf: &mut Pdf,
    refs: &mut Ref,
    plan: &'a DocumentPlan,
    images: &HashMap<CardName, CardImage>,
) -> Result<HashMap<&'a CardName, XObject>, RenderError> {
    let used: BTreeSet<&CardName> = plan
        .pages
        .iter()
        .flat_map(|page| &page.elements)
        .filter_map(|element| match element {
            Element::Card { card, .. } => Some(card),
            _ => None,
        })
        .collect();
    let mut x_objects = HashMap::new();
    for (index, card) in used.into_iter().enumerate() {
        let image = images
            .get(card)
            .ok_or_else(|| RenderError::MissingImage(card.clone()))?;
        let id = refs.bump();
        let mask_id = image.alpha.as_ref().map(|alpha| {
            let mask_id = refs.bump();
            write_plane(pdf, mask_id, image, alpha, Plane::Alpha);
            mask_id
        });
        write_plane(pdf, id, image, &image.rgb, Plane::Color { mask: mask_id });
        let name = format!("Im{index}");
        x_objects.insert(card, XObject { name, id });
    }
    Ok(x_objects)
}

/// Which plane of a card image an image object holds.
enum Plane {
    /// RGB samples, optionally masked by an alpha image object.
    Color { mask: Option<Ref> },
    /// Alpha samples, used as the soft mask of the color plane.
    Alpha,
}

/// Writes `samples` of `image` as a deflate-compressed image object.
fn write_plane(pdf: &mut Pdf, id: Ref, image: &CardImage, samples: &[u8], plane: Plane) {
    let encoded = compress_to_vec_zlib(samples, CompressionLevel::DefaultLevel as u8);
    let mut x_object = pdf.image_xobject(id, &encoded);
    x_object.filter(Filter::FlateDecode);
    x_object.width(image.width as i32);
    x_object.height(image.height as i32);
    x_object.bits_per_component(8);
    match plane {
        Plane::Color { mask } => {
            x_object.color_space().device_rgb();
            if let Some(mask) = mask {
                x_object.s_mask(mask);
            }
        }
        Plane::Alpha => {
            x_object.color_space().device_gray();
        }
    }
}

/// Writes one page object and its content stream.
fn write_page(
    pdf: &mut Pdf,
    page: &Page,
    ids: &PageIds,
    (page_id, content_id): (Ref, Ref),
    x_objects: &HashMap<&CardName, XObject>,
) -> Result<(), RenderError> {
    let mut content = Content::new();
    let mut used: Vec<&XObject> = Vec::new();
    for element in &page.elements {
        match element {
            Element::Card {
                card,
                x,
                y,
                width,
                height,
            } => {
                let x_object = x_objects
                    .get(card)
                    .ok_or_else(|| RenderError::MissingImage(card.clone()))?;
                content.save_state();
                content.transform([*width, 0.0, 0.0, *height, *x, *y]);
                content.x_object(Name(x_object.name.as_bytes()));
                content.restore_state();
                if !used.iter().any(|known| known.id == x_object.id) {
                    used.push(x_object);
                }
            }
            Element::CountBadge {
                count,
                center_x,
                center_y,
                radius,
                font_size,
            } => draw_badge(
                &mut content,
                *count,
                (*center_x, *center_y),
                *radius,
                *font_size,
            ),
            Element::Text {
                text,
                x,
                baseline,
                size,
                align,
            } => {
                let left = match align {
                    TextAlign::Left => *x,
                    TextAlign::Right => x - helvetica::text_width(text, *size),
                };
                draw_text(&mut content, text, left, *baseline, *size);
            }
        }
    }

    let mut writer = pdf.page(page_id);
    writer.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
    writer.parent(ids.tree);
    writer.contents(content_id);
    let mut resources = writer.resources();
    resources.fonts().pair(FONT_NAME, ids.font);
    let mut dict = resources.x_objects();
    for x_object in used {
        dict.pair(Name(x_object.name.as_bytes()), x_object.id);
    }
    dict.finish();
    resources.finish();
    writer.finish();
    pdf.stream(content_id, &content.finish());
    Ok(())
}

/// Draws a white disc with `count` centered on it in black.
fn draw_badge(
    content: &mut Content,
    count: u32,
    (cx, cy): (f32, f32),
    radius: f32,
    font_size: f32,
) {
    let k = radius * KAPPA;
    content.set_fill_gray(1.0);
    content.move_to(cx + radius, cy);
    content.cubic_to(cx + radius, cy + k, cx + k, cy + radius, cx, cy + radius);
    content.cubic_to(cx - k, cy + radius, cx - radius, cy + k, cx - radius, cy);
    content.cubic_to(cx - radius, cy - k, cx - k, cy - radius, cx, cy - radius);
    content.cubic_to(cx + k, cy - radius, cx + radius, cy - k, cx + radius, cy);
    content.close_path();
    content.fill_nonzero();
    content.set_fill_gray(0.0);

    let text = count.to_string();
    let left = cx - helvetica::text_width(&text, font_size) / 2.0;
    // Digits are about two thirds of an em tall; a third below the center centers them.
    draw_text(content, &text, left, cy - font_size / 3.0, font_size);
}

fn draw_text(content: &mut Content, text: &str, left: f32, baseline: f32, size: f32) {
    content.begin_text();
    content.set_font(FONT_NAME, size);
    content.next_line(left, baseline);
    content.show(Str(&helvetica::encode(text)));
    content.end_text();
}
