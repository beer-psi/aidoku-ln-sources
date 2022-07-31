use aidoku::{
	std::{defaults::defaults_get, html::Node, Vec},
	Manga, MangaPageResult,
};
use libaidokuln::{ImageOptions, Padding};

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
		.and_then(|v| v.as_float())
		.unwrap_or(60.0);
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
		lines: lines as usize,
        text_color,
        background_color,
	}
}

pub fn parse_manga_list(html: &Node) -> MangaPageResult {
	MangaPageResult {
		manga: html
			.select("li.novel-item")
			.array()
			.filter_map(|v| v.as_node().map(parse_novel_item).ok())
			.collect::<Vec<_>>(),
		has_more: !html.select("a[rel=next]").array().is_empty(),
	}
}

pub fn parse_novel_item(node: Node) -> Manga {
	let anchor = node.select("a[title]");
	let title = node.select(".novel-title").text().read();
	let id = anchor.attr("href").read();
	let url = anchor.attr("abs:href").read();
	let cover = {
		let img = anchor.select(".novel-cover img");
		let url = img.attr("abs:data-src").read();
		if url.is_empty() {
			img.attr("abs:src").read()
		} else {
			url
		}
	};
	Manga {
		id,
		title,
		url,
		cover,
		..Default::default()
	}
}
