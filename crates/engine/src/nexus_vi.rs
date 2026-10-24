//! NexusOffice: the Vietnamese text of the starter decks (File › New) and the sample deck, so a
//! presentation started from Nexus Slide in Vietnamese reads Vietnamese. Our own wording, written
//! for these decks (nothing comes from another product's templates); the English stays upstream's.
//!
//! `file.new {"template": …, "language": "vi"}` and [`crate::sample::open_sample_in`] pick it up;
//! names that are names (themes, cities, ISO) stay as they are. Every text is NFC.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Whether `language` (a BCP 47 tag such as `vi`, `vi-VN`) asks for the Vietnamese decks.
pub fn is_vi(language: Option<&str>) -> bool {
    language.and_then(|l| l.trim().split(['-', '_']).next()).is_some_and(|l| l.eq_ignore_ascii_case("vi"))
}

/// The Vietnamese text of `en` (a starter's or the sample's own text) when `language` is
/// Vietnamese and the table has it; otherwise `en` as it is.
pub fn text(language: Option<&str>, en: &'static str) -> &'static str {
    if !is_vi(language) || en.is_empty() {
        return en;
    }
    match vi(en) {
        Some(t) => t,
        None => {
            #[cfg(test)]
            misses::note(en);
            en
        }
    }
}

/// The Vietnamese text of `en`, if the table has it.
pub fn vi(en: &str) -> Option<&'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| ENTRIES.iter().copied().collect()).get(en).copied()
}

/// Starters whose name, summary, theme or a slide title contains `query`, in English or in
/// Vietnamese (empty matches all), in gallery order.
pub fn matching(query: &str) -> Vec<&'static crate::templates::Starter> {
    // Upstream's English matches, plus the starters whose Vietnamese name, summary or a slide
    // title contains the query.
    let english = crate::templates::matching(query);
    let q = query.trim().to_lowercase();
    let hit = |en: &str| vi(en).is_some_and(|v| v.to_lowercase().contains(&q));
    crate::templates::starters()
        .iter()
        .filter(|t| english.iter().any(|e| std::ptr::eq(*e, *t)) || hit(t.name) || hit(t.summary) || t.slides.iter().any(|s| hit(s.title)))
        .collect()
}

/// The table (English, Vietnamese), for audits.
pub fn entries() -> &'static [(&'static str, &'static str)] {
    ENTRIES
}

const ENTRIES: &[(&str, &str)] = &[
    // ---- Starter: Briefing
    ("Briefing", "Giao ban"),
    (
        "A weekly check-in: what changed, what's stuck, and who does the next thing",
        "Giao ban hằng tuần: điều gì đã thay đổi, việc gì đang vướng, và ai làm bước tiếp theo",
    ),
    ("Weekly briefing", "Giao ban tuần"),
    ("What changed, what's stuck, and what we do next", "Điều gì đã thay đổi, việc gì đang vướng, và bước tiếp theo"),
    ("Open with what changed since last week.", "Mở đầu bằng những gì đã thay đổi từ tuần trước."),
    ("This week's agenda", "Nội dung tuần này"),
    (
        "Shipped since last time\n\tOne concrete change\nStill open\n\tThe decision we need\nNext\n\tWho does it, and by when",
        "Đã hoàn thành từ lần trước\n\tMột thay đổi cụ thể\nCòn để ngỏ\n\tQuyết định chúng ta cần\nTiếp theo\n\tAi làm, và hạn đến khi nào",
    ),
    ("Where we are", "Chúng ta đang ở đâu"),
    ("A short picture before the details", "Bức tranh ngắn gọn trước khi vào chi tiết"),
    ("Done and next", "Đã xong và tiếp theo"),
    (
        "Done\n\tThe change is in people's hands\n\tWe wrote down what we learned",
        "Đã xong\n\tThay đổi đã đến tay mọi người\n\tChúng ta đã ghi lại điều rút ra",
    ),
    ("Next\n\tPick the one risk to retire\n\tSay who owns it", "Tiếp theo\n\tChọn một rủi ro để xử lý dứt điểm\n\tNói rõ ai phụ trách"),
    ("See you next week", "Hẹn gặp tuần sau"),
    ("Bring one number and one question", "Mang theo một con số và một câu hỏi"),
    // ---- Starter: Lesson
    ("Lesson", "Bài học"),
    ("A short class that ends with the learner explaining the idea", "Một buổi học ngắn, kết thúc bằng việc người học tự giải thích ý tưởng"),
    ("How a lock works", "Ổ khóa hoạt động thế nào"),
    ("A short lesson for people new to the idea", "Bài học ngắn cho người mới làm quen"),
    ("By the end", "Sau bài học"),
    (
        "You can say what a lock protects\n\tAnd what it does not\nYou can sketch one in three steps",
        "Bạn nói được ổ khóa bảo vệ điều gì\n\tVà không bảo vệ điều gì\nBạn phác họa được nó trong ba bước",
    ),
    ("The idea", "Ý tưởng"),
    ("One metaphor, then the real thing", "Một phép so sánh, rồi đến thứ thật"),
    ("Three steps", "Ba bước"),
    (
        "Name the thing being shared\nDecide who may change it\nLeave it so the next person can tell",
        "Gọi tên thứ đang được dùng chung\nQuyết định ai được thay đổi nó\nĐể lại dấu hiệu cho người sau biết",
    ),
    ("Try it", "Thử ngay"),
    ("Explain a lock to someone who has never seen one", "Giải thích ổ khóa cho người chưa từng thấy nó"),
    // ---- Starter: Pitch
    ("Pitch", "Trình bày ý tưởng"),
    ("A ten-minute conversation about one promise, not a tour of features", "Mười phút nói về một lời hứa, không phải dạo quanh các tính năng"),
    ("A quieter inbox", "Hộp thư yên tĩnh hơn"),
    ("Ten minutes. One promise.", "Mười phút. Một lời hứa."),
    ("The mess", "Mớ hỗn độn"),
    (
        "People miss the one message that mattered\n\tIt arrived between newsletters\nThey spend the morning sorting\n\tAnd still feel behind",
        "Người ta bỏ lỡ đúng bức thư quan trọng\n\tNó đến lẫn giữa các bản tin\nHọ mất cả buổi sáng để lọc thư\n\tMà vẫn thấy mình chậm trễ",
    ),
    ("A smaller promise", "Một lời hứa nhỏ hơn"),
    ("Show the one message. Hide the rest until asked.", "Hiện đúng bức thư cần đọc. Ẩn phần còn lại cho đến khi cần."),
    ("Today and after", "Hôm nay và sau này"),
    ("Today", "Hôm nay"),
    ("The important note is buried\nFinding it takes a search", "Bức thư quan trọng bị vùi lấp\nPhải tìm kiếm mới thấy"),
    ("After", "Sau này"),
    ("It sits at the top until you deal with it\nEverything else waits", "Nó nằm trên cùng cho đến khi bạn xử lý\nMọi thứ khác chờ đến lượt"),
    ("The ask", "Đề nghị"),
    (
        "Twenty people, two weeks, one question: did the right note surface?",
        "Hai mươi người, hai tuần, một câu hỏi: bức thư cần đọc có hiện lên đúng lúc không?",
    ),
    // ---- Starter: Field Notes
    ("Field Notes", "Ghi chép thực địa"),
    ("What an outing set out to see, and what it actually found", "Chuyến đi định xem gì, và thực tế đã thấy gì"),
    ("Saturday on the ridge", "Thứ Bảy trên sườn núi"),
    ("What we went to see, and what we found", "Điều chúng tôi định xem, và điều chúng tôi tìm thấy"),
    ("Plan", "Kế hoạch"),
    (
        "Leave at dawn\n\tCold, clear, little wind\nWalk the west path\nBe back before dusk",
        "Xuất phát lúc bình minh\n\tLạnh, trời quang, ít gió\nĐi theo lối mòn phía tây\nVề trước khi trời tối",
    ),
    ("What we saw", "Những gì chúng tôi thấy"),
    ("Notes taken on the spot", "Ghi chép ngay tại chỗ"),
    ("Expected and found", "Dự kiến và thực tế"),
    ("Expected\n\tA clear view from the ridge\n\tThe creek on the map", "Dự kiến\n\tTầm nhìn thoáng từ sườn núi\n\tCon suối trên bản đồ"),
    ("Found\n\tLow cloud on the ridge\n\tThe creek, and a better pencil", "Thực tế\n\tMây thấp phủ sườn núi\n\tCon suối, và một cây bút chì tốt hơn"),
    ("Pack list for next time", "Đồ cần mang lần sau"),
    ("Warmer gloves. A second pencil. The map that shows the creek.", "Găng tay ấm hơn. Thêm một cây bút chì. Tấm bản đồ có vẽ con suối."),
    // ---- Starter: Quarterly
    ("Quarterly", "Báo cáo quý"),
    ("A report meant to be read aloud in half an hour", "Bản báo cáo để đọc to trong nửa giờ"),
    ("Spring report", "Báo cáo mùa xuân"),
    ("Read aloud in thirty minutes", "Đọc to trong ba mươi phút"),
    ("Headlines", "Điểm chính"),
    (
        "The waiting list got shorter\n\tBy the amount we promised\nTwo things slipped\n\tBoth have a new date",
        "Danh sách chờ đã ngắn lại\n\tĐúng mức chúng ta đã hứa\nHai việc bị trễ\n\tCả hai đã có hạn mới",
    ),
    ("The numbers", "Các con số"),
    ("Only the ones that change a decision", "Chỉ những con số làm thay đổi quyết định"),
    ("What we will stop", "Những việc chúng ta sẽ dừng"),
    ("The report nobody reads\nThe meeting that repeats the report", "Bản báo cáo không ai đọc\nCuộc họp nhắc lại bản báo cáo"),
    ("Questions we want", "Câu hỏi chúng ta mong nhận"),
    ("Where should the next quarter refuse work?", "Quý tới nên từ chối những việc gì?"),
    // ---- Starter: Evening Talk
    ("Evening Talk", "Buổi nói chuyện tối"),
    ("A public talk the audience can check on the way home", "Buổi nói chuyện mà khán giả tự kiểm chứng được trên đường về"),
    ("Lights out", "Tắt đèn"),
    ("A talk about how cities sleep", "Câu chuyện về giấc ngủ của thành phố"),
    ("Road map", "Lộ trình"),
    (
        "Why the sky looks orange\nWhat a dark street is for\nOne change a block can make",
        "Vì sao bầu trời có màu cam\nCon phố tối để làm gì\nMột thay đổi mà một khu phố làm được",
    ),
    ("Look up", "Nhìn lên"),
    ("The part you can check on the way home", "Phần bạn tự kiểm chứng được trên đường về"),
    ("One change", "Một thay đổi"),
    (
        "Shield the lamp so light goes down\n\tNot into bedroom windows\nLeave a path you can still walk",
        "Che chụp đèn để ánh sáng chiếu xuống\n\tKhông rọi vào cửa sổ phòng ngủ\nVẫn chừa lối đi đủ sáng",
    ),
    ("Walk home", "Đi bộ về nhà"),
    ("Count the lamps you can see from your pillow", "Đếm số ngọn đèn bạn thấy được từ gối"),
    // ---- The sample deck (`--sample`, File › New › Sample deck)
    ("Northern Lights", "Bắc cực quang"),
    (
        "A short tour of the aurora — where it comes from, when to look, and how to photograph it",
        "Tìm hiểu nhanh về cực quang: từ đâu mà có, khi nào nên ngắm, và chụp ảnh thế nào",
    ),
    ("Welcome everyone. This deck was made in Nexus Slide.", "Chào mọi người. Bản trình chiếu này được làm bằng Nexus Slide."),
    ("Aurora Band", "Dải cực quang"),
    ("What we'll cover", "Nội dung"),
    (
        "The science: solar wind meets the magnetosphere\nWhere and when to see it\n\tHigh latitudes, dark skies, long winter nights\n\tKp index and space-weather forecasts\nPhotographing the aurora\nSafety and etiquette in the field",
        "Khoa học: gió mặt trời gặp từ quyển\nNgắm ở đâu và khi nào\n\tVĩ độ cao, trời tối, đêm đông dài\n\tChỉ số Kp và dự báo thời tiết không gian\nChụp ảnh cực quang\nAn toàn và phép lịch sự khi đi thực địa",
    ),
    ("The Science", "Khoa học"),
    ("Charged particles, magnetic fields and glowing gases", "Hạt mang điện, từ trường và chất khí phát sáng"),
    ("From the Sun to your sky", "Từ Mặt Trời đến bầu trời của bạn"),
    ("Solar wind", "Gió mặt trời"),
    ("Magnetosphere", "Từ quyển"),
    ("Particles collide", "Hạt va chạm"),
    ("Light!", "Phát sáng!"),
    ("Oxygen glows green and red; nitrogen adds blue and purple fringes.", "Oxi phát sáng xanh lục và đỏ; nitơ thêm viền xanh lam và tím."),
    ("Best months to look (clear nights)", "Tháng nên ngắm (đêm quang mây)"),
    ("Average clear, dark nights per month", "Số đêm tối, quang mây trung bình mỗi tháng"),
    ("Sep", "Th9"),
    ("Oct", "Th10"),
    ("Nov", "Th11"),
    ("Dec", "Th12"),
    ("Jan", "Th1"),
    ("Feb", "Th2"),
    ("Mar", "Th3"),
    ("Camera settings that work", "Thông số máy ảnh nên dùng"),
    ("Setting", "Thông số"),
    ("Faint aurora", "Cực quang mờ"),
    ("Bright, active aurora", "Cực quang sáng, mạnh"),
    ("Aperture", "Khẩu độ"),
    ("f/1.4 – f/2.8", "f/1,4 – f/2,8"),
    ("f/2.8", "f/2,8"),
    ("Shutter", "Tốc độ màn trập"),
    ("8 – 15 s", "8 – 15 giây"),
    ("1 – 4 s", "1 – 4 giây"),
    ("Focus", "Lấy nét"),
    ("Manual, on a bright star", "Thủ công, vào một ngôi sao sáng"),
    ("Field checklist", "Danh sách kiểm tra thực địa"),
    (
        "Bring\n\tTripod and spare batteries\n\tHeadlamp with a red mode\n\tWarm layers and a thermos",
        "Mang theo\n\tChân máy và pin dự phòng\n\tĐèn đội đầu có chế độ đỏ\n\tÁo ấm nhiều lớp và bình giữ nhiệt",
    ),
    (
        "Remember\n\tCheck the forecast and cloud cover\n\tLet your eyes adapt for 20 minutes\n\tLeave no trace",
        "Ghi nhớ\n\tXem dự báo và độ che phủ mây\n\tĐể mắt thích nghi trong 20 phút\n\tKhông để lại dấu vết",
    ),
    ("Clear skies!", "Chúc trời quang!"),
    ("Made with Nexus Slide — free and open source", "Làm bằng Nexus Slide, miễn phí và mã nguồn mở"),
    ("Introduction", "Giới thiệu"),
    ("Science", "Khoa học"),
    ("In the Field", "Ngoài thực địa"),
];

/// Tests: the English texts asked for in Vietnamese that the table lacks.
#[cfg(test)]
pub(crate) mod misses {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    thread_local! {
        static MISSES: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }

    pub(crate) fn note(en: &str) {
        MISSES.with(|m| m.borrow_mut().insert(en.to_string()));
    }

    pub(crate) fn take() -> Vec<String> {
        MISSES.with(|m| std::mem::take(&mut *m.borrow_mut())).into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use super::*;
    use crate::Session;

    /// Every starter and the sample deck, built in Vietnamese, find each of their texts in the
    /// table; the English builds stay upstream's.
    #[test]
    fn every_starter_and_the_sample_read_vietnamese() {
        let _ = misses::take();
        let mut s = Session::new();
        for starter in crate::templates::starters() {
            let r = s.execute("file.new", &json!({"template": starter.name, "language": "vi-VN"})).unwrap();
            assert_eq!(r["template"], json!(starter.name), "the template keeps its (English) id");
            let st = s.active().unwrap();
            let want = text(Some("vi"), starter.slides[0].title);
            assert_eq!(st.doc.slides[0].title(), want, "{}", starter.name);
            assert_ne!(want, starter.slides[0].title, "{}", starter.name);
            assert_eq!(st.doc.masters[0].theme.name, starter.theme, "theme names stay names");
            assert!(!st.is_dirty());
            let _ = text(Some("vi"), starter.name);
            let _ = text(Some("vi"), starter.summary);
        }
        crate::sample::open_sample_in(&mut s, Some("vi")).unwrap();
        let st = s.active().unwrap();
        assert_eq!(st.title, "Bắc cực quang");
        assert_eq!(st.doc.slides[0].title(), "Bắc cực quang");
        assert!(!st.is_dirty() && st.history.undo.is_empty(), "the sample opens clean");
        let missing = misses::take();
        assert!(missing.is_empty(), "no Vietnamese for: {missing:#?}");
        // English (and no language) builds upstream's decks.
        s.execute("file.new", &json!({"template": "Briefing"})).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Weekly briefing");
        s.execute("file.new", &json!({"template": "Briefing", "language": "en"})).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Weekly briefing");
        crate::sample::open_sample(&mut s).unwrap();
        assert_eq!(s.active().unwrap().doc.slides[0].title(), "Northern Lights");
    }

    /// The Vietnamese keeps each text's outline (lines and indent levels) and is NFC; no English
    /// text is listed twice.
    #[test]
    fn the_table_keeps_outlines_and_is_nfc() {
        for (en, vi) in ENTRIES {
            assert!(!vi.trim().is_empty(), "{en:?}");
            assert!(!vi.chars().any(|c| ('\u{0300}'..='\u{0323}').contains(&c)), "{vi:?} is not NFC");
            let shape = |t: &str| t.split('\n').map(|l| l.chars().take_while(|c| *c == '\t').count()).collect::<Vec<_>>();
            assert_eq!(shape(en), shape(vi), "{en:?} → {vi:?} changes the outline");
        }
        let mut keys: Vec<&str> = ENTRIES.iter().map(|(en, _)| *en).collect();
        keys.sort_unstable();
        let n = keys.len();
        keys.dedup();
        assert_eq!(n, keys.len(), "an English text is listed twice");
    }

    #[test]
    fn languages_and_search() {
        assert!(is_vi(Some("vi")) && is_vi(Some("VI_vn")) && is_vi(Some(" vi-VN ")));
        assert!(!is_vi(Some("en")) && !is_vi(None) && !is_vi(Some("")) && !is_vi(Some("vie")));
        assert_eq!(text(Some("en"), "Briefing"), "Briefing");
        assert_eq!(text(Some("vi"), "Briefing"), "Giao ban");
        assert_eq!(matching("sườn núi").iter().map(|t| t.name).collect::<Vec<_>>(), vec!["Field Notes"]);
        assert_eq!(matching("ridge").iter().map(|t| t.name).collect::<Vec<_>>(), vec!["Field Notes"]);
        assert_eq!(matching("").len(), crate::templates::starters().len());
        assert!(matching("không có mẫu này").is_empty());
    }
}
