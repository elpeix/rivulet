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

    // Instructions
    pub enter_confirm_esc_cancel: &'static str,
    pub y_confirm_n_cancel: &'static str,
    pub group_manage_hint: &'static str,
    pub status_bar_hint: &'static str,

    // Time
    pub now: &'static str,
    pub yesterday: &'static str,

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
            uncategorized: "Uncategorized",
            no_categories: "(no categories)",
            saved_marker: "(S) ",

            unread_label: "Unread",
            refreshing: "Refreshing...",
            already_refreshing: "Already refreshing...",
            no_feeds_to_refresh: "No feeds to refresh",
            opened_in_browser: "Opened in browser",
            entry_has_no_url: "Entry has no URL",

            search_prompt: "Search: ",
            add_feed_prompt: "Add feed URL: ",
            new_group_name: "New group name: ",
            rename_prompt: "Rename: ",
            delete_feed_confirm: "Delete feed? (y/N) ",
            delete_group_confirm: "Delete group? (y/N)",

            search_title: "Search",
            add_feed_title: "Add feed",
            delete_feed_title: "Delete feed",
            new_category: "New category",
            rename_category: "Rename category",
            category_title: " Category ",
            categories_title: " Categories ",
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

            enter_confirm_esc_cancel: "Enter to confirm, Esc to cancel",
            y_confirm_n_cancel: "y = confirm, n/Esc = cancel",
            group_manage_hint: "Manage groups: a=add d=delete r=rename Esc=close",
            status_bar_hint: "a add  d del  r refresh  f unread  g saved  s save  c group  C groups  / search  o open  H/L resize  ? help  q quit",

            now: "now",
            yesterday: "yesterday",

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
            uncategorized: "Sense categoria",
            no_categories: "(sense categories)",
            saved_marker: "(D) ",

            unread_label: "No llegits",
            refreshing: "Actualitzant...",
            already_refreshing: "Ja s'està actualitzant...",
            no_feeds_to_refresh: "No hi ha fonts per actualitzar",
            opened_in_browser: "Obert al navegador",
            entry_has_no_url: "L'entrada no té URL",

            search_prompt: "Cerca: ",
            add_feed_prompt: "URL del feed: ",
            new_group_name: "Nom del grup: ",
            rename_prompt: "Reanomena: ",
            delete_feed_confirm: "Eliminar font? (s/N) ",
            delete_group_confirm: "Eliminar grup? (s/N)",

            search_title: "Cerca",
            add_feed_title: "Afegir font",
            delete_feed_title: "Eliminar font",
            new_category: "Nova categoria",
            rename_category: "Reanomenar categoria",
            category_title: " Categoria ",
            categories_title: " Categories ",
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

            enter_confirm_esc_cancel: "Enter per confirmar, Esc per cancel·lar",
            y_confirm_n_cancel: "s = confirmar, n/Esc = cancel·lar",
            group_manage_hint: "Gestionar grups: a=afegir d=eliminar r=reanomenar Esc=tancar",
            status_bar_hint: "a afegir  d elim  r actual  f no llegits  g desats  s desar  c grup  C grups  / cerca  o obrir  H/L mida  ? ajuda  q sortir",

            now: "ara",
            yesterday: "ahir",

            app_name: "Rivulet",
            feed_label: "Font: ",
            focus_label: "Focus: ",
            filter_label: "Filtre: ",
            search_label: "Cerca: ",
        }
    }

    // Dynamic methods

    pub fn minutes_ago(&self, n: i64) -> String {
        format!("{}m ago", n)
    }

    pub fn hours_ago(&self, n: i64) -> String {
        format!("{}h ago", n)
    }

    pub fn days_ago(&self, n: i64) -> String {
        format!("{}d ago", n)
    }

    pub fn weeks_ago(&self, n: i64) -> String {
        format!("{}w ago", n)
    }

    pub fn feed_saved(&self, url: &str) -> String {
        match self.code {
            "ca" => format!("Font desada: {}", url),
            _ => format!("Feed saved: {}", url),
        }
    }

    pub fn invalid_url(&self, url: &str) -> String {
        match self.code {
            "ca" => format!("URL no vàlida: {}", url),
            _ => format!("Invalid URL: {}", url),
        }
    }

    pub fn refreshed_summary(&self, feeds: usize, entries: i64, errors: usize) -> String {
        match self.code {
            "ca" => format!(
                "Actualitzades {} fonts ({} entrades, {} errors)",
                feeds, entries, errors
            ),
            _ => format!(
                "Refreshed {} feeds ({} entries, {} errors)",
                feeds, entries, errors
            ),
        }
    }

    pub fn entries_panel_title(&self, shown: usize, total: usize) -> String {
        if total == 0 {
            self.entries.to_string()
        } else if shown < total {
            format!("{} ({}/{})", self.entries, shown, total)
        } else {
            format!("{} ({})", self.entries, total)
        }
    }

    pub fn preview_panel_title(&self, current_line: usize, total: usize) -> String {
        format!("{} [{}/{}]", self.preview, current_line, total)
    }

}
