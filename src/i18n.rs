use serde::Deserialize;
use std::collections::HashMap;

static EN_TOML: &str = include_str!("../locales/en.toml");
static CA_TOML: &str = include_str!("../locales/ca.toml");

fn embedded_locales() -> HashMap<&'static str, &'static str> {
    HashMap::from([("en", EN_TOML), ("ca", CA_TOML)])
}

// Raw TOML structure matching the locale files.
#[derive(Deserialize)]
struct RawLocale {
    panels: HashMap<String, String>,
    placeholders: HashMap<String, String>,
    status: HashMap<String, String>,
    prompts: HashMap<String, String>,
    modals: HashMap<String, String>,
    labels: HashMap<String, String>,
    instructions: HashMap<String, String>,
    help_sections: HashMap<String, String>,
    help_items: HashMap<String, String>,
    time: HashMap<String, String>,
    sort: HashMap<String, String>,
    header: HashMap<String, String>,
    discovery: HashMap<String, String>,
    templates: HashMap<String, String>,
}

impl RawLocale {
    fn flatten(self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for section in [
            self.panels,
            self.placeholders,
            self.status,
            self.prompts,
            self.modals,
            self.labels,
            self.instructions,
            self.help_sections,
            self.help_items,
            self.time,
            self.sort,
            self.header,
            self.discovery,
            self.templates,
        ] {
            map.extend(section);
        }
        map
    }
}

fn parse_locale(toml_str: &str) -> HashMap<String, String> {
    match toml::from_str::<RawLocale>(toml_str) {
        Ok(raw) => raw.flatten(),
        Err(e) => {
            log::error!("Failed to parse locale TOML: {e}");
            HashMap::new()
        }
    }
}

macro_rules! get {
    ($map:expr, $fallback:expr, $key:literal) => {
        $map.get($key).cloned().unwrap_or_else(|| {
            $fallback
                .get($key)
                .cloned()
                .expect(concat!("missing key in en locale: ", $key))
        })
    };
}

pub struct Lang {
    // Panel titles
    pub feeds: String,
    pub entries: String,
    pub preview: String,

    // Placeholders
    pub no_title: String,
    pub no_entry_selected: String,
    pub no_feed_selected: String,
    pub all_feeds: String,
    pub uncategorized: String,
    pub no_categories: String,
    pub saved_marker: String,

    // Status
    pub unread_label: String,
    pub refreshing: String,
    pub already_refreshing: String,
    pub no_feeds_to_refresh: String,
    pub opened_in_browser: String,
    pub entry_has_no_url: String,
    pub refresh_thread_crashed: String,

    // Prompts
    pub search_prompt: String,
    pub add_feed_prompt: String,
    pub new_group_name: String,
    pub rename_prompt: String,
    pub delete_feed_confirm: String,
    pub delete_group_confirm: String,

    // Modal titles
    pub search_title: String,
    pub add_feed_title: String,
    pub rename_feed_title: String,
    pub rename_feed_hint: String,
    pub delete_feed_title: String,
    pub new_category: String,
    pub rename_category: String,
    pub category_title: String,
    pub categories_title: String,
    pub help_title: String,

    // Labels
    pub name_label: String,
    pub query_label: String,
    pub url_label: String,
    pub select_category: String,
    pub no_category: String,
    pub new_category_option: String,
    pub filter_all: String,
    pub filter_unread: String,
    pub filter_saved: String,
    pub filter_all_time: String,

    // Instructions
    pub enter_confirm_esc_cancel: String,
    pub y_confirm_n_cancel: String,
    pub group_manage_hint: String,
    pub assign_group_prompt: String,
    pub updated_entries: String,
    pub status_bar_hint: String,

    // Help sections
    pub help_navigation: String,
    pub help_feeds: String,
    pub help_entries: String,
    pub help_general: String,

    // Help items
    pub help_next_link: String,
    pub help_prev_link: String,
    pub help_move_panel: String,
    pub help_move_selection: String,
    pub help_scroll_preview: String,
    pub help_top_bottom: String,
    pub help_resize_panel: String,
    pub help_toggle_layout: String,
    pub help_select_open: String,
    pub help_collapse_category: String,
    pub help_jump_panel: String,
    pub help_back: String,
    pub help_add_feed: String,
    pub help_rename_feed: String,
    pub help_delete_feed: String,
    pub help_refresh_all: String,
    pub help_toggle_unread: String,
    pub help_toggle_saved: String,
    pub help_assign_category: String,
    pub help_manage_categories: String,
    pub help_toggle_read: String,
    pub help_mark_all_read: String,
    pub help_mark_feed_read: String,
    pub help_save_later: String,
    pub help_open_browser: String,
    pub help_search: String,
    pub help_cycle_sort: String,
    pub help_toggle_time: String,
    pub help_toggle_help: String,
    pub help_quit: String,

    // Time
    pub now: String,

    // Sort
    pub sort_label: String,
    pub sort_date_desc: String,
    pub sort_date_asc: String,
    pub sort_title_asc: String,

    // Header
    pub app_name: String,
    pub feed_label: String,
    pub filter_label: String,
    pub search_label: String,

    // Discovery
    pub discovering: String,
    pub select_feed_title: String,
    pub select_feed_prompt: String,

    // Templates (kept private, accessed via methods)
    tpl_minutes_ago: String,
    tpl_hours_ago: String,
    tpl_days_ago: String,
    tpl_feed_saved: String,
    tpl_invalid_url: String,
    tpl_refreshed_summary: String,
    tpl_filter_recent_days: String,
    tpl_no_feed_found: String,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        let locales = embedded_locales();
        let en = parse_locale(locales["en"]);
        let map = if code != "en" {
            locales
                .get(code)
                .map(|toml_str| parse_locale(toml_str))
                .unwrap_or_else(|| en.clone())
        } else {
            en.clone()
        };

        Self {
            feeds: get!(map, en, "feeds"),
            entries: get!(map, en, "entries"),
            preview: get!(map, en, "preview"),

            no_title: get!(map, en, "no_title"),
            no_entry_selected: get!(map, en, "no_entry_selected"),
            no_feed_selected: get!(map, en, "no_feed_selected"),
            all_feeds: get!(map, en, "all_feeds"),
            uncategorized: get!(map, en, "uncategorized"),
            no_categories: get!(map, en, "no_categories"),
            saved_marker: get!(map, en, "saved_marker"),

            unread_label: get!(map, en, "unread_label"),
            refreshing: get!(map, en, "refreshing"),
            already_refreshing: get!(map, en, "already_refreshing"),
            no_feeds_to_refresh: get!(map, en, "no_feeds_to_refresh"),
            opened_in_browser: get!(map, en, "opened_in_browser"),
            entry_has_no_url: get!(map, en, "entry_has_no_url"),
            refresh_thread_crashed: get!(map, en, "refresh_thread_crashed"),

            search_prompt: get!(map, en, "search_prompt"),
            add_feed_prompt: get!(map, en, "add_feed_prompt"),
            new_group_name: get!(map, en, "new_group_name"),
            rename_prompt: get!(map, en, "rename_prompt"),
            delete_feed_confirm: get!(map, en, "delete_feed_confirm"),
            delete_group_confirm: get!(map, en, "delete_group_confirm"),

            search_title: get!(map, en, "search_title"),
            add_feed_title: get!(map, en, "add_feed_title"),
            rename_feed_title: get!(map, en, "rename_feed_title"),
            rename_feed_hint: get!(map, en, "rename_feed_hint"),
            delete_feed_title: get!(map, en, "delete_feed_title"),
            new_category: get!(map, en, "new_category"),
            rename_category: get!(map, en, "rename_category"),
            category_title: get!(map, en, "category_title"),
            categories_title: get!(map, en, "categories_title"),
            help_title: get!(map, en, "help_title"),

            name_label: get!(map, en, "name_label"),
            query_label: get!(map, en, "query_label"),
            url_label: get!(map, en, "url_label"),
            select_category: get!(map, en, "select_category"),
            no_category: get!(map, en, "no_category"),
            new_category_option: get!(map, en, "new_category_option"),
            filter_all: get!(map, en, "filter_all"),
            filter_unread: get!(map, en, "filter_unread"),
            filter_saved: get!(map, en, "filter_saved"),
            filter_all_time: get!(map, en, "filter_all_time"),

            enter_confirm_esc_cancel: get!(map, en, "enter_confirm_esc_cancel"),
            y_confirm_n_cancel: get!(map, en, "y_confirm_n_cancel"),
            group_manage_hint: get!(map, en, "group_manage_hint"),
            assign_group_prompt: get!(map, en, "assign_group_prompt"),
            updated_entries: get!(map, en, "updated_entries"),
            status_bar_hint: get!(map, en, "status_bar_hint"),

            help_navigation: get!(map, en, "help_navigation"),
            help_feeds: get!(map, en, "help_feeds"),
            help_entries: get!(map, en, "help_entries"),
            help_general: get!(map, en, "help_general"),

            help_next_link: get!(map, en, "help_next_link"),
            help_prev_link: get!(map, en, "help_prev_link"),
            help_move_panel: get!(map, en, "help_move_panel"),
            help_move_selection: get!(map, en, "help_move_selection"),
            help_scroll_preview: get!(map, en, "help_scroll_preview"),
            help_top_bottom: get!(map, en, "help_top_bottom"),
            help_resize_panel: get!(map, en, "help_resize_panel"),
            help_toggle_layout: get!(map, en, "help_toggle_layout"),
            help_select_open: get!(map, en, "help_select_open"),
            help_collapse_category: get!(map, en, "help_collapse_category"),
            help_jump_panel: get!(map, en, "help_jump_panel"),
            help_back: get!(map, en, "help_back"),
            help_add_feed: get!(map, en, "help_add_feed"),
            help_rename_feed: get!(map, en, "help_rename_feed"),
            help_delete_feed: get!(map, en, "help_delete_feed"),
            help_refresh_all: get!(map, en, "help_refresh_all"),
            help_toggle_unread: get!(map, en, "help_toggle_unread"),
            help_toggle_saved: get!(map, en, "help_toggle_saved"),
            help_assign_category: get!(map, en, "help_assign_category"),
            help_manage_categories: get!(map, en, "help_manage_categories"),
            help_toggle_read: get!(map, en, "help_toggle_read"),
            help_mark_all_read: get!(map, en, "help_mark_all_read"),
            help_mark_feed_read: get!(map, en, "help_mark_feed_read"),
            help_save_later: get!(map, en, "help_save_later"),
            help_open_browser: get!(map, en, "help_open_browser"),
            help_search: get!(map, en, "help_search"),
            help_cycle_sort: get!(map, en, "help_cycle_sort"),
            help_toggle_time: get!(map, en, "help_toggle_time"),
            help_toggle_help: get!(map, en, "help_toggle_help"),
            help_quit: get!(map, en, "help_quit"),

            now: get!(map, en, "now"),

            sort_label: get!(map, en, "sort_label"),
            sort_date_desc: get!(map, en, "sort_date_desc"),
            sort_date_asc: get!(map, en, "sort_date_asc"),
            sort_title_asc: get!(map, en, "sort_title_asc"),

            app_name: get!(map, en, "app_name"),
            feed_label: get!(map, en, "feed_label"),
            filter_label: get!(map, en, "filter_label"),
            search_label: get!(map, en, "search_label"),

            discovering: get!(map, en, "discovering"),
            select_feed_title: get!(map, en, "select_feed_title"),
            select_feed_prompt: get!(map, en, "select_feed_prompt"),

            tpl_minutes_ago: get!(map, en, "minutes_ago"),
            tpl_hours_ago: get!(map, en, "hours_ago"),
            tpl_days_ago: get!(map, en, "days_ago"),
            tpl_feed_saved: get!(map, en, "feed_saved"),
            tpl_invalid_url: get!(map, en, "invalid_url"),
            tpl_refreshed_summary: get!(map, en, "refreshed_summary"),
            tpl_filter_recent_days: get!(map, en, "filter_recent_days"),
            tpl_no_feed_found: get!(map, en, "no_feed_found"),
        }
    }

    // Dynamic methods using templates

    pub fn minutes_ago(&self, n: i64) -> String {
        self.tpl_minutes_ago.replace("{n}", &n.to_string())
    }

    pub fn hours_ago(&self, n: i64) -> String {
        self.tpl_hours_ago.replace("{n}", &n.to_string())
    }

    pub fn days_ago(&self, n: i64) -> String {
        self.tpl_days_ago.replace("{n}", &n.to_string())
    }

    pub fn feed_saved(&self, url: &str) -> String {
        self.tpl_feed_saved.replace("{url}", url)
    }

    pub fn invalid_url(&self, url: &str) -> String {
        self.tpl_invalid_url.replace("{url}", url)
    }

    pub fn refreshed_summary(&self, feeds: usize, entries: i64, errors: usize) -> String {
        self.tpl_refreshed_summary
            .replace("{feeds}", &feeds.to_string())
            .replace("{entries}", &entries.to_string())
            .replace("{errors}", &errors.to_string())
    }

    pub fn preview_panel_title(&self, current_line: usize, total: usize) -> String {
        format!("{} [{}/{}]", self.preview, current_line, total)
    }

    pub fn filter_recent_days(&self, days: i64) -> String {
        self.tpl_filter_recent_days
            .replace("{days}", &days.to_string())
    }

    pub fn no_feed_found(&self, url: &str) -> String {
        self.tpl_no_feed_found.replace("{url}", url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_code_defaults_to_english() {
        let lang = Lang::from_code("xx");
        assert_eq!(lang.feeds, "FEEDS");
    }

    #[test]
    fn from_code_catalan() {
        let lang = Lang::from_code("ca");
        assert_eq!(lang.feeds, "FONTS");
    }

    #[test]
    fn filter_recent_days_english() {
        let lang = Lang::from_code("en");
        assert_eq!(lang.filter_recent_days(7), "Last 7 days");
        assert_eq!(lang.filter_recent_days(30), "Last 30 days");
    }

    #[test]
    fn filter_recent_days_catalan() {
        let lang = Lang::from_code("ca");
        assert_eq!(lang.filter_recent_days(7), "Últims 7 dies");
    }

    #[test]
    fn time_formatting() {
        let lang = Lang::from_code("en");
        assert_eq!(lang.minutes_ago(5), "5min");
        assert_eq!(lang.hours_ago(2), "2h");
        assert_eq!(lang.days_ago(3), "3d");

        let lang_ca = Lang::from_code("ca");
        assert_eq!(lang_ca.minutes_ago(5), "5 min");
        assert_eq!(lang_ca.hours_ago(2), "2 h");
    }

    #[test]
    fn preview_panel_title_format() {
        let lang = Lang::from_code("en");
        assert_eq!(lang.preview_panel_title(5, 20), "PREVIEW [5/20]");
    }

    #[test]
    fn all_locales_have_same_keys() {
        let en = parse_locale(EN_TOML);
        let mut en_keys: Vec<&String> = en.keys().collect();
        en_keys.sort();

        for (code, toml_str) in embedded_locales() {
            if code == "en" {
                continue;
            }
            let locale = parse_locale(toml_str);
            let mut locale_keys: Vec<&String> = locale.keys().collect();
            locale_keys.sort();

            let missing: Vec<&&String> = en_keys
                .iter()
                .filter(|k| !locale.contains_key(**k))
                .collect();
            let extra: Vec<&&String> = locale_keys
                .iter()
                .filter(|k| !en.contains_key(**k))
                .collect();

            assert!(
                missing.is_empty(),
                "Locale '{code}' is missing keys: {missing:?}"
            );
            assert!(
                extra.is_empty(),
                "Locale '{code}' has extra keys not in en: {extra:?}"
            );
        }
    }

    #[test]
    fn template_methods() {
        let lang = Lang::from_code("en");
        assert_eq!(
            lang.feed_saved("https://x.com"),
            "Feed saved: https://x.com"
        );
        assert_eq!(lang.invalid_url("bad"), "Invalid URL: bad");
        assert_eq!(
            lang.refreshed_summary(3, 42, 1),
            "Refreshed 3 feeds (42 entries, 1 errors)"
        );
        assert_eq!(
            lang.no_feed_found("https://x.com"),
            "No feed found at: https://x.com"
        );

        let lang_ca = Lang::from_code("ca");
        assert_eq!(
            lang_ca.feed_saved("https://x.com"),
            "Font desada: https://x.com"
        );
    }
}
