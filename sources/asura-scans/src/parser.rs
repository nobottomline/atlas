//! HTML parsing for Asura Scans (asuracomic.net).
//!
//! Uses `scraper` for CSS selector-based DOM queries.

use atlas_sdk::prelude::*;
use scraper::{Html, Selector, ElementRef};

use crate::BASE_URL;

// ── Manga list (search / browse) ────────────────────────────────────────────

/// Parse manga entries from the series listing page.
pub fn parse_manga_list(html_text: &str) -> Vec<MangaEntry> {
    let document = Html::parse_document(html_text);

    // Each manga card is an <a> inside div.grid
    let card_sel = Selector::parse("div.grid > a[href]").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let title_sel = Selector::parse("div.block > span.block").unwrap();

    let mut entries = Vec::new();

    for card in document.select(&card_sel) {
        let href = match card.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        let id = match extract_manga_id(href) {
            Some(id) => id,
            None => continue,
        };

        let title = card
            .select(&title_sel)
            .next()
            .map(|el| el_text(el))
            .unwrap_or_default();

        let cover_url = card
            .select(&img_sel)
            .next()
            .and_then(|el| el.value().attr("src"))
            .map(|src| absolutize(src));

        entries.push(MangaEntry {
            id,
            title,
            url: format!("{BASE_URL}{href}"),
            cover_url,
            content_rating: ContentRating::Safe,
        });
    }

    entries
}

/// Check if there's a "Next" pagination link.
pub fn has_next_page(html_text: &str) -> bool {
    html_text.contains(">Next<")
}

// ── Manga details ───────────────────────────────────────────────────────────

pub fn parse_manga_details(html_text: &str, id: &str) -> Result<Manga, SourceError> {
    let document = Html::parse_document(html_text);

    let wrapper_sel = Selector::parse("div.grid.grid-cols-12").unwrap();
    let wrapper = document.select(&wrapper_sel).next().ok_or(SourceError::Parse {
        message: "manga details wrapper not found".into(),
    })?;

    // Cover
    let cover_sel = Selector::parse("img[alt=poster]").unwrap();
    let cover_url = wrapper
        .select(&cover_sel)
        .next()
        .and_then(|el| el.value().attr("src"))
        .map(|s| absolutize(s));

    // Title
    let title_sel = Selector::parse("span.text-xl.font-bold").unwrap();
    let title = wrapper
        .select(&title_sel)
        .next()
        .map(|el| el_text(el))
        .unwrap_or_default();

    // Description
    let desc_sel = Selector::parse("span.font-medium.text-sm").unwrap();
    let description = wrapper.select(&desc_sel).next().map(|el| el_text(el));

    // Author & Artist — look for labeled rows
    let author = extract_labeled_field(&wrapper, "Author");
    let artist = extract_labeled_field(&wrapper, "Artist");

    // Genres
    let genre_sel = Selector::parse("button.text-white").unwrap();
    let mut tags = Vec::new();
    let mut content_rating = ContentRating::Safe;

    for btn in wrapper.select(&genre_sel) {
        let genre = el_text(btn);
        if genre.eq_ignore_ascii_case("Adult") || genre.eq_ignore_ascii_case("Ecchi") {
            content_rating = ContentRating::Suggestive;
        }
        if !genre.is_empty() {
            tags.push(genre);
        }
    }

    // Status
    let status = extract_labeled_field(&wrapper, "Status")
        .map(|s| match s.as_str() {
            "Ongoing" => MangaStatus::Ongoing,
            "Hiatus" | "Season End" => MangaStatus::Hiatus,
            "Completed" => MangaStatus::Completed,
            "Dropped" => MangaStatus::Cancelled,
            _ => MangaStatus::Ongoing,
        })
        .unwrap_or(MangaStatus::Ongoing);

    // Content type
    let content_type = extract_labeled_field(&wrapper, "Type")
        .map(|s| match s.as_str() {
            "Manhwa" => ContentType::Manhwa,
            "Manhua" => ContentType::Manhua,
            "Manga" => ContentType::Manga,
            _ => ContentType::Manhwa,
        })
        .unwrap_or(ContentType::Manhwa);

    Ok(Manga {
        id: id.to_string(),
        title,
        url: format!("{BASE_URL}/series/{id}"),
        cover_url,
        author: author.filter(|s| s != "_"),
        artist: artist.filter(|s| s != "_"),
        description,
        tags,
        status,
        content_rating,
        content_type,
        lang: "en".into(),
        alt_titles: vec![],
    })
}

// ── Chapter list ────────────────────────────────────────────────────────────

pub fn parse_chapter_list(html_text: &str, manga_id: &str) -> Vec<Chapter> {
    let document = Html::parse_document(html_text);

    let group_sel = Selector::parse("div.scrollbar-thumb-themecolor > div.group").unwrap();
    let link_sel = Selector::parse("a[href]").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let svg_sel = Selector::parse("svg").unwrap();

    let mut chapters = Vec::new();

    for group in document.select(&group_sel) {
        // Skip premium/locked chapters (ones with SVG lock icon)
        if group.select(&svg_sel).next().is_some() {
            continue;
        }

        let link = match group.select(&link_sel).next() {
            Some(a) => a,
            None => continue,
        };

        let href = match link.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        // Extract chapter number from URL (authoritative source)
        let chapter_num = extract_chapter_number(href);
        let chapter_number: Option<f64> = chapter_num
            .as_deref()
            .and_then(|s| s.parse().ok());

        // Extract chapter title from h3 elements
        let h3_elements: Vec<_> = group.select(&h3_sel).collect();

        // Chapter title from span inside first h3
        let span_sel = Selector::parse("span").unwrap();
        let chapter_title = h3_elements
            .first()
            .and_then(|el| el.select(&span_sel).next())
            .map(|el| el_text(el))
            .filter(|s| !s.is_empty());

        // Date from the second h3 (if present)
        let date_text = h3_elements.get(1).map(|el| el_text(*el));
        let date_updated = date_text.as_deref().and_then(parse_date);

        // Build chapter ID: "manga-slug-/42"
        let chapter_id = format!("{manga_id}/{}", chapter_num.as_deref().unwrap_or("0"));

        chapters.push(Chapter {
            id: chapter_id,
            manga_id: manga_id.to_string(),
            title: chapter_title,
            number: chapter_number,
            volume: None,
            lang: "en".into(),
            date_updated,
            scanlator: Some("Asura Scans".into()),
            url: format!("{BASE_URL}{href}"),
        });
    }

    chapters
}

// ── Page list ───────────────────────────────────────────────────────────────

/// Extract page image URLs from the chapter page.
///
/// Asura embeds page data as JSON in Next.js hydration `<script>` tags:
/// `"pages":[{"order":1,"url":"https://gg.asuracomic.net/storage/media/..."}]`
pub fn parse_pages(html_text: &str) -> Vec<Page> {
    // Clean up split hydration scripts
    let cleaned = html_text.replace(
        r#""])</script><script>self.__next_f.push([1,""#,
        "",
    );

    let mut pages = Vec::new();

    // Find the pages JSON marker
    let marker = r#"\"pages\":["#;
    let Some(start) = cleaned.find(marker) else {
        return pages;
    };

    let slice = &cleaned[start..];

    // Find all image URLs in the pages array
    let media_prefix = "https://gg.asuracomic.net/storage/media/";
    let mut remaining = slice;
    let mut index: u32 = 0;

    loop {
        let Some(pos) = remaining.find(media_prefix) else {
            break;
        };
        remaining = &remaining[pos..];

        // Find the end of the URL (terminated by escaped quote or regular quote)
        let end = remaining
            .find(r#"\""#)
            .or_else(|| remaining.find('"'))
            .unwrap_or(remaining.len());

        let url = remaining[..end].replace('\\', "");
        remaining = &remaining[end..];

        // Skip duplicates
        if pages.iter().any(|p: &Page| {
            if let PageData::Url(ref u) = p.data {
                u == &url
            } else {
                false
            }
        }) {
            continue;
        }

        pages.push(Page {
            index,
            data: PageData::Url(url),
        });
        index += 1;
    }

    pages
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Extract manga slug from URL path.
///
/// Asura appends a random suffix to manga slugs. We keep everything up to
/// and including the last `-` so URLs remain stable.
///
/// Example: `/series/swordmasters-youngest-son-cb22671f` → `swordmasters-youngest-son-`
fn extract_manga_id(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or("");

    let mut segments = path.split('/');
    while let Some(seg) = segments.next() {
        if seg == "series" {
            if let Some(manga_seg) = segments.next() {
                if let Some(pos) = manga_seg.rfind('-') {
                    return Some(manga_seg[..=pos].to_string());
                }
                return Some(manga_seg.to_string());
            }
        }
    }
    None
}

/// Extract chapter number string from URL.
fn extract_chapter_number(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or("");
    let mut segments = path.split('/');
    while let Some(seg) = segments.next() {
        if seg == "chapter" {
            return segments.next().map(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect()
            });
        }
    }
    None
}

/// Extract text content from a labeled field row.
///
/// Asura uses patterns like:
/// ```html
/// <div><h3>Author</h3><h3>Author Name</h3></div>
/// ```
fn extract_labeled_field(parent: &ElementRef, label: &str) -> Option<String> {
    let h3_sel = Selector::parse("h3").unwrap();

    // Look for divs containing h3 pairs
    let div_sel = Selector::parse("div").unwrap();
    for div in parent.select(&div_sel) {
        let h3s: Vec<_> = div.select(&h3_sel).collect();
        if h3s.len() >= 2 {
            let label_text = el_text(h3s[0]);
            if label_text.contains(label) {
                let value = el_text(h3s[1]);
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// Get trimmed text content of an element.
fn el_text(el: ElementRef) -> String {
    el.text().collect::<String>().trim().to_string()
}

/// Make a URL absolute if it's relative.
fn absolutize(url: &str) -> String {
    if url.starts_with("http") {
        url.to_string()
    } else if url.starts_with("//") {
        format!("https:{url}")
    } else if url.starts_with('/') {
        format!("{BASE_URL}{url}")
    } else {
        url.to_string()
    }
}

/// Parse date strings like "December 1st 2024", "March 15 2024".
fn parse_date(text: &str) -> Option<i64> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    let month = match parts[0].to_lowercase().as_str() {
        "january" => 1,
        "february" => 2,
        "march" => 3,
        "april" => 4,
        "may" => 5,
        "june" => 6,
        "july" => 7,
        "august" => 8,
        "september" => 9,
        "october" => 10,
        "november" => 11,
        "december" => 12,
        _ => return None,
    };

    // Strip ordinal suffixes: "1st" → "1", "23rd" → "23"
    let day_str: String = parts[1].chars().filter(|c| c.is_ascii_digit()).collect();
    let day: u32 = day_str.parse().ok()?;

    let year: i64 = parts[2].parse().ok()?;

    // Rough Unix timestamp: days since epoch
    let days = days_since_epoch(year, month, day);
    Some(days * 86400)
}

fn days_since_epoch(year: i64, month: u32, day: u32) -> i64 {
    // Simplified date → days calculation
    let mut y = year;
    let mut m = month as i64;

    if m <= 2 {
        y -= 1;
        m += 12;
    }

    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
