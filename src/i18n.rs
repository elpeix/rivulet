pub struct Lang {
    code: &'static str,

    // Panel titles
    pub feeds: &'static str,
    pub entries: &'static str,
    pub preview: &'static str,

    // Placeholders
    pub no_title: &'static str,
    pub no_entry_selected: &'static str,
    pub no_feed_selected: &'static str,
    pub all_feeds: &'static str,
    pub uncategorized: &'static str,
    pub no_categories: &'static str,
    pub saved_marker: &'static str,

    // Status
    pub unread_label: &'static str,
    pub refreshing: &'static str,
    pub already_refreshing: &'static str,
    pub no_feeds_to_refresh: &'static str,
    pub opened_in_browser: &'static str,
    pub entry_has_no_url: &'static str,
    pub refresh_thread_crashed: &'static str,

    // Prompts
    pub search_prompt: &'static str,
    pub add_feed_prompt: &'static str,
    pub new_group_name: &'static str,
    pub rename_prompt: &'static str,
    pub delete_feed_confirm: &'static str,
    pub delete_group_confirm: &'static str,

    // Modal titles
    pub search_title: &'static str,
    pub add_feed_title: &'static str,
    pub rename_feed_title: &'static str,
    pub rename_feed_hint: &'static str,
    pub delete_feed_title: &'static str,
    pub new_category: &'static str,
    pub rename_category: &'static str,
    pub category_title: &'static str,
    pub categories_title: &'static str,
    pub help_title: &'static str,

    // Labels
    pub name_label: &'static str,
    pub query_label: &'static str,
    pub url_label: &'static str,
    pub select_category: &'static str,
    pub no_category: &'static str,
    pub new_category_option: &'static str,
    pub filter_all: &'static str,
    pub filter_unread: &'static str,
    pub filter_saved: &'static str,
    pub filter_all_time: &'static str,

    // Instructions
    pub enter_confirm_esc_cancel: &'static str,
    pub y_confirm_n_cancel: &'static str,
    pub group_manage_hint: &'static str,
    pub assign_group_prompt: &'static str,
    pub updated_entries: &'static str,
    pub status_bar_hint: &'static str,

    // Help sections
    pub help_navigation: &'static str,
    pub help_feeds: &'static str,
    pub help_entries: &'static str,
    pub help_general: &'static str,

    // Help items
    pub help_next_link: &'static str,
    pub help_prev_link: &'static str,
    pub help_move_panel: &'static str,
    pub help_move_selection: &'static str,
    pub help_scroll_preview: &'static str,
    pub help_top_bottom: &'static str,
    pub help_resize_panel: &'static str,
    pub help_select_open: &'static str,
    pub help_collapse_category: &'static str,
    pub help_back: &'static str,
    pub help_add_feed: &'static str,
    pub help_rename_feed: &'static str,
    pub help_delete_feed: &'static str,
    pub help_refresh_all: &'static str,
    pub help_toggle_unread: &'static str,
    pub help_toggle_saved: &'static str,
    pub help_assign_category: &'static str,
    pub help_manage_categories: &'static str,
    pub help_toggle_read: &'static str,
    pub help_mark_all_read: &'static str,
    pub help_mark_feed_read: &'static str,
    pub help_save_later: &'static str,
    pub help_open_browser: &'static str,
    pub help_search: &'static str,
    pub help_cycle_sort: &'static str,
    pub help_toggle_time: &'static str,
    pub help_toggle_help: &'static str,
    pub help_quit: &'static str,

    // Time
    pub now: &'static str,

    // Sort
    pub sort_label: &'static str,
    pub sort_date_desc: &'static str,
    pub sort_date_asc: &'static str,
    pub sort_title_asc: &'static str,

    // Header
    pub app_name: &'static str,
    pub feed_label: &'static str,
    pub focus_label: &'static str,
    pub filter_label: &'static str,
    pub search_label: &'static str,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "ca" => Self::catalan(),
            _ => Self::english(),
        }
    }

    fn english() -> Self {
        Self {
            code: "en",
            feeds: "FEEDS",
            entries: "ENTRIES",
            preview: "PREVIEW",

            no_title: "(no title)",
            no_entry_selected: "No entry selected",
            no_feed_selected: "No feed selected",
            all_feeds: "All",
            uncategorized: "Uncategorized",
            no_categories: "(no categories)",
            saved_marker: "◆ ",

            unread_label: "Unread",
            refreshing: "Refreshing...",
            already_refreshing: "Already refreshing...",
            no_feeds_to_refresh: "No feeds to refresh",
            opened_in_browser: "Opened in browser",
            entry_has_no_url: "Entry has no URL",
            refresh_thread_crashed: "Refresh failed unexpectedly",

            search_prompt: "Search: ",
            add_feed_prompt: "Add feed URL: ",
            new_group_name: "New category name: ",
            rename_prompt: "Rename: ",
            delete_feed_confirm: "Delete feed? (y/N) ",
            delete_group_confirm: "Delete category? (y/N)",

            search_title: "Search",
            add_feed_title: "Add feed",
            rename_feed_title: "Rename feed",
            rename_feed_hint: "Leave empty to restore original name",
            delete_feed_title: "Delete feed",
            new_category: "New category",
            rename_category: "Rename category",
            category_title: " Assign category ",
            categories_title: " Manage categories ",
            help_title: " HELP ",

            name_label: "Name:",
            query_label: "Query",
            url_label: "URL",
            select_category: "Select category:",
            no_category: "No category",
            new_category_option: "New category...",
            filter_all: "All",
            filter_unread: "Unread",
            filter_saved: "Saved",
            filter_all_time: "All time",

            enter_confirm_esc_cancel: "Enter to confirm, Esc to cancel",
            y_confirm_n_cancel: "y = confirm, n/Esc = cancel",
            group_manage_hint: "a add  d delete  r rename  J/K reorder  Esc close",
            assign_group_prompt: "Assign group...",
            updated_entries: "Updated entries",
            status_bar_hint: "? help  q quit",

            help_navigation: "Navigation",
            help_feeds: "Feeds",
            help_entries: "Entries",
            help_general: "General",
            help_next_link: "Next link",
            help_prev_link: "Previous link",
            help_move_panel: "Move panel",
            help_move_selection: "Move selection",
            help_scroll_preview: "Scroll preview",
            help_top_bottom: "Top / bottom",
            help_resize_panel: "Resize panel",
            help_select_open: "Select / open",
            help_collapse_category: "Collapse category",
            help_back: "Back",
            help_add_feed: "Add feed",
            help_rename_feed: "Rename feed",
            help_delete_feed: "Delete feed",
            help_refresh_all: "Refresh all",
            help_toggle_unread: "Toggle unread",
            help_toggle_saved: "Toggle saved",
            help_assign_category: "Assign category",
            help_manage_categories: "Manage categories (J/K reorder)",
            help_toggle_read: "Toggle read",
            help_mark_all_read: "Mark all read",
            help_mark_feed_read: "Mark feed read",
            help_save_later: "Save for later",
            help_open_browser: "Open in browser",
            help_search: "Search",
            help_cycle_sort: "Cycle sort",
            help_toggle_time: "Toggle time filter",
            help_toggle_help: "Toggle help",
            help_quit: "Quit",

            sort_label: "Sort",
            sort_date_desc: "Newest first",
            sort_date_asc: "Oldest first",
            sort_title_asc: "Title A-Z",

            now: "now",

            app_name: "Rivulet",
            feed_label: "Feed: ",
            focus_label: "Focus: ",
            filter_label: "Filter: ",
            search_label: "Search: ",
        }
    }

    fn catalan() -> Self {
        Self {
            code: "ca",
            feeds: "FONTS",
            entries: "ENTRADES",
            preview: "PREVISUALITZACIÓ",

            no_title: "(sense títol)",
            no_entry_selected: "Cap entrada seleccionada",
            no_feed_selected: "Cap font seleccionada",
            all_feeds: "Tot",
            uncategorized: "Sense categoria",
            no_categories: "(sense categories)",
            saved_marker: "◆ ",

            unread_label: "No llegits",
            refreshing: "Actualitzant...",
            already_refreshing: "Ja s'està actualitzant...",
            no_feeds_to_refresh: "No hi ha fonts per actualitzar",
            opened_in_browser: "Obert al navegador",
            entry_has_no_url: "L'entrada no té URL",
            refresh_thread_crashed: "L'actualització ha fallat inesperadament",

            search_prompt: "Cerca: ",
            add_feed_prompt: "URL del feed: ",
            new_group_name: "Nom de la categoria: ",
            rename_prompt: "Reanomena: ",
            delete_feed_confirm: "Elimina font? (s/N) ",
            delete_group_confirm: "Elimina categoria? (s/N)",

            search_title: "Cerca",
            add_feed_title: "Afegeix font",
            rename_feed_title: "Reanomena font",
            rename_feed_hint: "Deixa en blanc per restaurar el nom original",
            delete_feed_title: "Elimina font",
            new_category: "Nova categoria",
            rename_category: "Reanomena categoria",
            category_title: " Assigna categoria ",
            categories_title: " Gestiona categories ",
            help_title: " AJUDA ",

            name_label: "Nom:",
            query_label: "Consulta",
            url_label: "URL",
            select_category: "Selecciona categoria:",
            no_category: "Sense categoria",
            new_category_option: "Nova categoria...",
            filter_all: "Tot",
            filter_unread: "No llegits",
            filter_saved: "Desats",
            filter_all_time: "Tot el temps",

            enter_confirm_esc_cancel: "Enter per confirmar, Esc per cancel·lar",
            y_confirm_n_cancel: "s = confirma, n/Esc = cancel·la",
            group_manage_hint: "a afegeix  d elimina  r reanomena  J/K ordena  Esc tanca",
            assign_group_prompt: "Assigna grup...",
            updated_entries: "Entrades actualitzades",
            status_bar_hint: "? ajuda  q surt",

            help_navigation: "Navegació",
            help_feeds: "Fonts",
            help_entries: "Entrades",
            help_general: "General",
            help_next_link: "Següent enllaç",
            help_prev_link: "Anterior enllaç",
            help_move_panel: "Mou panell",
            help_move_selection: "Mou selecció",
            help_scroll_preview: "Desplaça previsualització",
            help_top_bottom: "Inici / final",
            help_resize_panel: "Redimensiona panell",
            help_select_open: "Selecciona / obre",
            help_collapse_category: "Plega categoria",
            help_back: "Enrere",
            help_add_feed: "Afegeix font",
            help_rename_feed: "Reanomena font",
            help_delete_feed: "Elimina font",
            help_refresh_all: "Actualitza tot",
            help_toggle_unread: "Filtre no llegits",
            help_toggle_saved: "Filtre desats",
            help_assign_category: "Assigna categoria",
            help_manage_categories: "Gestiona categories (J/K ordena)",
            help_toggle_read: "Marca llegit/no llegit",
            help_mark_all_read: "Marca tot com a llegit",
            help_mark_feed_read: "Marca font com a llegida",
            help_save_later: "Desa per més tard",
            help_open_browser: "Obre al navegador",
            help_search: "Cerca",
            help_cycle_sort: "Canvia ordenació",
            help_toggle_time: "Filtre temporal",
            help_toggle_help: "Mostra ajuda",
            help_quit: "Surt",

            sort_label: "Ordenació",
            sort_date_desc: "Més recents",
            sort_date_asc: "Més antics",
            sort_title_asc: "Títol A-Z",

            now: "ara",

            app_name: "Rivulet",
            feed_label: "Font: ",
            focus_label: "Focus: ",
            filter_label: "Filtre: ",
            search_label: "Cerca: ",
        }
    }

    // Dynamic methods

    pub fn minutes_ago(&self, n: i64) -> String {
        match self.code {
            "ca" => format!("{n} min"),
            _ => format!("{n}min"),
        }
    }

    pub fn hours_ago(&self, n: i64) -> String {
        match self.code {
            "ca" => format!("{n} h"),
            _ => format!("{n}h"),
        }
    }

    pub fn days_ago(&self, n: i64) -> String {
        match self.code {
            "ca" => format!("{n} d"),
            _ => format!("{n}d"),
        }
    }

    pub fn feed_saved(&self, url: &str) -> String {
        match self.code {
            "ca" => format!("Font desada: {url}"),
            _ => format!("Feed saved: {url}"),
        }
    }

    pub fn invalid_url(&self, url: &str) -> String {
        match self.code {
            "ca" => format!("URL no vàlida: {url}"),
            _ => format!("Invalid URL: {url}"),
        }
    }

    pub fn refreshed_summary(&self, feeds: usize, entries: i64, errors: usize) -> String {
        match self.code {
            "ca" => format!("Actualitzades {feeds} fonts ({entries} entrades, {errors} errors)"),
            _ => format!("Refreshed {feeds} feeds ({entries} entries, {errors} errors)"),
        }
    }

    pub fn preview_panel_title(&self, current_line: usize, total: usize) -> String {
        format!("{} [{}/{}]", self.preview, current_line, total)
    }

    pub fn filter_recent_days(&self, days: i64) -> String {
        match self.code {
            "ca" => format!("Últims {days} dies"),
            _ => format!("Last {days} days"),
        }
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
}
