use crate::helper::*;
extern crate alloc;
use aidoku::{
	error::Result,
	std::{defaults::defaults_get, String, Vec, html::Node}, Manga, MangaContentRating, MangaPageResult, MangaStatus,
	MangaViewer,
};
use libaidokuln::{ImageOptions, Padding};



pub fn parse_manga_listing(
	html: Node,
) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	for manga in html.select(".ul-list1 .con").array() {
		let manga_node = manga.as_node()?;
		let title = manga_node.select("h3").text().read();
		let id = manga_node.select("a").attr("href").read();
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
}

pub fn parse_manga_details(html: Node, id: String) -> Result<Manga> {
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
		if tag == "Adult" || tag == "Smut" || tag == "Mature" {
			nsfw = MangaContentRating::Nsfw;
		} else if tag == "Ecchi" {
			nsfw = MangaContentRating::Suggestive
		}
		else {
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
