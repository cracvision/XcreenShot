//! What the Library view shows (LIB-004 to LIB-007, LIB-009, LIB-011): the items that
//! pass the filters and the search, in order, and how many items each filter value
//! would show.
//!
//! Filters combine as the spec says: values of one section add up (OR), sections cross
//! (AND), and the search crosses everything. Each section's counts are taken over the
//! items the other sections and the search let through.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::index::Item;

/// How the list is ordered (LIB-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[default]
    Modified,
    Captured,
    Name,
    Size,
    Type,
    App,
}

/// The filters of the left pane (LIB-005, LIB-006, LIB-009, LIB-011).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Filters {
    /// image, video, gif, guide.
    pub types: Vec<String>,
    /// `2026`, `2026-11`, `2026-11-12`, or today, yesterday, last7, month.
    pub dates: Vec<String>,
    /// Application names; "" is Unknown.
    pub apps: Vec<String>,
    /// Domains; "" is No website.
    pub sites: Vec<String>,
    pub tags: Vec<String>,
    pub favorites: bool,
    /// A range of capture days, `YYYY-MM-DD`, both included (LIB-034).
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
            && self.dates.is_empty()
            && self.apps.is_empty()
            && self.sites.is_empty()
            && self.tags.is_empty()
            && !self.favorites
    }
}

/// Lower case, without accents: "Camión" and "camion" are the same (LIB-007).
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
            'é' | 'è' | 'ë' | 'ê' | 'ē' | 'ė' | 'ę' | 'ě' => 'e',
            'í' | 'ì' | 'ï' | 'î' | 'ī' | 'į' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => 'o',
            'ú' | 'ù' | 'ü' | 'û' | 'ū' | 'ů' | 'ű' | 'ų' => 'u',
            'ý' | 'ÿ' => 'y',
            'ñ' | 'ń' | 'ň' => 'n',
            'ç' | 'ć' | 'č' => 'c',
            'š' | 'ś' => 's',
            'ž' | 'ź' | 'ż' => 'z',
            'ř' => 'r',
            'ď' => 'd',
            'ť' => 't',
            'ł' => 'l',
            'ğ' => 'g',
            other => other,
        })
        .collect()
}

/// A domain without "www.", from a URL (LIB-006).
pub fn domain(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next()?.rsplit('@').next()?;
    let host = host.split(':').next()?.trim().to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();
    (!host.is_empty() && host.contains('.')).then_some(host)
}

/// An application's name without its version (LIB-006: versions group by name).
pub fn app_name(app: &str) -> String {
    let words: Vec<&str> = app
        .split_whitespace()
        .take_while(|w| !w.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .collect();
    if words.is_empty() {
        app.trim().to_string()
    } else {
        words.join(" ")
    }
}

/// The type of an item for the Type filter.
pub fn kind(item: &Item) -> &'static str {
    // A guide of steps is a .xshot with steps (SMT-035).
    if item.steps.is_some() {
        return "guide";
    }
    match item.format.as_str() {
        "mp4" | "m4v" | "mov" | "webm" => "video",
        "gif" => "gif",
        "xguide" => "guide",
        _ => "image",
    }
}

/// One search term (LIB-007).
#[derive(Debug, Clone, PartialEq)]
enum Term {
    /// A word that some word of any field starts with.
    Word(String),
    /// Words in this order, in one field.
    Phrase(String),
    Tag(String),
    App(String),
    Site(String),
    Type(String),
}

fn parse(query: &str) -> Vec<Term> {
    let mut terms = Vec::new();
    let mut rest = query.trim();
    while !rest.is_empty() {
        let (token, next) = if let Some(after) = rest.strip_prefix('"') {
            match after.find('"') {
                Some(end) => (format!("\"{}", &after[..end]), &after[end + 1..]),
                None => (format!("\"{after}"), ""),
            }
        } else {
            // A field prefix may be followed by a quoted value: tag:"cliente a".
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let word = &rest[..end];
            match word.split_once(':') {
                Some((_, v)) if v.starts_with('"') => {
                    let start = word.len() - v.len() + 1;
                    match rest[start..].find('"') {
                        Some(close) => (rest[..start + close].to_string(), &rest[start + close + 1..]),
                        None => (rest.to_string(), ""),
                    }
                }
                _ => (word.to_string(), &rest[end..]),
            }
        };
        rest = next.trim_start();
        let token = token.replace('"', "");
        let t = fold(&token);
        let field = |prefix: &str| t.strip_prefix(prefix).map(|v| v.trim().to_string());
        let term = if let Some(v) = field("tag:") {
            Term::Tag(v)
        } else if let Some(v) = field("app:") {
            Term::App(v)
        } else if let Some(v) = field("site:") {
            Term::Site(v)
        } else if let Some(v) = field("type:") {
            Term::Type(v)
        } else if t.contains(char::is_whitespace) {
            Term::Phrase(t.trim().to_string())
        } else {
            Term::Word(t)
        };
        let empty = matches!(&term, Term::Word(v) | Term::Phrase(v) | Term::Tag(v) | Term::App(v) | Term::Site(v) | Term::Type(v) if v.is_empty());
        if !empty {
            terms.push(term);
        }
    }
    terms
}

/// What the search reads of an item, folded.
struct Fields {
    name: String,
    tags: Vec<String>,
    app: String,
    site: String,
    text: String,
}

impl Fields {
    fn of(item: &Item) -> Fields {
        Fields {
            name: fold(&item.name),
            tags: item.tags.iter().map(|t| fold(t)).collect(),
            app: fold(item.app.as_deref().unwrap_or("")),
            site: fold(
                &[
                    item.url.as_deref().and_then(domain).unwrap_or_default(),
                    item.url.clone().unwrap_or_default(),
                ]
                .join(" "),
            ),
            text: fold(&item.text),
        }
    }

    fn all(&self) -> impl Iterator<Item = &str> {
        [
            self.name.as_str(),
            self.app.as_str(),
            self.site.as_str(),
            self.text.as_str(),
        ]
        .into_iter()
        .chain(self.tags.iter().map(String::as_str))
    }
}

/// Whether some word of `field` starts with `prefix`.
fn word_starts(field: &str, prefix: &str) -> bool {
    field
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w.starts_with(prefix))
        || (prefix.contains(|c: char| !c.is_alphanumeric()) && field.contains(prefix))
}

fn matches(item: &Item, f: &Fields, term: &Term) -> bool {
    match term {
        Term::Word(w) => f.all().any(|field| word_starts(field, w)),
        Term::Phrase(p) => f.all().any(|field| field.contains(p.as_str())),
        Term::Tag(t) => f.tags.iter().any(|tag| tag == t || word_starts(tag, t)),
        Term::App(a) => word_starts(&f.app, a),
        Term::Site(s) => f.site.contains(s.as_str()),
        Term::Type(t) => kind(item) == t,
    }
}

/// The calendar day of a time in milliseconds, here.
fn day_of(ms: i64) -> Option<NaiveDate> {
    Local.timestamp_millis_opt(ms).single().map(|t| t.date_naive())
}

fn date_matches(day: NaiveDate, sel: &str, today: NaiveDate) -> bool {
    match sel {
        "today" => day == today,
        "yesterday" => day == today - Duration::days(1),
        "last7" => day > today - Duration::days(7) && day <= today,
        "month" => day.year() == today.year() && day.month() == today.month(),
        _ => {
            let key = format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day());
            key == sel || key.starts_with(&format!("{sel}-"))
        }
    }
}

/// The filter sections, to count each one over the others.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Types,
    Dates,
    Apps,
    Sites,
    Tags,
    Favorites,
}

struct Checked<'a> {
    filters: &'a Filters,
    terms: Vec<Term>,
    today: NaiveDate,
    tags: Vec<String>,
}

impl Checked<'_> {
    fn passes(&self, item: &Item, fields: &Fields, skip: Option<Section>) -> bool {
        let f = self.filters;
        let on = |s: Section| skip != Some(s);
        if on(Section::Types) && !f.types.is_empty() && !f.types.iter().any(|t| t == kind(item)) {
            return false;
        }
        if on(Section::Dates) && (!f.dates.is_empty() || f.from.is_some() || f.to.is_some()) {
            let Some(day) = day_of(item.captured) else {
                return false;
            };
            if !f.dates.is_empty() && !f.dates.iter().any(|d| date_matches(day, d, self.today)) {
                return false;
            }
            let parse = |s: &Option<String>| {
                s.as_deref()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            };
            if parse(&f.from).is_some_and(|from| day < from) || parse(&f.to).is_some_and(|to| day > to) {
                return false;
            }
        }
        if on(Section::Apps) && !f.apps.is_empty() {
            let app = item.app.as_deref().map(app_name).unwrap_or_default();
            if !f.apps.iter().any(|a| a.eq_ignore_ascii_case(&app)) {
                return false;
            }
        }
        if on(Section::Sites) && !f.sites.is_empty() {
            let site = item.url.as_deref().and_then(domain).unwrap_or_default();
            if !f.sites.iter().any(|s| s.eq_ignore_ascii_case(&site)) {
                return false;
            }
        }
        if on(Section::Tags)
            && !self.tags.is_empty()
            && !item.tags.iter().any(|t| self.tags.contains(&t.to_lowercase()))
        {
            return false;
        }
        if on(Section::Favorites) && f.favorites && !item.favorite {
            return false;
        }
        self.terms.iter().all(|t| matches(item, fields, t))
    }
}

/// Natural order of names: "Captura 2" before "Captura 10", without case or accents.
pub fn natural(a: &str, b: &str) -> Ordering {
    let (a, b) = (fold(a), fold(b));
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let mut n = String::new();
                while let Some(c) = x.peek().copied().filter(char::is_ascii_digit) {
                    n.push(c);
                    x.next();
                }
                let mut m = String::new();
                while let Some(d) = y.peek().copied().filter(char::is_ascii_digit) {
                    m.push(d);
                    y.next();
                }
                let (n, m) = (n.trim_start_matches('0'), m.trim_start_matches('0'));
                let o = n.len().cmp(&m.len()).then_with(|| n.cmp(m));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                if c != d {
                    return c.cmp(&d);
                }
                x.next();
                y.next();
            }
        }
    }
}

fn compare(a: &Item, b: &Item, sort: Sort) -> Ordering {
    let by = match sort {
        Sort::Modified => a.modified.cmp(&b.modified),
        Sort::Captured => a.captured.cmp(&b.captured),
        Sort::Name => natural(&a.name, &b.name),
        Sort::Size => a.size.cmp(&b.size),
        Sort::Type => a.format.cmp(&b.format),
        Sort::App => natural(
            a.app.as_deref().unwrap_or("\u{10ffff}"),
            b.app.as_deref().unwrap_or("\u{10ffff}"),
        ),
    };
    by.then_with(|| natural(&a.name, &b.name))
        .then_with(|| a.path.cmp(&b.path))
}

/// A month's count and its days' counts.
type Month = (u64, Vec<Value>);
/// A year's count and its months.
type Year = (u64, BTreeMap<String, Month>);

/// The result of a query: the items in order and the counts of each filter value.
pub struct Found<'a> {
    pub items: Vec<&'a Item>,
    pub facets: Value,
}

/// The items of `all` that pass `filters` and `query`, sorted; with `today` for the
/// relative dates.
pub fn run<'a>(
    all: &'a [Item],
    filters: &Filters,
    query: &str,
    sort: Sort,
    descending: bool,
    today: NaiveDate,
) -> Found<'a> {
    let checked = Checked {
        filters,
        terms: parse(query),
        today,
        tags: filters.tags.iter().map(|t| t.to_lowercase()).collect(),
    };
    let fields: Vec<Fields> = all.iter().map(Fields::of).collect();
    let mut items: Vec<&Item> = all
        .iter()
        .zip(&fields)
        .filter(|(i, f)| checked.passes(i, f, None))
        .map(|(i, _)| i)
        .collect();
    items.sort_by(|a, b| {
        let o = compare(a, b, sort);
        if descending {
            o.reverse()
        } else {
            o
        }
    });
    // Each section counted over what the others let through.
    let (checked, fields) = (&checked, &fields);
    let over = |s: Section| {
        all.iter()
            .zip(fields)
            .filter(move |(i, f)| checked.passes(i, f, Some(s)))
            .map(|(i, _)| i)
    };
    let mut types: BTreeMap<&str, u64> = BTreeMap::new();
    for i in over(Section::Types) {
        *types.entry(kind(i)).or_default() += 1;
    }
    let mut days: BTreeMap<String, u64> = BTreeMap::new();
    let mut quick: BTreeMap<&str, u64> = BTreeMap::new();
    for i in over(Section::Dates) {
        if let Some(day) = day_of(i.captured) {
            *days
                .entry(format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day()))
                .or_default() += 1;
            for q in ["today", "yesterday", "last7", "month"] {
                if date_matches(day, q, today) {
                    *quick.entry(q).or_default() += 1;
                }
            }
        }
    }
    // Year ▸ Month ▸ Day.
    let mut years: BTreeMap<String, Year> = BTreeMap::new();
    for (day, n) in &days {
        let (y, m) = (&day[..4], &day[..7]);
        let year = years.entry(y.to_string()).or_default();
        year.0 += n;
        let month = year.1.entry(m.to_string()).or_default();
        month.0 += n;
        month.1.push(json!({"value": day, "count": n}));
    }
    let dates: Vec<Value> = years
        .into_iter()
        .rev()
        .map(|(y, (n, months))| {
            let months: Vec<Value> = months
                .into_iter()
                .rev()
                .map(|(m, (n, mut days))| {
                    days.reverse();
                    json!({"value": m, "count": n, "days": days})
                })
                .collect();
            json!({"value": y, "count": n, "months": months})
        })
        .collect();
    let counted = |values: BTreeMap<String, (String, u64)>| -> Vec<Value> {
        let mut v: Vec<(String, String, u64)> =
            values.into_iter().map(|(k, (label, n))| (k, label, n)).collect();
        v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| natural(&a.1, &b.1)));
        v.into_iter()
            .map(|(_, label, n)| json!({"value": label, "count": n}))
            .collect()
    };
    let mut apps: BTreeMap<String, (String, u64)> = BTreeMap::new();
    for i in over(Section::Apps) {
        let name = i.app.as_deref().map(app_name).unwrap_or_default();
        apps.entry(name.to_lowercase()).or_insert((name, 0)).1 += 1;
    }
    let mut sites: BTreeMap<String, (String, u64)> = BTreeMap::new();
    for i in over(Section::Sites) {
        let site = i.url.as_deref().and_then(domain).unwrap_or_default();
        sites.entry(site.clone()).or_insert((site, 0)).1 += 1;
    }
    let mut tags: BTreeMap<String, (String, u64)> = BTreeMap::new();
    for i in over(Section::Tags) {
        for t in &i.tags {
            tags.entry(t.to_lowercase()).or_insert((t.clone(), 0)).1 += 1;
        }
    }
    let favorites = over(Section::Favorites).filter(|i| i.favorite).count();
    Found {
        items,
        facets: json!({
            "types": types.into_iter().map(|(k, n)| json!({"value": k, "count": n})).collect::<Vec<_>>(),
            "dates": dates,
            "quickDates": quick.into_iter().map(|(k, n)| json!({"value": k, "count": n})).collect::<Vec<_>>(),
            "apps": counted(apps),
            "sites": counted(sites),
            "tags": counted(tags),
            "favorites": favorites,
        }),
    }
}

/// Today, here.
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, size: u64) -> Item {
        Item {
            id: name.to_string(),
            path: format!("{name}.xshot"),
            name: name.to_string(),
            format: "xshot".into(),
            size,
            modified: 1_700_000_000_000,
            captured: Local
                .with_ymd_and_hms(2026, 11, 12, 10, 0, 0)
                .single()
                .unwrap()
                .timestamp_millis(),
            width: Some(100),
            height: Some(100),
            app: None,
            url: None,
            favorite: false,
            tags: Vec::new(),
            text: String::new(),
            cloud: false,
            steps: None,
        }
    }

    fn ids(found: &Found) -> Vec<String> {
        found.items.iter().map(|i| i.id.clone()).collect()
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
    }

    #[test]
    fn names_sort_naturally_and_without_accents() {
        let all = vec![item("Captura 10", 3), item("Captura 2", 1), item("árbol", 2)];
        let f = run(&all, &Filters::default(), "", Sort::Name, false, day());
        assert_eq!(ids(&f), ["árbol", "Captura 2", "Captura 10"]);
        let f = run(&all, &Filters::default(), "", Sort::Size, true, day());
        assert_eq!(ids(&f), ["Captura 10", "árbol", "Captura 2"]);
    }

    #[test]
    fn search_by_prefix_fields_phrases_and_annotations() {
        let mut a = item("Factura cliente", 1);
        a.app = Some("Google Chrome 131".into());
        a.tags = vec!["Cliente".into()];
        let mut b = item("Otra", 2);
        b.tags = vec!["facturas".into()];
        let mut c = item("Notas", 3);
        c.text = "Pulsa el botón Guardar".into();
        c.app = Some("Code".into());
        let all = vec![a, b, c];
        let q = |s: &str| ids(&run(&all, &Filters::default(), s, Sort::Name, false, day()));
        assert_eq!(q("factura"), ["Factura cliente", "Otra"]);
        assert_eq!(q("boton guardar"), ["Notas"]);
        assert_eq!(q("\"el boton\""), ["Notas"]);
        assert_eq!(q("app:chrome tag:cliente"), ["Factura cliente"]);
        assert_eq!(q("camion"), Vec::<String>::new());
        assert_eq!(q("type:image").len(), 3);
    }

    #[test]
    fn filters_cross_sections_and_count_over_the_others() {
        let mut a = item("a", 1);
        a.tags = vec!["Cliente A".into()];
        a.favorite = true;
        a.url = Some("https://www.github.com/x".into());
        let mut b = item("b", 2);
        b.tags = vec!["cliente a".into(), "Bug".into()];
        let mut c = item("c", 3);
        c.captured = Local
            .with_ymd_and_hms(2025, 3, 1, 9, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        let all = vec![a, b, c];
        let filters = Filters {
            tags: vec!["CLIENTE A".into()],
            ..Filters::default()
        };
        let f = run(&all, &filters, "", Sort::Name, false, day());
        assert_eq!(ids(&f), ["a", "b"]);
        // The tag counts ignore the tag filter; the favorites count does not.
        assert_eq!(f.facets["tags"][0], json!({"value": "Cliente A", "count": 2}));
        assert_eq!(f.facets["favorites"], 1);
        assert_eq!(f.facets["sites"][0]["value"], "");
        let filters = Filters {
            dates: vec!["2025".into()],
            ..Filters::default()
        };
        let f = run(&all, &filters, "", Sort::Name, false, day());
        assert_eq!(ids(&f), ["c"]);
        assert_eq!(f.facets["dates"][0]["value"], "2026");
        assert_eq!(f.facets["dates"][1]["months"][0]["value"], "2025-03");
        let today = Filters {
            dates: vec!["today".into()],
            ..Filters::default()
        };
        assert_eq!(run(&all, &today, "", Sort::Name, false, day()).items.len(), 2);
    }

    /// APP-052: a search by tag and a filter over 50,000 items answer in under 300 ms
    /// (release build; `cargo test --release -p xs-library -- --ignored scale`).
    #[test]
    #[ignore]
    fn scale_fifty_thousand_items() {
        let apps = ["Google Chrome 131", "Code", "Excel", "Slack", "Outlook"];
        let all: Vec<Item> = (0..50_000)
            .map(|i| {
                let mut it = item(&format!("Captura {i}"), (i * 37 % 9000) as u64);
                it.id = format!("id{i}");
                it.app = Some(apps[i % apps.len()].to_string());
                it.url = (i % 3 == 0).then(|| format!("https://www.site{}.com/page", i % 40));
                it.tags = if i % 10 == 0 {
                    vec!["Cliente".into()]
                } else {
                    Vec::new()
                };
                it.text = if i % 7 == 0 {
                    "Pulsa el botón Guardar".into()
                } else {
                    String::new()
                };
                it.captured -= (i as i64) * 3_600_000;
                it
            })
            .collect();
        let mut times = Vec::new();
        for q in ["tag:cliente", "guardar", "app:chrome", "captura 4"] {
            let t = std::time::Instant::now();
            let f = run(&all, &Filters::default(), q, Sort::Modified, true, day());
            times.push((q, t.elapsed(), f.items.len()));
        }
        let t = std::time::Instant::now();
        let f = run(
            &all,
            &Filters {
                tags: vec!["Cliente".into()],
                dates: vec!["2026".into()],
                ..Filters::default()
            },
            "",
            Sort::Name,
            false,
            day(),
        );
        times.push(("filters", t.elapsed(), f.items.len()));
        for (q, d, n) in &times {
            println!("{q}: {} ms, {n} items", d.as_millis());
        }
        assert!(times.iter().all(|(_, d, _)| d.as_millis() < 300), "{times:?}");
    }

    #[test]
    fn domains_and_app_names() {
        assert_eq!(
            domain("https://www.github.com/a?b").as_deref(),
            Some("github.com")
        );
        assert_eq!(domain("http://user@docs.rs:8080/x").as_deref(), Some("docs.rs"));
        assert_eq!(domain("not a url"), None);
        assert_eq!(app_name("Google Chrome 131.0"), "Google Chrome");
        assert_eq!(app_name("Code"), "Code");
    }
}
