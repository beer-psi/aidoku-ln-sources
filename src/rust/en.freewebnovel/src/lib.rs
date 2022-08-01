#![no_std]
#![feature(let_chains)]
extern crate alloc;
use alloc::{string::ToString, vec};

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::{node::NodeHelpers, uri::encode_uri_component},
	prelude::*,
	std::{defaults::defaults_get, html::Node, net::Request, String, Vec},
	Chapter, DeepLink, Filter, FilterType, Listing, Manga, MangaContentRating, MangaPageResult,
	MangaStatus, MangaViewer, Page,
};
use deunicode::deunicode;
use libaidokuln::fonts;

mod parser;
use parser::image_options;
use safe_regex::{regex, Matcher1};

static GENRE_LIST: [&str; 37] = [
	"All",
	"Action",
	"Adult",
	"Adventure",
	"Comedy",
	"Drama",
	"Ecchi",
	"Fantasy",
	"Gender Bender",
	"Harem",
	"Historical",
	"Horror",
	"Josei",
	"Game",
	"Martial Arts",
	"Mature",
	"Mecha",
	"Mystery",
	"Psychological",
	"Romance",
	"School Life",
	"Sci-fi",
	"Seinen",
	"Shoujo",
	"Shounen Ai",
	"Shounen",
	"Slice of Life",
	"Smut",
	"Sports",
	"Supernatural",
	"Tragedy",
	"Wuxia",
	"Xianxia",
	"Xuanhuan",
	"Yaoi",
	"Eastern",
	"Reincarnation",
];

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(title) = filter.value.as_string() {
					let title = encode_uri_component(title.read());
					let html = Request::post("https://freewebnovel.com/search/")
						.header("Content-Type", "application/x-www-form-urlencoded")
						.body(["searchkey=", &title].concat())
						.html()?;
					let mut ret = parser::parse_manga_list(&html);
					ret.has_more = false;
					return Ok(ret);
				}
			}
			FilterType::Select => {
				let value = filter.value.as_int().unwrap_or(-1);
				if value > 0 {
					let html = Request::get(
						[
							"https://freewebnovel.com/genre/",
							&GENRE_LIST[value as usize].replace(char::is_whitespace, "+"),
							"/",
						]
						.concat(),
					)
					.html()?;
					return Ok(parser::parse_manga_list(&html));
				}
			}
			_ => continue,
		}
	}

	let html = Request::get(
		[
			"https://freewebnovel.com/latest-release-novel/",
			itoa::Buffer::new().format(page),
			"/",
		]
		.concat(),
	)
	.html()?;

	Ok(parser::parse_manga_list(&html))
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut url = String::from(match listing.name.as_str() {
		"Popular" => "https://freewebnovel.com/most-popular-novel/",
		"New" => "https://freewebnovel.com/latest-novel/",
		"Completed" => "https://freewebnovel.com/completed-novel/",
		_ => "https://freewebnovel.com/latest-release-novel/",
	});
	if listing.name != "Popular" {
		url.push_str(itoa::Buffer::new().format(page));
		url.push('/');
	}

	let html = Request::get(url).html()?;
	let mut ret = parser::parse_manga_list(&html);
	if listing.name == "Popular" {
		ret.has_more = false;
	}
	Ok(ret)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = ["https://freewebnovel.com", &id].concat();
	let html = Request::get(&url).html()?;

	let categories = html
		.select(".txt span[title=Genre] + .right a")
		.array()
		.filter_map(|v| v.as_node().map(|v| v.text().read()).ok())
		.collect::<Vec<_>>();

	let nsfw = if categories.iter().any(|v| v == "Adult") {
		MangaContentRating::Nsfw
	} else if categories.iter().any(|v| v == "Ecchi") {
		MangaContentRating::Suggestive
	} else {
		MangaContentRating::Safe
	};

	Ok(Manga {
		id,
		title: html.select(".m-desc .tit").text().read(),
		cover: html.select(".pic img").attr("abs:src").read(),
		author: html
			.select(".txt span[title=Author] + .right a")
			.array()
			.filter_map(|v| v.as_node().map(|v| v.text().read()).ok())
			.collect::<Vec<_>>()
			.join(", "),
		artist: String::new(),
		description: html.select(".m-desc .txt .inner").text_with_newlines(),
		url,
		categories,
		status: if html
			.select(".txt span[title=Status] + .right a")
			.text()
			.read()
			.to_lowercase()
			== "completed"
		{
			MangaStatus::Completed
		} else {
			MangaStatus::Ongoing
		},
		nsfw,
		viewer: MangaViewer::Ltr,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let html = Request::get(["https://freewebnovel.com", &id].concat()).html()?;

	let final_page = html
		.select("a.index-container-btn:contains(Last)")
		.attr("href")
		.read()
		.split('/')
		.last()
		.unwrap_or_default()
		.replace(".html", "")
		.parse::<usize>()
		.unwrap_or(1);
	
	let mut ret = Vec::with_capacity(final_page * 40);

	for i in 1..=final_page {
		let html = if i == 1 {
			Ok(html.clone())
		} else {
			Request::get(
				[
					"https://freewebnovel.com",
					&id.replace(".html", ""),
					"/",
					itoa::Buffer::new().format(i),
					".html",
				]
				.concat(),
			)
			.html()
		};

		if let Ok(html) = html {
			ret.extend(html.select(".m-newest2 ul li")
				.array()
				.filter_map(|v| v.as_node().map(|node| {
					let mut title = node.select("a").text().read().trim().to_string();
					let (volume, chapter) = {
						let matcher: Matcher1<_> = regex!(br".*([-+]?(?:[0-9]*[.])?[0-9]+).*");
						if let Some((matchnum,)) = matcher.match_slices(title.as_bytes()) {
							if title.to_lowercase().starts_with("vol") {
								let volume = String::from_utf8_lossy(matchnum).to_string();
								title = title.split(&volume).nth(1).unwrap_or(&title).to_string();
								if let Some((matchnum,)) = matcher.match_slices(title.as_bytes()) {
									let chapter = String::from_utf8_lossy(matchnum).to_string();
									title = title.split(&chapter).nth(1).unwrap_or(&title).to_string();
									(volume.parse::<f32>().unwrap_or(-1.0), chapter.parse::<f32>().unwrap_or(-1.0))
								} else {
									(volume.parse::<f32>().unwrap_or(-1.0), -1.0)
								}
							} else {
								let chapter = String::from_utf8_lossy(matchnum).to_string();
								title = title.split(&chapter).nth(1).unwrap_or(&title).to_string();
								(-1.0, chapter.parse::<f32>().unwrap_or(-1.0))
							}
						} else {
							(-1.0, -1.0)
						}
					};
					Chapter {
						id: node.select("a").attr("href").read(),
						url: node.select("a").attr("abs:href").read(),
						title,
						volume,
						chapter,
						..Default::default()
					}
				}).ok())
			);
		}
	}

	Ok(ret)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	todo!()
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", "https://freewebnovel.com/")
		.header("Origin", "https://freewebnovel.com");
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	todo!()
}
