use aidoku::{
	prelude::format,
	std::String,
	std::Vec,
	MangaStatus,
};

pub fn get_author(text: String) -> String {
	String::from(&text[text.find("author").unwrap_or(0) + 6..text.rfind(',').unwrap_or(0)])
}

pub fn next() -> bool {
	true
}
pub fn manga_status(status: String) -> MangaStatus {
	match status.as_str() {
		"ONGOING" => MangaStatus::Ongoing,
		"COMPLETED" => MangaStatus::Completed,
		"HIATUS" => MangaStatus::Hiatus,
		"CANCELLED" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

pub fn get_full_url(base_url: String, id: String) -> String {
	format!("{}{}", base_url, id)
}

pub fn get_chapter_number(id: String) -> f32 {
	id.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

pub fn extract_i32_from_string(text: String) -> i32 {
	text.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0) as i32
}

pub fn get_filter_url(base_url: String, tag: String, sort_by: String, page: i32) -> String {
	let mut url = String::new();
	if !tag.is_empty() {
		url = format!("{}{}{}.html", base_url, tag, page);
	}
	if !sort_by.is_empty() {
		url = format!("{}/latest-release-novel/{}/{}/", base_url, sort_by, page);
	}
	url
}

pub fn genres() -> [&'static str; 37] {
	[
		"",
		"/genre/Action/",
		"/genre/Adult/",
		"/genre/Adventure/",
		"/genre/Comedy/",
		"/genre/Drama/",
		"/genre/Ecchi/",
		"/genre/Fantasy/",
		"/genre/Gender+Bender/",
		"/genre/Harem/",
		"/genre/Historical/",
		"/genre/Horror/",
		"/genre/Josei/",
		"/genre/Game/",
		"/genre/Martial+Arts/",
		"/genre/Mature/",
		"/genre/Mecha/",
		"/genre/Mystery/",
		"/genre/Psychological/",
		"/genre/Romance/",
		"/genre/School+Life/",
		"/genre/Sci-fi/",
		"/genre/Seinen/",
		"/genre/Shoujo/",
		"/genre/Shounen+Ai/",
		"/genre/Shounen/",
		"/genre/Slice+of+Life/",
		"/genre/Smut/",
		"/genre/Sports/",
		"/genre/Supernatural/",
		"/genre/Tragedy/",
		"/genre/Wuxia/",
		"/genre/Xianxia/",
		"/genre/Xuanhuan/",
		"/genre/Yaoi/",
		"/genre/Eastern/",
		"/genre/Reincarnation/",
	]
}
