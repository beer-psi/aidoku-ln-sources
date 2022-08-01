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

pub fn parse_manga_list(html: &Node) -> MangaPageResult {
	MangaPageResult {
		manga: html
			.select(".col-content .li-row .con")
			.array()
			.filter_map(|v| v.as_node().map(parse_novel_item).ok())
			.collect::<Vec<_>>(),
		has_more: html.select(".pages strong").text().read()
			!= html.select(".pages a:last-child").text().read(),
	}
}

pub fn parse_novel_item(node: Node) -> Manga {
	let id = node.select(".pic a").attr("href").read();
	let title = node.select(".txt .tit").text().read();
	let url = node.select(".pic a").attr("abs:href").read();
	let cover = node.select(".pic img").attr("abs:src").read();
	Manga {
		id,
		title,
		url,
		cover,
		..Default::default()
	}
}
