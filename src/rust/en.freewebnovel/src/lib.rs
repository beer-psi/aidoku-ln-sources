#![no_std]
extern crate alloc;
use aidoku::{
	error::Result,
	prelude::*,
	helpers::node::NodeHelpers,
	prelude::format,
	std::{net::Request, defaults::defaults_get},
	std::{net::HttpMethod, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use helper::*;
use parser::*;
use alloc::{string::ToString, vec};
use deunicode::deunicode;
use libaidokuln::fonts;

pub mod helper;
pub mod parser;

const BASE_URL: &str = "https://freewebnovel.com";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let tag_list = genres();
	let mut tag: String = String::new();
	let mut lang: String = String::new();
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				title = filter.value.as_string()?.read();
			}
			FilterType::Select => {
				if filter.name.as_str() == "Genres" {
					let index = filter.value.as_int()? as usize;
					match index {
						0 => continue,
						_ => tag = String::from(tag_list[index]),
					}
				} else if filter.name.as_str() == "Original Language" {
					let index = filter.value.as_int().unwrap_or(-1);
					lang = match index {
						0 => String::new(),
						1 => String::from("chinese-novel"),
						2 => String::from("korean-novel"),
						3 => String::from("japanese-novel"),
						4 => String::from("english-novel"),
						_ => continue,
					};
				}
			}
			_ => continue,
		}
	}

	if !title.is_empty() {
		let url = format!("{}/search/", BASE_URL);
		let request = Request::new(&url, HttpMethod::Post);
		let body_data = format!("searchkey={}", title);
		let html = request
			.header("X-Requested-With", "XMLHttpRequest")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body_data.as_bytes())
			.html()?;
		parse_manga_listing(html)
	} else if title.is_empty() && tag.is_empty() && lang.is_empty() {
		let html = Request::new(
			format!("{}/latest-release-novel/{}/", BASE_URL, page),
			HttpMethod::Get,
		)
		.html()?;
		parse_manga_listing(html)
	} else {
		let url = get_filter_url(String::from(BASE_URL), tag, lang, page);
		let html = Request::new(
			&url,
			HttpMethod::Get
		)
		.html()?;
		parse_manga_listing(html)
	}
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Latest" => format!("{}/latest-release-novel/{}/", BASE_URL, page),
		"Popular" => format!("{}/most-popular-novel/", BASE_URL),
		"New" => format!("{}/latest-novel/{}/", BASE_URL, page),
		_ => format!("{}/completed-novel/{}/", BASE_URL, page),
	};
	let html = Request::new(&url, HttpMethod::Get).html()?;
	parser::parse_manga_listing(html)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let html = Request::new(format!("{}{}", BASE_URL, id), HttpMethod::Get).html()?;
	parser::parse_manga_details(html, id)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let url = format!("{}{}", BASE_URL, id);
	let html1 = Request::new(&url, HttpMethod::Get).html()?;
	let last_page = extract_i32_from_string(html1.select(".page a:eq(4)").attr("href").read());
	let mut page = 1;
	while page != last_page {
		let url = format!("{}/{}.html", url.replace(".html", ""), page);
		let html = Request::new(&url, HttpMethod::Get).html()?;
		for chapter in html.select(".m-newest2 .ul-list5 a").array() {
			let chapter_node = chapter.as_node()?;
			let title = chapter_node.text().read();
			let chapter_id = chapter_node.attr("href").read();
			let chapter_number = get_chapter_number(chapter_id.clone());
			chapters.push(Chapter {
				id: chapter_id.clone(),
				title,
				volume: -1.0,
				chapter: chapter_number,
				date_updated: -1.0,
				scanlator: String::new(),
				url: chapter_id,
				lang: String::from("en"),
			});
		}
		page += 1;
	}
	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let html = Request::new(format!("{}{}", BASE_URL, id), HttpMethod::Get).html()?;
	let raw_text = html.select(".txt").text_with_newlines();
	let novel_text = deunicode(&html_escape::decode_html_entities(&raw_text));
	let font = {
		let font_name = defaults_get("fontName")
			.and_then(|v| v.as_string())
			.map(|v| v.read())
			.unwrap_or_else(|_| "times".to_string());
		let font_size = defaults_get("fontSize")
			.and_then(|v| v.as_string())
			.map(|v| v.read())
			.unwrap_or_else(|_| "18".to_string());
		fonts::Font::from_name([font_name, font_size].concat())
	};
	let options = image_options();
	let page_count =
		((libaidokuln::break_apart(&novel_text, options.width - options.padding.0 * 2.0, &font)
			.split
			.len() as f32)
			/ (options.lines as f32)
			+ 1.0) as usize;
	let mut pages = Vec::with_capacity(page_count);
	for i in 0..page_count {
		let data = libaidokuln::write_text(&novel_text, i + 1, font, options);
		let mut buf = vec![0; data.len() * 4 / 3 + 4];
		let bytes = base64::encode_config_slice(data, base64::STANDARD, &mut buf);
		buf.resize(bytes, 0);
		if let Ok(base64) = String::from_utf8(buf) {
			pages.push(Page {
				base64,
				index: (i + 1) as i32,
				..Default::default()
			});
		}
	}
	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request.header("Referer", BASE_URL);
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: get_manga_details(url).ok(),
		chapter: None,
	})
}
