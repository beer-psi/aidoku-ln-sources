use crate::helper::*;
extern crate alloc;
use aidoku::{
	error::Result,
	helpers::node::NodeHelpers,
	prelude::{format, println},
	std::{defaults::defaults_get, net::HttpMethod, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer, Page,
};
use alloc::{string::ToString, vec};
use deunicode::deunicode;
use libaidokuln::{fonts, ImageOptions, Padding};

pub fn parse_manga_list(
	base_url: String,
	filters: Vec<Filter>,
	page: i32,
) -> Result<MangaPageResult> {
	let mut title: String = String::new();
	let tag_list = genres();
	let mut tag: String = String::new();
	let mut sort: String = String::new();
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
				} else if filter.name.as_str() == "Sort" {
					let index = filter.value.as_int().unwrap_or(-1);
					sort = match index {
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
		let mut mangas: Vec<Manga> = Vec::new();
		let url = format!("{}/search/", base_url);
		let request = Request::new(url.as_str(), HttpMethod::Post);
		let body_data = format!("searchkey={}", title);
		let html = request
			.header("X-Requested-With", "XMLHttpRequest")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body_data.as_bytes())
			.html()?;

		for manga in html.select(".ul-list1 .con").array() {
			let manga_node = manga.as_node()?;
			let title = manga_node.select("h3").text().read();
			let id = base_url.clone() + &manga_node.select("a").attr("href").read();
			let cover = manga_node.select("img").attr("src").read();
			mangas.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Rtl,
			});
		}
		let last_page = html.select(".pages li").text().read();
		let has_more = last_page.contains(">>");
		Ok(MangaPageResult {
			manga: mangas,
			has_more,
		})
	} else if title.is_empty() && tag.is_empty() && sort.is_empty() {
		parse_manga_listing(
			base_url.clone(),
			format!("{}/latest-release-novel/{}/", base_url, page),
			String::from("Latest"),
		)
	} else {
		let url = get_filter_url(base_url.clone(), tag, sort, page);
		println!("{}", url);
		parse_manga_listing(base_url, url, String::new())
	}
}

pub fn parse_manga_listing(
	base_url: String,
	url: String,
	list_name: String,
) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let html = Request::new(&url, HttpMethod::Get).html()?;
	for manga in html.select(".ul-list1 .con").array() {
		let manga_node = manga.as_node()?;
		let title = manga_node.select("h3").text().read();
		let id = base_url.clone() + &manga_node.select("a").attr("href").read();
		let cover = manga_node.select("img").attr("src").read();
		mangas.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Rtl,
		});
	}
	let last_page = html.select(".pages li").text().read();
	let has_more = last_page.contains(">>") && list_name != "Popular";
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(id: String) -> Result<Manga> {
	let html = Request::new(&id, HttpMethod::Get).html()?;
	let title = html.select(".m-desc .tit").text().read();
	let cover = html.select(".pic img").attr("src").read();
	let author = get_author(html.select("meta[name=description]").attr("content").read());
	let artist = html.select("#last_episode small").text().read();
	let description = String::from(html.select(".inner").first().text().read().trim());
	let status = manga_status(html.select(".s1.s2").text().read().to_uppercase());
	let categories: Vec<String> = html
		.select("meta[property=og:novel:genre]")
		.attr("content")
		.read()
		.split(',')
		.map(String::from)
		.collect();
	let mut nsfw = MangaContentRating::Safe;
	for tag in categories.clone() {
		if tag == "Adult" || tag == "Smut" || tag == "Mature" || tag == "Ecchi" {
			nsfw = MangaContentRating::Nsfw;
		} else {
			nsfw = MangaContentRating::Safe;
		}
	}
	Ok(Manga {
		id: id.clone(),
		cover,
		title,
		author,
		artist,
		description,
		url: id,
		categories,
		status,
		nsfw,
		viewer: MangaViewer::Ltr,
	})
}

pub fn parse_chapter_list(base_url: String, id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	println!("{}", id);
	let html1 = Request::new(&id, HttpMethod::Get).html()?;
	/*let next_page = get_full_url(
		base_url.clone(),
		html1.select(".page a:eq(2)").attr("href").read(),
	);*/
	let last_page = extract_i32_from_string(html1.select(".page a:eq(4)").attr("href").read());
	let mut page = 1;
	while page != last_page {
		let url = format!("{}/{}.html", id.replace(".html", ""), page);
		let html = Request::new(&url, HttpMethod::Get).html()?;
		for chapter in html.select(".m-newest2 .ul-list5 a").array() {
			let chapter_node = chapter.as_node()?;
			let title = chapter_node.text().read();
			let chapter_id = get_full_url(base_url.clone(), chapter_node.attr("href").read());
			let chapter_number = get_chapter_number(title.clone());
			println!("{}", chapter_number);
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

pub fn parse_page_list(id: String) -> Result<Vec<Page>> {
	let html = Request::new(&id, HttpMethod::Get).html()?;
	let raw_text = html.select(".txt").text_with_newlines();
	let novel_text = deunicode(&html_escape::decode_html_entities(&raw_text));
	println!("{}", novel_text);
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

pub fn modify_image_request(base_url: String, request: Request) {
	request.header("Referer", &base_url);
}

pub fn handle_url(url: String) -> Result<DeepLink> {
	Ok(DeepLink {
		manga: parse_manga_details(url).ok(),
		chapter: None,
	})
}
pub fn image_options() -> ImageOptions {
	let horizontal_padding = defaults_get("horizontalPadding")
		.and_then(|v| v.as_float())
		.unwrap_or(40.0);
	let vertical_padding = defaults_get("verticalPadding")
		.and_then(|v| v.as_float())
		.unwrap_or(40.0);
	let page_width = defaults_get("pageWidth")
		.and_then(|v| v.as_float())
		.unwrap_or(800.0);
	let constant_width = defaults_get("constantWidth")
		.and_then(|v| v.as_bool())
		.unwrap_or(true);
	let lines = defaults_get("linesPerPage")
		.and_then(|v| v.as_int())
		.map(|v| v.try_into().unwrap_or(35))
		.unwrap_or(35);
	let text_color = defaults_get("textColor")
		.and_then(|v| v.as_int())
		.map(|v| v.try_into().unwrap_or(0xFFFFFF))
		.unwrap_or(0xFFFFFF);
	let background_color = defaults_get("bgColor")
		.and_then(|v| v.as_int())
		.map(|v| v.try_into().unwrap_or(0x000000))
		.unwrap_or(0x000000);
	ImageOptions {
		padding: Padding(horizontal_padding as f32, vertical_padding as f32),
		width: page_width as f32,
		constant_width,
		lines,
		text_color,
		background_color,
	}
}
