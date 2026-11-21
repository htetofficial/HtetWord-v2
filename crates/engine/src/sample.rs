//! Built-in sample documents and templates (original text, generated in code).

use std::sync::Arc;
use wordcraft_doc::para::InlineObject;
use wordcraft_doc::props::{Align, Border, BorderStyle, Borders, CharProps, NumRef, Rgb, TabAlign, TabLeader, TabStop, TextColor};
use wordcraft_doc::{Block, Document, ListKind, Paragraph, PartKind, Table, para_block};

fn para(text: &str) -> Paragraph {
    Paragraph::with_text(text, CharProps::default())
}

fn styled(text: &str, style: &str) -> Paragraph {
    para(text).styled(style)
}

/// Append text with formatting to a paragraph.
fn add(p: &mut Paragraph, text: &str, props: CharProps) {
    let at = p.len();
    let _ = p.insert_text(at, text, &props);
}

fn bold() -> CharProps {
    CharProps { bold: Some(true), ..Default::default() }
}
fn italic() -> CharProps {
    CharProps { italic: Some(true), ..Default::default() }
}

/// The showcase document: a short studio handbook with headings, lists, a table, a quote,
/// a footer with page numbers.
pub fn sample_document() -> Document {
    let mut d = Document::new();
    d.core.title = "The Open Studio Handbook".into();
    d.core.creator = "HtetWord".into();
    let mut b: Vec<Block> = Vec::new();
    b.push(Block::Para(styled("The Open Studio Handbook", "Title")));
    b.push(Block::Para(styled("How a small community of makers shares space, tools and time", "Subtitle")));
    let mut intro = para("Every Thursday evening the old print shop on Alder Street fills with people who make things. ");
    add(&mut intro, "Painters", bold());
    add(&mut intro, " set up by the north windows, ", CharProps::default());
    add(&mut intro, "letterpress", italic());
    add(
        &mut intro,
        " regulars claim the back room, and somebody always brings bread. This handbook collects what we have learned about running a shared studio: the routines that keep it pleasant, the tools we pool, and the small agreements that let thirty strangers become a crew.",
        CharProps::default(),
    );
    b.push(Block::Para(intro));
    b.push(Block::Para(styled("Why share a studio?", "Heading1")));
    b.push(Block::Para(para("Rent is the obvious reason, but it is not the best one. A shared room puts your work next to work you would never have seen otherwise. A ceramicist explains glaze chemistry to a comic artist; a photographer borrows a weaver's loom for a still life. The cross-talk is the point.")));
    let mut quote = styled("A studio is a place where unfinished things are welcome.", "Quote");
    quote.props.space_before = Some(12.0);
    b.push(Block::Para(quote));
    b.push(Block::Para(styled("What we agree on", "Heading2")));
    let mut num = (*d.numbering).clone();
    let bullets = num.add_list(ListKind::Bullet);
    let steps = num.add_list(ListKind::Numbered);
    d.numbering = Arc::new(num);
    for (lead, rest) in [
        ("Clean as you go. ", "Leave a table the way you would like to find it."),
        ("Label what is yours. ", "Unlabelled supplies on the commons shelf are fair game."),
        ("Ask before you borrow. ", "Even from the shelf, a quick note helps everyone keep track."),
        ("Quiet hours ", "run from 9 to 11 on weekday mornings."),
    ] {
        let mut p = Paragraph::new().styled("ListParagraph");
        add(&mut p, lead, bold());
        add(&mut p, rest, CharProps::default());
        p.props.numbering = Some(NumRef { num: bullets, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Shared equipment", "Heading2")));
    b.push(Block::Para(para("The table below lists what the membership owns together and who keeps each item in working order.")));
    let rows = [
        ["Equipment", "Location", "Steward", "Booking"],
        ["Etching press", "Back room", "Mara O.", "Sign-up sheet"],
        ["Kiln (cone 10)", "Yard shed", "Theo P.", "48 h notice"],
        ["Large-format printer", "Office", "Ines R.", "Calendar"],
        ["Light table", "North windows", "Anyone", "First come"],
    ];
    let mut t = Table::new(rows.len(), 4, 468.0);
    t.props.style = Some("GridTable4AccentBlue".into());
    for (r, row) in rows.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cell.blocks = vec![para_block(para(txt))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    b.push(Block::Table(t));
    b.push(Block::Para(styled("Opening the studio", "Heading2")));
    for s in [
        "Switch on the main breaker by the door.",
        "Open both skylights unless it is raining.",
        "Start the kettle. This step is not optional.",
        "Check the booking sheet for the kiln and press.",
    ] {
        let mut p = para(s).styled("ListParagraph");
        p.props.numbering = Some(NumRef { num: steps, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Membership", "Heading1")));
    let mut m = para("Membership is ");
    add(&mut m, "pay what you can", CharProps { bold: Some(true), color: Some(TextColor::Rgb(Rgb(0x15, 0x60, 0x82))), ..Default::default() });
    add(
        &mut m,
        ", reviewed every season. New members shadow a regular for their first two visits, and everyone takes one cleaning shift a month. If you have never made anything before, you are exactly who we hoped would show up.",
        CharProps::default(),
    );
    b.push(Block::Para(m));
    let mut h = para("Questions go to the front desk, or to the notice board by the sink. ");
    add(
        &mut h,
        "Visit the community board",
        CharProps { style: Some("Hyperlink".into()), link: Some("https://github.com/storytold/wordcraft".into()), ..Default::default() },
    );
    add(&mut h, " for events.", CharProps::default());
    b.push(Block::Para(h));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    // Footer: centred page number.
    let mut f = Paragraph::new().styled("Footer");
    f.props.align = Some(Align::Center);
    let _ = f.insert_object(
        0,
        InlineObject::Field { instr: "PAGE".into(), result: "1".into(), locked: false, dirty: false, code: false, form: String::new() },
        &CharProps::default(),
    );
    let fid = d.add_part(PartKind::Footer, vec![para_block(f)]);
    d.last_section.footers.default = Some(fid);
    let mut hd = styled("The Open Studio Handbook\t\tSpring edition", "Header");
    hd.props.borders = Some(Borders {
        bottom: Some(Border { style: BorderStyle::Single, width: 0.5, color: Some(Rgb(0xBF, 0xBF, 0xBF)), space: 4.0 }),
        ..Default::default()
    });
    let hid = d.add_part(PartKind::Header, vec![para_block(hd)]);
    d.last_section.headers.default = Some(hid);
    d
}

/// A block-style letter.
pub fn letter() -> Document {
    let mut d = Document::new();
    let lines: Vec<Paragraph> = vec![
        styled("Your Name", "Title"),
        para("123 Your Street · City, ST 00000 · you@example.com"),
        para(""),
        para(&crate::cmd::insert::format_date("MMMM d, yyyy")),
        para(""),
        para("Recipient Name"),
        para("Title, Company"),
        para("Street Address"),
        para(""),
        para("Dear Recipient,"),
        para(
            "Start with a sentence that says why you are writing. Keep paragraphs short and specific, and close with what you would like to happen next.",
        ),
        para("Thank you for your time."),
        para("Sincerely,"),
        para(""),
        para("Your Name"),
    ];
    d.body = lines.into_iter().map(para_block).collect();
    d
}

/// A one-page résumé.
pub fn resume() -> Document {
    let mut d = Document::new();
    let mut b: Vec<Block> =
        vec![Block::Para(styled("Alex Rivera", "Title")), Block::Para(para("Illustrator · alex@example.com · portfolio.example.com"))];
    for (head, items) in [
        ("Experience", vec![("Lead Illustrator, Harbor Books", "2021 – present"), ("Freelance Illustrator", "2016 – 2021")]),
        ("Education", vec![("BFA Illustration, Coastal College of Art", "2016")]),
        ("Skills", vec![("Ink, gouache, digital painting, book layout, lettering", "")]),
    ] {
        b.push(Block::Para(styled(head, "Heading1")));
        for (what, when) in items {
            let mut p = para(what);
            p.props.tabs = Some(vec![TabStop { pos: 468.0, align: TabAlign::Right, leader: TabLeader::None }]);
            if !when.is_empty() {
                add(&mut p, &format!("\t{when}"), italic());
            }
            b.push(Block::Para(p));
        }
    }
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// A report with a cover heading and sections.
pub fn report() -> Document {
    let mut d = Document::new();
    let b: Vec<Block> = vec![
        Block::Para(styled("Report Title", "Title")),
        Block::Para(styled("Subtitle or date", "Subtitle")),
        Block::Para(styled("Summary", "Heading1")),
        Block::Para(para("Summarise the findings in two or three sentences.")),
        Block::Para(styled("Background", "Heading1")),
        Block::Para(para("Describe the context and the question the report answers.")),
        Block::Para(styled("Findings", "Heading1")),
        Block::Para(styled("First finding", "Heading2")),
        Block::Para(para("Explain the evidence.")),
        Block::Para(styled("Recommendations", "Heading1")),
        Block::Para(para("List what should happen next.")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

fn with_default_font(mut d: Document, font_name: &str) -> Document {
    let styles = Arc::make_mut(&mut d.styles);
    styles.default_chr.font = Some(font_name.into());
    styles.default_chr.lang = Some("my-MM".into());
    for st in &mut styles.styles {
        if st.id == "Title" || st.id.starts_with("Heading") {
            st.chr.font = Some(font_name.into());
            if st.id.starts_with("Heading") {
                st.chr.color = Some(TextColor::Rgb(Rgb(0xC2, 0x38, 0x72)));
            }
        }
    }
    d
}

/// Myanmar Article template (`ဆောင်းပါး`) using Pyidaungsu font.
pub fn article() -> Document {
    let mut d = with_default_font(Document::new(), "Pyidaungsu");
    d.core.title = "နှင်းဆီရောင် ဖန်သားပြင်နှင့် မြန်မာစာပေ အနုပညာ".into();
    d.core.creator = "HtetWord".into();
    let mut b: Vec<Block> = Vec::new();
    b.push(Block::Para(styled("နှင်းဆီရောင် ဖန်သားပြင်နှင့် မြန်မာစာပေ အနုပညာ", "Title")));
    b.push(Block::Para(styled("HtetWord — ပြည်ထောင်စုဖောင့် (Pyidaungsu) နှင့် Liquid Glass ဒီဇိုင်းဖြင့် ဆောင်းပါးရေးသားခြင်း", "Subtitle")));

    let mut intro = para("စာရေးသားခြင်းသည် စိတ်ကူးနှင့် ခံစားချက်တို့ကို စာဖတ်သူထံသို့ အလှပဆုံး ပုံဖော်ပေးပို့သည့် အနုပညာတစ်ရပ် ဖြစ်ပါသည်။ ");
    add(&mut intro, "HtetWord", bold());
    add(
        &mut intro,
        " တွင် မြန်မာယူနီကုဒ် စံနှုန်းမီ ပြည်ထောင်စုဖောင့် (Pyidaungsu) အပါအဝင် အခမဲ့ မြန်မာဖောင့်များနှင့် အင်္ဂလိပ် Editorial ဖောင့်များစွာကို တစ်ပါတည်း ထည့်သွင်းပေးထားပါသည်။",
        CharProps::default(),
    );
    b.push(Block::Para(intro));

    b.push(Block::Para(styled("၁။ ဆောင်းပါးဖွဲ့စည်းပုံ အဓိက အချက်များ", "Heading1")));
    let mut num = (*d.numbering).clone();
    let bullets = num.add_list(ListKind::Bullet);
    d.numbering = Arc::new(num);
    for (lead, rest) in [
        ("ဆွဲဆောင်မှုရှိသော ခေါင်းစဉ် — ", "စာဖတ်သူ၏ အာရုံကို ပထမဆုံး စက္ကန့်ပိုင်းအတွင်း ဖမ်းစားနိုင်မည့် ခေါင်းစဉ်နှင့် နိဒါန်း။"),
        ("ရှင်းလင်းသော အပိုဒ်ခွဲများ — ", "Heading 1, Heading 2 နှင့် Quote စတိုင်များကို သုံး၍ အကြောင်းအရာကို စနစ်တကျ ဖွဲ့စည်းခြင်း။"),
        ("ဖောင့်ရွေးချယ်မှု — ", "Pyidaungsu, Pyidaungsu Book, Padauk Book, Noto Serif Myanmar စသည့် ဖောင့်များကို စိတ်ကြိုက် ပြောင်းလဲအသုံးပြုနိုင်ခြင်း။"),
    ] {
        let mut p = Paragraph::new().styled("ListParagraph");
        add(&mut p, lead, bold());
        add(&mut p, rest, CharProps::default());
        p.props.numbering = Some(NumRef { num: bullets, level: 0 });
        b.push(Block::Para(p));
    }

    let mut quote = styled("“လှပသော စာမျက်နှာတစ်ခုသည် စာရေးသူ၏ စိတ်ကူးဉာဏ်ကို ပိုမိုလွတ်လပ်စွာ ပွင့်လန်းစေပါသည်။”", "Quote");
    quote.props.space_before = Some(10.0);
    b.push(Block::Para(quote));

    b.push(Block::Para(styled("၂။ ပါဝင်ပြီးသား မြန်မာဖောင့်များ နှိုင်းယှဉ်ချက်", "Heading1")));
    let rows = [
        ["ဖောင့်အမည် (Font)", "အမျိုးအစား", "အသုံးပြုရန် သင့်လျော်သော ကဏ္ဍ"],
        ["Pyidaungsu", "Official Unicode UI & Body", "ရုံးသုံးစာ၊ ဆောင်းပါး၊ App UI"],
        ["Pyidaungsu Book", "Book Typography", "စာအုပ်၊ ဝတ္ထုရှည်၊ စာတမ်း"],
        ["Padauk Book", "Classic Literary", "စာပေဆောင်းပါး၊ ကဗျာနှင့် သုတေသန"],
        ["Myanmar Tagu / Gantgaw", "Creative Display", "ခေါင်းစဉ်ကြီး၊ ပို့စ်နှင့် ဖိတ်စာ"],
    ];
    let mut t = Table::new(rows.len(), 3, 468.0);
    t.props.style = Some("GridTable4AccentBlue".into());
    for (r, row) in rows.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cell.blocks = vec![para_block(para(txt))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    b.push(Block::Table(t));

    b.push(Block::Para(styled("၃။ နိဂုံးချုပ်", "Heading1")));
    b.push(Block::Para(para(
        "ဤဆောင်းပါး နမူနာပုံစံ (Article Template) ကို အခြေခံ၍ သင်၏ ဆောင်းပါး၊ သုံးသပ်ချက်နှင့် အက်ဆေးများကို DOCX, PDF, Markdown နှင့် HTML ဖိုင်များအဖြစ် အလွယ်တကူ သိမ်းဆည်း ထုတ်ယူနိုင်ပါသည်။",
    )));

    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// Myanmar Book Chapter template (`စာအုပ်`) using Pyidaungsu Book font.
pub fn book() -> Document {
    let mut d = with_default_font(Document::new(), "Pyidaungsu Book");
    d.core.title = "အခန်း (၁) — ပန်းနုရောင် မနက်ခင်း".into();
    d.core.creator = "HtetWord".into();
    let mut b: Vec<Block> = Vec::new();
    let mut ch = styled("အခန်း (၁)", "Subtitle");
    ch.props.align = Some(Align::Center);
    b.push(Block::Para(ch));

    let mut title = styled("ပန်းနုရောင် မနက်ခင်းနှင့် စာအုပ်အိပ်မက်", "Title");
    title.props.align = Some(Align::Center);
    b.push(Block::Para(title));

    let mut epigraph = styled("“စာအုပ်တစ်အုပ်သည် လက်ထဲတွင် ဆုပ်ကိုင်ထားနိုင်သော အိပ်မက်တစ်ခု ဖြစ်သည်။”", "Quote");
    epigraph.props.align = Some(Align::Center);
    b.push(Block::Para(epigraph));

    b.push(Block::Para(para(
        "မနက်ခင်း နေခြည်နုနုက ပြတင်းပေါက်မှန်ချပ်ပေါ်သို့ ဖြာကျလာချိန်တွင် စားပွဲပေါ်ရှိ လက်ဖက်ရည်ပန်းကန်မှ အငွေ့နွေးနွေးတို့ တဖြည်းဖြည်း လွင့်ပျံနေသည်။ စာအုပ်တစ်အုပ် ရေးသားရာတွင် စာလုံးအချိုးအစား၊ စာကြောင်းအကွာအဝေးနှင့် စာမျက်နှာ အပြင်အဆင်တို့သည် စာဖတ်သူ၏ မျက်စိကို သက်သောင့်သက်သာ ရှိစေရန် အရေးကြီးဆုံး ဖြစ်ပါသည်။",
    )));
    b.push(Block::Para(para(
        "HtetWord ၏ စာအုပ်ပုံစံ (Book Template) သည် Pyidaungsu Book ဖောင့်ကို အသုံးပြုထားပြီး အခန်းခေါင်းစဉ်၊ စာမျက်နှာနံပါတ် (Footer Page Number) နှင့် မာတိကာ (Table of Contents) အလိုအလျောက် ထုတ်ယူနိုင်သော Heading စတိုင်များကို အဆင်သင့် ပြင်ဆင်ပေးထားပါသည်။",
    )));
    b.push(Block::Para(styled("အပိုင်း (က) — ဇာတ်လမ်း၏ အစ", "Heading1")));
    b.push(Block::Para(para(
        "ဤနေရာတွင် သင်၏ ဝတ္ထု၊ စာအုပ် သို့မဟုတ် စာတမ်း အကြောင်းအရာများကို ဆက်လက် ရေးသားနိုင်ပါသည်။ References တက်ဘ်မှ Table of Contents နှင့် Footnote များကိုလည်း တိုက်ရိုက် ထည့်သွင်းနိုင်ပါသည်။",
    )));

    d.body = b.into_iter().map(Arc::new).collect();
    let mut f = Paragraph::new().styled("Footer");
    f.props.align = Some(Align::Center);
    let _ = f.insert_object(
        0,
        InlineObject::Field { instr: "PAGE".into(), result: "1".into(), locked: false, dirty: false, code: false, form: String::new() },
        &CharProps::default(),
    );
    let fid = d.add_part(PartKind::Footer, vec![para_block(f)]);
    d.last_section.footers.default = Some(fid);
    d
}

/// Social Media Post & Blog Content template (`ပို့စ်`) using Pyidaungsu font.
pub fn post() -> Document {
    let mut d = with_default_font(Document::new(), "Pyidaungsu");
    d.core.title = "Social Media Post & Content Planner".into();
    d.core.creator = "HtetWord".into();
    let mut b: Vec<Block> = Vec::new();
    b.push(Block::Para(styled("✨ နေ့စဉ် စာရေးသားခြင်း အလေ့အကျင့် (Social Post)", "Title")));
    b.push(Block::Para(styled("Facebook · Instagram · Telegram · Blog Post Template", "Subtitle")));

    let mut hook = para("🌸 ");
    add(&mut hook, "Hook (အဖွင့်စာသား): ", bold());
    add(&mut hook, "တစ်နေ့ကို မိနစ် (၂၀) ပဲ အချိန်ပေးပြီး ကိုယ့်ရဲ့ အတွေးတွေကို စနစ်တကျ ချရေးဖြစ်ရင် ဘာတွေ ပြောင်းလဲသွားမလဲ?", CharProps::default());
    b.push(Block::Para(hook));

    b.push(Block::Para(styled("အဓိက အချက် (၃) ချက်", "Heading1")));
    let mut num = (*d.numbering).clone();
    let steps = num.add_list(ListKind::Numbered);
    d.numbering = Arc::new(num);
    for item in [
        "စိတ်ရှုပ်ထွေးမှုတွေ လျော့နည်းပြီး အတွေးအမြင် ပိုမိုကြည်လင်လာမယ်။",
        "ကိုယ်ပိုင် အရေးအသား စတိုင် (Personal Voice) ပိုမိုပေါ်လွင်လာမယ်။",
        "ဆောင်းပါး၊ စာအုပ်နဲ့ ပို့စ်ကောင်းတွေကို အချိန်တိုအတွင်း ဖန်တီးနိုင်လာမယ်။",
    ] {
        let mut p = para(item).styled("ListParagraph");
        p.props.numbering = Some(NumRef { num: steps, level: 0 });
        b.push(Block::Para(p));
    }

    let mut cta = para("💖 ");
    add(&mut cta, "Call to Action: ", bold());
    add(&mut cta, "ဒီနေ့ သင်ရေးချင်တဲ့ အကြောင်းအရာကို အောက်မှာ ဆက်ရေးကြည့်လိုက်ပါ။ #HtetWord #MyanmarWriting #CreativeLife", CharProps::default());
    b.push(Block::Para(cta));

    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// Poetry & Creative Notes template (`ကဗျာ / မှတ်စု`) using Padauk Book & Pyidaungsu.
pub fn poetry() -> Document {
    let mut d = with_default_font(Document::new(), "Padauk Book");
    d.core.title = "နှင်းဆီပွင့်ဖတ်ပေါ်က ကဗျာ".into();
    d.core.creator = "HtetWord".into();
    let mut b: Vec<Block> = Vec::new();
    for (line, style) in [
        ("နှင်းဆီပွင့်ဖတ်ပေါ်က ကဗျာ", "Title"),
        ("ကဗျာနှင့် ရသစာစု နမူနာပုံစံ", "Subtitle"),
        ("လေပြည်နုနု တိုက်ခတ်ချိန်မှာ", "Normal"),
        ("ပန်းနုရောင် တိမ်တိုက်တို့ လွင့်မျောလာ၊", "Normal"),
        ("စာမျက်နှာဖြူဖြူပေါ် တင်ကျန်ရစ်တဲ့", "Normal"),
        ("နှလုံးသားရဲ့ ကဗျာတစ်ပုဒ်။", "Normal"),
    ] {
        let mut p = styled(line, style);
        p.props.align = Some(Align::Center);
        b.push(Block::Para(p));
    }
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_valid() {
        assert_eq!(sample_document().word_count(), 361);
        for d in [sample_document(), letter(), resume(), report(), article(), book(), post(), poetry()] {
            assert!(d.word_count() > 5);
            let l = wordcraft_layout::layout(&d, &mut wordcraft_layout::LayoutCache::new(), &Default::default());
            assert!(!l.pages.is_empty());
        }
    }
}
