//! Ornagai English-to-Myanmar Dictionary module for HtetWord.
//!
//! Provides instant lookup, auto-complete, definition rendering, and insertion
//! into the active document for over 111,000 words.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::OnceLock;

use egui::{Color32, Context, RichText, ScrollArea, Ui, Vec2};
use serde_json::Value;

use craft::ui::contract::ShowContainer as _;
use craft::ui::dialog::{Dialog, DialogId, DialogResult, DialogStack, StackDialog, dialog_id};
use craft::ui::sections::{DialogRole, FooterButton};

use crate::WordApp;
use crate::dialogs::host::{DialogHooks, Settle, open_with};

/// An entry in the StarDict index table.
#[derive(Clone, Debug)]
pub struct StarDictEntry {
    pub word: String,
    pub lower: String,
    pub offset: u32,
    pub size: u32,
}

/// Loaded StarDict index and dictionary file path.
pub struct StarDict {
    pub dict_path: PathBuf,
    pub entries: Vec<StarDictEntry>,
}

impl StarDict {
    /// Attempts to locate and load the StarDict dictionary files.
    pub fn load() -> Option<Self> {
        let dir = find_dict_dir()?;
        let idx_path = dir.join("ornagai-en-my.idx");
        let dict_path = dir.join("ornagai-en-my.dict");

        if !idx_path.exists() || !dict_path.exists() {
            return None;
        }

        let idx_bytes = std::fs::read(&idx_path).ok()?;
        let mut entries = Vec::with_capacity(112_000);
        let mut pos = 0;
        let len = idx_bytes.len();

        while pos < len {
            // Find null terminator for word
            let mut null_pos = pos;
            while null_pos < len && idx_bytes[null_pos] != 0 {
                null_pos += 1;
            }
            if null_pos >= len || null_pos + 9 > len {
                break;
            }

            let word = String::from_utf8_lossy(&idx_bytes[pos..null_pos]).into_owned();
            let lower = word.to_lowercase();

            let off_bytes = [idx_bytes[null_pos + 1], idx_bytes[null_pos + 2], idx_bytes[null_pos + 3], idx_bytes[null_pos + 4]];
            let size_bytes = [idx_bytes[null_pos + 5], idx_bytes[null_pos + 6], idx_bytes[null_pos + 7], idx_bytes[null_pos + 8]];

            let offset = u32::from_be_bytes(off_bytes);
            let size = u32::from_be_bytes(size_bytes);

            entries.push(StarDictEntry { word, lower, offset, size });

            pos = null_pos + 9;
        }

        // Sort by lower for fast binary search
        entries.sort_by(|a, b| a.lower.cmp(&b.lower));

        Some(Self { dict_path, entries })
    }

    /// Searches for words matching or prefixed by `query`.
    pub fn search(&self, query: &str, limit: usize) -> Vec<StarDictEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let idx = match self.entries.binary_search_by(|e| e.lower.cmp(&q)) {
            Ok(i) => i,
            Err(i) => i,
        };

        let mut results = Vec::new();
        let mut i = idx;
        while i < self.entries.len() && results.len() < limit {
            if self.entries[i].lower.starts_with(&q) {
                results.push(self.entries[i].clone());
            } else if !self.entries[i].lower.is_empty() && self.entries[i].lower > q {
                break;
            }
            i += 1;
        }

        // If prefix search had few results and user typed a substring, fallback search
        if results.is_empty() && q.len() >= 3 {
            for entry in &self.entries {
                if entry.lower.contains(&q) {
                    results.push(entry.clone());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    /// Reads raw definition for an entry from the .dict file.
    pub fn read_definition(&self, entry: &StarDictEntry) -> Option<String> {
        let mut file = File::open(&self.dict_path).ok()?;
        file.seek(SeekFrom::Start(u64::from(entry.offset))).ok()?;
        let mut buf = vec![0u8; entry.size as usize];
        file.read_exact(&mut buf).ok()?;
        String::from_utf8(buf).ok()
    }
}

/// Global cached StarDict instance.
static DICT: OnceLock<Option<StarDict>> = OnceLock::new();

pub fn get_dict() -> Option<&'static StarDict> {
    DICT.get_or_init(StarDict::load).as_ref()
}

/// Finds the dictionary assets folder across typical installation paths.
fn find_dict_dir() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("assets/dictionary/ornagai-en-my"),
        PathBuf::from("../assets/dictionary/ornagai-en-my"),
        PathBuf::from("../../assets/dictionary/ornagai-en-my"),
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("assets/dictionary/ornagai-en-my"));
            candidates.push(parent.join("dictionary/ornagai-en-my"));
            if let Some(contents) = parent.parent() {
                candidates.push(contents.join("Resources/dictionary/ornagai-en-my"));
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home);
        candidates.push(p.join(".config/wordcraft/dictionary/ornagai-en-my"));
        candidates.push(p.join(".config/htetword/dictionary/ornagai-en-my"));
        candidates.push(p.join("Library/Application Support/WordCraft/dictionary/ornagai-en-my"));
        candidates.push(p.join("Library/Application Support/HtetWord/dictionary/ornagai-en-my"));
    }

    for c in candidates {
        if c.join("ornagai-en-my.idx").exists() {
            return Some(c);
        }
    }
    None
}

/// Formatted dictionary entry definition.
#[derive(Clone, Debug, Default)]
pub struct ParsedDefinition {
    pub headword: String,
    pub pos: String,
    pub ipa: String,
    pub phonetics_my: String,
    pub myanmar_meanings: Vec<String>,
    pub examples: Vec<String>,
    pub plain_text: String,
}

/// Parses HTML-tagged Ornagai StarDict definitions.
pub fn parse_ornagai_entry(headword: &str, html: &str) -> ParsedDefinition {
    let mut parsed = ParsedDefinition { headword: headword.to_string(), ..Default::default() };

    // Normalize breaks
    let normalized = html.replace("<br/>", "\n").replace("<br />", "\n").replace("<br>", "\n");

    let mut lines_out = Vec::new();

    for line in normalized.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for <b>pos</b>
        if trimmed.starts_with("<b>") {
            if let Some(end_b) = trimmed.find("</b>") {
                parsed.pos = trimmed[3..end_b].trim().to_string();
                let rest = &trimmed[end_b + 4..];

                if let Some(start_i) = rest.find("<i>") {
                    if let Some(end_i) = rest.find("</i>") {
                        parsed.ipa = rest[start_i + 3..end_i].trim().to_string();
                    }
                }
                if let Some(start_brk) = rest.find('[') {
                    if let Some(end_brk) = rest.find(']') {
                        parsed.phonetics_my = rest[start_brk + 1..end_brk].trim().to_string();
                    }
                }
            }
            continue;
        }

        // Example sentences usually begin with <i>•
        if trimmed.starts_with("<i>•") || trimmed.starts_with("•") {
            let clean = trimmed.replace("<i>", "").replace("</i>", "").replace("<b>", "").replace("</b>", "");
            parsed.examples.push(clean.trim().to_string());
            continue;
        }

        // Burmese meanings
        let clean = trimmed.replace("<i>", "").replace("</i>", "").replace("<b>", "").replace("</b>", "");
        if !clean.trim().is_empty() {
            parsed.myanmar_meanings.push(clean.trim().to_string());
        }
    }

    // Build plain text version for insertion
    lines_out.push(format!("{} [{}]", parsed.headword, parsed.pos));
    if !parsed.phonetics_my.is_empty() {
        lines_out.push(format!("အသံထွက်: {} {}", parsed.ipa, parsed.phonetics_my));
    }
    for m in &parsed.myanmar_meanings {
        lines_out.push(m.clone());
    }
    for e in &parsed.examples {
        lines_out.push(e.clone());
    }

    parsed.plain_text = lines_out.join("\n");
    parsed
}

/// The English-Myanmar Dictionary Dialog.
pub struct DictionaryDialog {
    pub search_query: String,
    pub matches: Vec<StarDictEntry>,
    pub selected_entry: Option<StarDictEntry>,
    pub current_definition: Option<ParsedDefinition>,
    pub text_to_insert: Option<String>,
    pub status_message: Option<String>,
}

impl DictionaryDialog {
    pub fn new() -> Self {
        let mut d = Self {
            search_query: String::new(),
            matches: Vec::new(),
            selected_entry: None,
            current_definition: None,
            text_to_insert: None,
            status_message: None,
        };
        // Initial popular search or default word
        d.search("welcome");
        d
    }

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        if let Some(dict) = get_dict() {
            self.matches = dict.search(query, 30);
            if let Some(first) = self.matches.first().cloned() {
                self.select_entry(first);
            } else {
                self.selected_entry = None;
                self.current_definition = None;
            }
        } else {
            self.status_message = Some("အဘိဓာန်ဒေတာဖိုင် ရှာမတွေ့ပါ (Dictionary files not found)".to_string());
        }
    }

    pub fn select_entry(&mut self, entry: StarDictEntry) {
        if let Some(dict) = get_dict() {
            if let Some(raw) = dict.read_definition(&entry) {
                let parsed = parse_ornagai_entry(&entry.word, &raw);
                self.current_definition = Some(parsed);
            }
        }
        self.selected_entry = Some(entry);
    }

    pub fn take_insert(&mut self) -> Option<String> {
        self.text_to_insert.take()
    }
}

impl Default for DictionaryDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl StackDialog for DictionaryDialog {
    type Result = ();

    fn id(&self) -> DialogId {
        dialog_id("dictionary")
    }

    fn key(&self) -> Option<&str> {
        Some("dictionary")
    }

    fn show(&mut self, ui: &mut Ui, _stack: &DialogStack) -> Option<DialogResult<()>> {
        let mut to_select = None;
        let mut query_changed = false;
        let mut insert_clicked = false;

        let shown = Dialog::new(tl!("Ornagai English-Myanmar Dictionary").to_string())
            .buttons([FooterButton::new(tl!("Close").to_string(), DialogRole::Accept)])
            .show(ui, |ui, _| {
                ui.set_min_width(520.0);
                ui.set_min_height(380.0);

                // Search Bar Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍 ရှာဖွေရန်:").color(Color32::from_rgb(180, 70, 110)).strong());
                    let response = ui.add_sized(
                        Vec2::new(260.0, 26.0),
                        egui::TextEdit::singleline(&mut self.search_query).hint_text("English word (e.g. peace, book, article)"),
                    );
                    if response.changed() {
                        query_changed = true;
                    }

                    if ui.button("✖").clicked() {
                        self.search_query.clear();
                        query_changed = true;
                    }

                    if let Some(dict) = get_dict() {
                        let total = dict.entries.len();
                        ui.label(RichText::new(format!("({} စကားလုံး)", total)).small().color(Color32::GRAY));
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                if let Some(status) = &self.status_message {
                    ui.colored_label(Color32::RED, status);
                }

                // Two columns: Left = Word list, Right = Definition details
                ui.horizontal(|ui| {
                    // Left list column
                    ui.vertical(|ui| {
                        ui.set_width(170.0);
                        ui.label(RichText::new(format!("ကိုက်ညီမှု ({})", self.matches.len())).small().color(Color32::DARK_GRAY));

                        ScrollArea::vertical().id_salt("dict_matches_scroll").max_height(300.0).show(ui, |ui| {
                            for entry in &self.matches {
                                let is_selected = self.selected_entry.as_ref().is_some_and(|s| s.word == entry.word);
                                let label =
                                    RichText::new(&entry.word).color(if is_selected { Color32::from_rgb(190, 50, 100) } else { Color32::BLACK });

                                let resp = ui.selectable_label(is_selected, label);
                                if resp.clicked() {
                                    to_select = Some(entry.clone());
                                }
                            }
                        });
                    });

                    ui.separator();

                    // Right definition column
                    ui.vertical(|ui| {
                        ui.set_min_width(320.0);

                        if let Some(def) = &self.current_definition {
                            ScrollArea::vertical().id_salt("dict_def_scroll").max_height(280.0).show(ui, |ui| {
                                // Word Header with Liquid Glass styling
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new(&def.headword).size(20.0).strong().color(Color32::from_rgb(160, 40, 90)));
                                    if !def.pos.is_empty() {
                                        ui.label(RichText::new(format!("[{}]", def.pos)).color(Color32::from_rgb(120, 70, 140)).italics());
                                    }
                                    if !def.ipa.is_empty() {
                                        ui.label(RichText::new(&def.ipa).color(Color32::from_rgb(100, 100, 120)));
                                    }
                                    if !def.phonetics_my.is_empty() {
                                        ui.label(RichText::new(format!("[{}]", def.phonetics_my)).color(Color32::from_rgb(180, 50, 80)));
                                    }
                                });

                                ui.add_space(6.0);

                                // Meanings in Myanmar
                                ui.label(RichText::new("အဓိပ္ပာယ်:").small().strong().color(Color32::from_rgb(140, 50, 90)));
                                for meaning in &def.myanmar_meanings {
                                    ui.label(RichText::new(meaning).size(14.0).color(Color32::from_rgb(30, 30, 40)));
                                }

                                // Examples
                                if !def.examples.is_empty() {
                                    ui.add_space(8.0);
                                    ui.label(RichText::new("ဥပမာ သုံးစွဲပုံ:").small().strong().color(Color32::from_rgb(100, 70, 130)));
                                    for ex in &def.examples {
                                        ui.label(RichText::new(ex).italics().color(Color32::from_rgb(60, 60, 70)));
                                    }
                                }
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(4.0);

                            // Insert into document action button
                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("📄 စာသားထဲသို့ ထည့်မည် (Insert)").color(Color32::from_rgb(160, 40, 90))).clicked()
                                {
                                    insert_clicked = true;
                                }

                                if ui.button("📋 ကူးယူမည် (Copy)").clicked() {
                                    ui.ctx().copy_text(def.plain_text.clone());
                                }
                            });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(RichText::new("စကားလုံး ရှာဖွေပါ").size(15.0).color(Color32::GRAY));
                                ui.label(RichText::new("Ornagai အဘိဓာန်တွင် စကားလုံး ၁၁၁,၂၃၂ လုံး ပါဝင်ပါသည်").small().color(Color32::LIGHT_GRAY));
                            });
                        }
                    });
                });
            });

        if let Some(selected) = to_select {
            self.select_entry(selected);
        }

        if query_changed {
            let q = self.search_query.clone();
            self.search(&q);
        }

        if insert_clicked {
            if let Some(def) = &self.current_definition {
                self.text_to_insert = Some(def.plain_text.clone());
            }
        }

        shown.output.closed.map(|_| DialogResult::Ok(()))
    }
}

/// Opens the Dictionary dialog from commands or UI actions.
pub(crate) fn open_dialog(app: &mut WordApp, _params: &Value, parent: Option<DialogId>) -> Option<DialogId> {
    let dialog = DictionaryDialog::new();
    Some(open_with(app, parent, dialog, DialogHooks { prepare: None, settle: dictionary_settle }))
}

/// Settle hook: takes text to insert and pastes it directly at the document caret.
fn dictionary_settle(app: &mut WordApp, _ctx: &Context, id: DialogId) -> Settle {
    if let Some(dialog) = app.dialog_host.get_mut_by_id::<DictionaryDialog>(id) {
        if let Some(text) = dialog.take_insert() {
            let _ = wordcraft_engine::cmd::paste_text(&mut app.session, &text);
        }
    }

    if app.dialog_host.take_by_id::<DictionaryDialog>(id).is_some() { Settle::Done } else { Settle::Keep }
}
