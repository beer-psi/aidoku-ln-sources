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
use parser::{image_options, parse_manga_list};

mod parser;

#[derive(Default, Deserializable, Debug, PartialEq)]
struct LNSearchLiveResponse {
	resultview: Option<String>,
	success: bool,
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut use_adv_search = false;
	let mut adv_query_string =
		String::from("https://www.lightnovelpub.com/searchadv?tagcon=and&pageNo=");
	adv_query_string.push_str(itoa::Buffer::new().format(page));

	let mut translation_status = "all";
	let mut sort_by = "updated";

	for filter in filters {
		match filter.kind {
			FilterType::Genre => if let Ok(id) = filter.object.get("id").as_string()
								    && filter.value.as_int().unwrap_or(-1) == 1 {
				use_adv_search = true;
				adv_query_string.push_str("&categories=");
				adv_query_string.push_str(&id.read());
			}
			FilterType::Check => if let Ok(id) = filter.object.get("id").as_string()
									&& filter.value.as_int().unwrap_or(-1) == 1 {
				use_adv_search = true;
				adv_query_string.push_str(&id.read());
			}
			FilterType::Select => {
				match filter.name.as_str() {
					"Category Condition" => {
						adv_query_string.push_str("&ctgcon=");
						adv_query_string.push_str(match filter.value.as_int().unwrap_or(0) {
							0 => "and",
							1 => "or",
							_ => "and",
						})
					}
					"Rating Condition" => {
						adv_query_string.push_str("&ratcon=");
						adv_query_string.push_str(match filter.value.as_int().unwrap_or(0) {
							0 => "min",
							1 => "max",
							_ => "min",
						})
					}
					"Rating" => {
						let rating = filter.value.as_int().unwrap_or(0);
						if rating > 0 {
							use_adv_search = true;
						}
						adv_query_string.push_str("&rating=");
						adv_query_string.push_str(itoa::Buffer::new().format(rating));
					}
					"Translation Status" => {
						let status = filter.value.as_int().unwrap_or(0);
						adv_query_string.push_str("&status=");
						adv_query_string.push_str(itoa::Buffer::new().format(status));
						translation_status = match status {
							0 => "all",
							1 => "completed",
							2 => "ongoing",
							_ => "all",
						};
					}
					"Sort by" => {
						let sort = filter.value.as_int().unwrap_or(-1);
						adv_query_string.push_str("&sort=");
						adv_query_string.push_str(match sort {
							0 => "srank",
							1 => "srate",
							2 => "sread",
							3 => "sreview",
							4 => "abc",
							5 => "sdate",
							_ => "sdate"
						});
						sort_by = match sort {
							1 => "popular",
							5 => "new",
							_ => "updated"
						};
					}
					_ => continue
				}
			}
			FilterType::Title => {
				let title = encode_uri_component(
					filter
						.value
						.as_string()
						.map(|v| v.read())
						.unwrap_or_default(),
				);
				let token_html = Request::get("https://www.lightnovelpub.com/search").html()?;
				let token_node = token_html.select("#novelSearchForm input[type=hidden]");
				let token = token_node.attr("value").read();

				let data = Request::post("https://www.lightnovelpub.com/lnsearchlive")
					.header("X-Requested-With", "XMLHttpRequest")
					.header("LNRequestVerifyToken", &token)
					.header(
						"Content-Type",
						"application/x-www-form-urlencoded; charset=UTF-8",
					)
					.header("Content-Length", "16")
					.header("Referer", "https://www.lightnovelpub.com/search")
					.body(["inputContent=", &title].concat().as_bytes())
					.json()?;
				let json = LNSearchLiveResponse::try_from(data.as_object()?)?;
				return if json.success
						  && let Some(resultview) = json.resultview {
					let html = Node::new_fragment_with_uri(resultview, "https://www.lightnovelpub.com/search")?;
					Ok(parse_manga_list(&html))
				} else {
					Err(AidokuError { reason: AidokuErrorKind::Unimplemented })
				}
			}
			_ => continue,
		}
	}

	let html = if use_adv_search {
		Request::get(adv_query_string)
			.header("Referer", "https://www.lightnovelpub.com/searchadv/")
			.html()?
	} else if sort_by != "updated" || translation_status != "all" || page > 1 {
		Request::get(
			[
				"https://www.lightnovelpub.com/genre/all/",
				sort_by,
				"/",
				translation_status,
				"/",
				itoa::Buffer::new().format(page),
			]
			.concat(),
		)
		.html()?
	} else {
		Request::get("https://www.lightnovelpub.com/").html()?
	};

	let mut ret = parse_manga_list(&html);
	if page == 1 {
		ret.has_more = true;
	}
	return Ok(ret)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut url = match listing.name.as_str() {
		"Recently Updated" => String::from("https://www.lightnovelpub.com/latest-updates?p="),
		"Popular" => String::from("https://www.lightnovelpub.com/genre/all/popular/all/"),
		_ => {
			return Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			})
		}
	};
	url.push_str(itoa::Buffer::new().format(page));

	let html = Request::get(&url).html()?;
	Ok(parse_manga_list(&html))
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let url = ["https://www.lightnovelpub.com", &id].concat();
	let html = Request::get(&url).html()?;

	let categories = html
		.select("div.categories li")
		.array()
		.filter_map(|v| v.as_node().map(|v| v.text().read().trim().to_string()).ok())
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
		cover: {
			let img = html.select(".cover img");
			let url = img.attr("abs:data-src").read();
			if url.is_empty() {
				img.attr("abs:src").read()
			} else {
				url
			}
		},
		title: html
			.select(".novel-info h1.novel-title")
			.text()
			.read()
			.trim()
			.to_string(),
		author: html.select("span[itemprop=author]").text().read(),
		artist: String::new(),
		description: html
			.select(".summary .content.expand-wrapper")
			.text_with_newlines(),
		url,
		categories,
		status: if html
			.select(".header-stats span:nth-child(4)")
			.text()
			.read()
			.contains("Ongoing")
		{
			MangaStatus::Ongoing
		} else {
			MangaStatus::Completed
		},
		nsfw, // todo determine
		viewer: MangaViewer::Ltr,
	})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let html = Request::get(
		[
			"https://www.lightnovelpub.com",
			&id,
			"/chapters/?chorder=desc",
		]
		.concat(),
	)
	.html()?;

	let nav = html.select(".pagenav ul li");
	let arr_pages = nav.array();

	let final_page = if arr_pages.len() == 1 {
		1
	} else if arr_pages.len() == 6 {
		let last_page = arr_pages
			.get(5)
			.as_node()?
			.select("a")
			.attr("href")
			.read()
			.replace("?chorder=desc", "");
		last_page
			.split('-')
			.last()
			.unwrap_or("1")
			.parse::<usize>()
			.unwrap_or(1)
	} else {
		arr_pages.len().saturating_sub(1)
	};

	let mut chapters: Vec<Chapter> = Vec::new();
	for i in 1..=final_page {
		let html = if i == 1 {
			Ok(html.clone())
		} else {
			Request::get(
				[
					"https://www.lightnovelpub.com",
					&id,
					"/chapters/page-",
					itoa::Buffer::new().format(i),
					"?chorder=desc",
				]
				.concat(),
			)
			.html()
		};

		if let Ok(html) = html {
			chapters.extend(html.select(".chapter-list li").array().filter_map(|v| {
				v.as_node()
					.map(|v| {
						let id = v.select("a").attr("href").read();
						let url = v.select("a").attr("abs:href").read();
						let title = v.select(".chapter-title").text().read().trim().to_string();
						let chapter = v
							.attr("data-chapterno")
							.read()
							.parse::<f32>()
							.unwrap_or(-1.0);
						let mut volume = v
							.attr("data-volumeno")
							.read()
							.parse::<f32>()
							.unwrap_or(-1.0);
						if volume == 0.0 {
							volume = -1.0;
						}
						let date_updated = v.select("time").attr("datetime").as_date(
							"yyyy-MM-dd HH:mm",
							Some("en_US"),
							Some("Etc/GMT-12"),
						);
						Chapter {
							id,
							title,
							chapter,
							volume,
							date_updated,
							url,
							..Default::default()
						}
					})
					.ok()
			}));
		}
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let html = Request::get(["https://www.lightnovelpub.com", &id].concat()).html()?;
	let mut html_raw = html.select("#chapter-container").html().read();
	for sub in html.select("sub").array() {
		if let Ok(node) = sub.as_node() {
			html_raw = html_raw.replace(&node.outer_html().read(), "");
		}
	}
	let text_content = deunicode(&html_escape::decode_html_entities(
		&Node::new_fragment(html_raw)?
			.select("body")
			.text_with_newlines(),
	));

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
	let page_count = ((libaidokuln::break_apart(
		&text_content,
		options.width - options.padding.0 * 2.0,
		&font,
	)
	.split
	.len() as f32)
		/ (options.lines as f32) + 1.0) as usize;
	let mut pages = Vec::with_capacity(page_count);
	for i in 0..page_count {
		let data = libaidokuln::write_text(&text_content, i + 1, font, options);
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
	request
		.header("Referer", "https://www.lightnovelpub.com/")
		.header("Origin", "https://www.lightnovelpub.com");
}

#[handle_url]
fn handle_url(url: String) -> Result<DeepLink> {
	let url = url.replace("https://www.lightnovelpub.com/", "");

	let mut components = url.split('/');
	let manga_id = [
		"/",
		components.next().unwrap_or_default(),
		"/",
		components.next().unwrap_or_default(),
	]
	.concat();

	Ok(DeepLink {
		manga: get_manga_details(manga_id).ok(),
		chapter: components.next().map(|_| Chapter {
			id: ["/", &url].concat(),
			..Default::default()
		}),
	})
}
