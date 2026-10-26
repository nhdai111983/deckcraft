# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus Slide**, bộ **NexusOffice**, nhà phát hành **SonnyTech**, app id
`vn.sonnytech.nexus.slides`, tài sản phát hành `NexusSlide-*`, binary cài đặt
`nexus-slide`/`nexus-slide-cli` (MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build
Cargo vẫn là `deckcraft`/`deckcraft-cli`; bundle macOS `Nexus Slide.app`, executable `NexusSlide`).

Nguồn của bản build (About, `links.rs`, MCP `websiteUrl`, metadata gói, README): `https://github.com/sonnytech-vn/deckcraft`
(tổ chức sẽ được tạo và repo chuyển sang; GitHub chuyển hướng URL cũ). Remote git, workflow đồng bộ upstream và URL
`GITHUB_REPOSITORY`/zsync của bản phát hành chưa đổi.

Giữ nguyên định danh máy (không hiện cho người dùng): ProgId `Sonnytech.NexusSlides.Presentation`, `UpgradeCode` và GUID
component WiX, app id / UTI `vn.sonnytech.nexus.slides*` (chữ thường, có "s"), `serverInfo.name = "deckcraft"`,
tên crate, biến `DECKCRAFT_*`, email maintainer `nhdai111983@users.noreply.github.com`.

Đổi thương hiệu 2026-10-23: Sonnytech → SonnyTech, NexusOffices → NexusOffice, Nexus Slides → Nexus Slide
(binary `nexus-slide`/`nexus-slide-cli`, tài sản `NexusSlide-*`, thư mục cài/Start Menu `Nexus Slide`, icon nguồn
`nexus/assets/nexus-slide.*`). **Thư mục cấu hình/từ điển/AutoRecover giữ tên cũ `Nexus Slides` / `nexus-slides`**
(`SETTINGS_DIR(_XDG)` trong `crates/engine/src/nexus.rs`): đổi tên cần đọc thư mục cũ làm dự phòng ở
`apps/deckcraft/src/main.rs`, `spell.rs` và `recovery.rs` (hơn 30 dòng), nên hoãn.

Không đổi (spec §4.4): tên crate `deckcraft-*`, biến `DECKCRAFT_*`, đuôi `.deckcraft` và MIME
`application/x-deckcraft` (khoá định dạng; chỉ đổi tên hiển thị), `"format": "deckcraft"` và MIMETYPE
trong file `.deckcraft`, `serverInfo.name = "deckcraft"` của MCP (định danh máy), khoá plist
`DeckCraftVersion`/`DeckCraftBuildCommit`, id DOM `deckcraft_canvas`, tên file log `deckcraft.log`
(nằm trong thư mục cấu hình riêng của Nexus).

Giữ / đổi trong khai báo kiểu tệp macOS (`packaging/macos/Info.plist.in`): **đổi** UTI xuất
`ai.storyteller.deckcraft.presentation` → `vn.sonnytech.nexus.slides.presentation` (UTI gắn với bundle
id của app khai báo nó) và mô tả "Nexus Slide Presentation"; **giữ** đuôi `deckcraft`
(`public.filename-extension`), thẻ MIME `application/x-deckcraft` (`public.mime-type`) và
`UTTypeConformsTo`. Linux giữ cùng MIME type; Windows giữ đuôi `.deckcraft` (MIME `application/x-deckcraft` là định danh định dạng dùng chung).

## Cùng tồn tại với DeckCraft gốc (Review Focus #1)
- **Windows:** Nexus Slide **không sở hữu** `.deckcraft` (DeckCraft gốc sở hữu qua ProgId riêng). MSI chỉ
  ghi ProgId `Sonnytech.NexusSlides.Presentation` và giá trị có tên dưới `.deckcraft\OpenWithProgids`
  (như `.pptx/.potx/.ppsx`); không ghi giá trị mặc định hay `Content Type` của `HKLM\Software\Classes\.deckcraft`,
  nên cài/gỡ Nexus Slide không đổi hay xoá đăng ký của DeckCraft. Thư mục cài, khoá registry, UpgradeCode,
  GUID component đều riêng.
- **Linux:** file MIME cài dưới tên riêng `/usr/share/mime/packages/vn.sonnytech.nexus.slides.xml`; DeckCraft
  cài `ai.storyteller.deckcraft.xml` cùng định nghĩa `application/x-deckcraft` + `*.deckcraft`. shared-mime-info
  gộp hai file, gỡ gói nào chỉ xoá file của gói đó → định nghĩa vẫn còn từ gói kia. `.desktop` chỉ khai báo
  `MimeType=` (thêm Nexus vào danh sách ứng dụng mở được, không đặt mặc định). Gói deb/rpm tên khác,
  không trùng đường dẫn file (binary, desktop, metainfo, icon, doc). Không cần sửa.
- **macOS:** `Info.plist.in` không có `CFBundleDocumentTypes` (nên không có `LSHandlerRank`); chỉ có
  `UTExportedTypeDeclarations` với UTI riêng `vn.sonnytech.nexus.slides.presentation` cho đuôi `deckcraft`.
  Khi cài cả hai app, hai UTI xuất cùng một đuôi: Launch Services chọn một để gán kiểu cho tệp — không phá
  đăng ký của DeckCraft khi gỡ, nhưng là điểm cần xem lại khi thêm `CFBundleDocumentTypes` (khi đó dùng
  `LSHandlerRank` = `Alternate`, hoặc chuyển sang `UTImportedTypeDeclarations` với UTI của upstream).

## Cơ chế

DeckCraft **không có hệ thống i18n** và **không có điểm chặn chung** cho nhãn giao diện: nhãn lệnh nằm
trong mảng hằng (`UI_COMMANDS` ở `ui-egui/src/lib.rs`, cây menu ở `menus.rs`), tiêu đề hộp thoại ở
`dialogs.rs::title`, còn lại là literal trong từng `ui.label(...)`. Không có hàm hiển thị nào mà mọi
nhãn đều đi qua (CommandSpec.label của engine không chứa tên sản phẩm; nhãn "About DeckCraft" là UI
command). Một hàm `brand()` gắn vào chỗ hiển thị chung sẽ phải chạm cả văn bản của người dùng (tên
theme/tiêu đề trong bài trình chiếu), nên **không dùng**. Thay vào đó:

- `crates/engine/src/nexus.rs` (file mới) — hằng `APP_NAME`, `APP_ID`, `SETTINGS_DIR(_XDG)`,
  `DEFAULT_AUTHOR`, `THEME_NAME`, câu ghi công (en/vi) và bản quyền upstream; dùng chung cho engine,
  MCP, UI và app.
- `crates/ui-egui/src/nexus.rs` (file mới) — logo Nexus vẽ bằng code và trang About.
- Các literal người dùng thấy được sửa tại chỗ (bảng dưới). Văn bản của người dùng không bao giờ bị
  thay. Không Việt hoá ở P0 (chỉ câu ghi công có bản vi).
- Theme dựng sẵn đầu tiên đổi tên `DeckCraft` → `Nexus` (hiện trong Home/New/Design và ghi vào file
  pptx/.deckcraft). Bí danh chỉ dùng khi **tra theo tên**: `nexus::theme_alias()` đổi "DeckCraft"
  (không phân biệt hoa thường) thành `Nexus` trong `file.new`, `design.theme`, `design.colors`,
  `design.effects`, `insert.photoAlbum` và starter template (`templates.rs`), nên script/agent/starter
  viết cho upstream vẫn chạy và nhận theme `Nexus`. Không áp vào tên hiển thị hay tên lưu: bài trình
  chiếu có theme tên "DeckCraft" giữ nguyên tên đó khi mở và lưu lại (có test).

## File thương hiệu

| File | Lý do | Thay được bằng white-label? |
|---|---|---|
| `crates/engine/src/lib.rs` | `pub mod nexus;`; tác giả mặc định `nexus::DEFAULT_AUTHOR` = "Nexus Slide User" (ký bình luận ghi vào file; upstream là "Presenter") | Có — hằng qua biến build |
| `crates/engine/src/links.rs` | Bỏ `DISCORD`/`WEBSITE`/`APPS` (hạ tầng ArtCraft); `APP_PAGE`/`GITHUB`/`ISSUES` trỏ repo fork | Có |
| `crates/engine/src/recovery.rs` | Thư mục AutoRecover riêng (`Nexus Slides/Recovery`, `nexus-slides/recovery`) — không mở bản phục hồi của DeckCraft | Có — `SETTINGS_DIR` |
| `crates/engine/src/spell.rs` | Thư mục từ điển riêng + `user.dic` (Add to Dictionary) trong thư mục Nexus | Có — `SETTINGS_DIR` |
| `crates/engine/src/sample.rs` | Deck mẫu: theme `Nexus`, "made in Nexus Slide", "Made with Nexus Slide" | Có |
| `crates/engine/src/cmd/file.rs` | Lỗi mở file (mật khẩu, không đọc được) nêu Nexus Slide; `theme_named` tra qua `nexus::theme_alias` | Có |
| `crates/engine/src/cmd/format.rs` | Lỗi `font.add` nêu Nexus Slide | Có |
| `crates/engine/src/cmd/insert.rs` | Theme mặc định của Photo Album = `nexus::THEME_NAME`; tra theme qua `nexus::theme_alias`; 2 assert test theo tên mới | Có |
| `crates/engine/src/cmd/design.rs` | `design.theme`/`design.colors`/`design.effects` tra tên qua `nexus::theme_alias` (design.colors kèm màu giữ nguyên tên người dùng đặt, kể cả "DeckCraft"); test: tên theme/effect scheme `Nexus` | Đi cùng `theme.rs` |
| `crates/engine/src/templates.rs` | Starter template tra theme qua `nexus::theme_alias` (starter mới của upstream ghi "DeckCraft" vẫn chạy) | Đi cùng `theme.rs` |
| `crates/engine/src/tests.rs` | Test caret dùng theme `Nexus` | Đi cùng `theme.rs` |
| `crates/model/src/theme.rs` | Theme/colour/format scheme dựng sẵn đầu tiên tên `Nexus` (ghi vào file) | Có — tên theme qua biến build |
| `crates/model/src/tests.rs` | Assert tên theme mới | Đi cùng `theme.rs` |
| `crates/format/src/lib.rs` | `generator: "Nexus Slide <ver>"` ghi vào `.deckcraft`; lỗi "not a Nexus Slide file", media quá lớn (`format` giữ `deckcraft`) | Có |
| `crates/pdf/src/lib.rs` | PDF `/Creator` và `/Producer` = "Nexus Slide" | Có — tên app qua tham số |
| `crates/pdf/src/tests.rs` | Assert Creator/Producer mới | Đi cùng `lib.rs` |
| `crates/pptx/src/write/mod.rs` | `docProps/app.xml` `<Application>Nexus Slide</Application>` | Có |
| `crates/pptx/src/tables.rs` | Tên table style tự sinh "Nexus {n}" ghi vào `tableStyles.xml` (GUID không đổi) | Có |
| `crates/mcp/src/server.rs` | `serverInfo.title` = "Nexus Slide", `websiteUrl` repo fork, INSTRUCTIONS; bỏ link Discord; **`serverInfo.name` giữ `deckcraft`** | Có |
| `crates/mcp/src/backend.rs` | Thông báo kết nối app nêu Nexus Slide | Có |
| `crates/mcp/src/headless.rs` | Gợi ý `nexus-slide --control 7979` / `nexus-slide-cli mcp --connect` | Có |
| `crates/mcp/src/tools.rs` | Mô tả tool: `nexus-slide-cli mcp --connect`, theme `Nexus`, cửa sổ Nexus Slide | Có |
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; nhãn `app.about` "About Nexus Slide"; `app.links` "Project Links" trả {appPage, github, issues}; 2 thông báo trạng thái | Có — `APP_NAME` |
| `crates/ui-egui/src/dialogs.rs` | Tiêu đề About/quit/mặc định; About → `nexus::about_ui` (bỏ link Discord/getartcraft, ghi công chữ); `paint_logo` → logo Nexus; theme album `Nexus`; Find "Nexus Slide can't find" | Có |
| `crates/ui-egui/src/backstage.rs` | Home/Account: tên + logo Nexus, bỏ 2 link Discord ArtCraft, thêm câu ghi công; định dạng "Nexus Slide Presentation (*.deckcraft)"; câu in qua PDF; ô mẫu dùng theme `Nexus` | Có |
| `crates/ui-egui/src/menus.rs` | Help › About Nexus Slide | Có |
| `crates/ui-egui/src/ribbon.rs` | Tiêu đề thanh tiêu đề khi chưa mở bài = `APP_NAME` | Có |
| `crates/ui-egui/src/show.rs` | Cửa sổ "Presenter View — Nexus Slide" | Có |
| `crates/ui-egui/src/backstage_tests.rs` | Assert nhãn/thông báo mới | Đi cùng |
| `crates/ui-egui/src/effects_tests.rs` | Assert theme `Nexus` | Đi cùng |
| `crates/ui-egui/src/theme_colors_tests.rs` | Assert colour scheme `Nexus` | Đi cùng |
| `apps/deckcraft/src/main.rs` | Tiêu đề cửa sổ, app id, tên eframe, USAGE/`--version` (`nexus-slide`), log, bộ lọc file, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình riêng `Nexus Slides` / `nexus-slides` (tên cũ, giữ nguyên) (không đụng cấu hình DeckCraft) | Có |
| `apps/deckcraft/build.rs` | Resource Windows: icon Nexus, ProductName, CompanyName SonnyTech, LegalCopyright (ghi công), OriginalFilename `nexus-slide.exe` | Có |
| `apps/deckcraft-cli/src/main.rs` | `--version`, tiền tố lỗi, gợi ý `nexus-slide-cli`/`nexus-slide --control`, `links` (bỏ Discord/website), dòng "Source:" thay "Community:" | Có |
| `apps/deckcraft-cli/src/help.rs` | Trợ giúp dùng `nexus-slide-cli`, "Nexus Slide presentations", `links` mô tả mới | Có |
| `apps/deckcraft-cli/tests/cli.rs` | Assert trợ giúp theo tên `nexus-slide-cli` | Đi cùng `help.rs` |
| `apps/deckcraft-web/index.html` | `<title>Nexus Slide</title>`, favicon Nexus, "Loading Nexus Slide…" | Có |
| `apps/deckcraft-web/src/web.rs` | Lỗi khởi động bản web | Có |
| `packaging/windows/deckcraft.wxs` | MSI: Name/Manufacturer SonnyTech, **UpgradeCode + GUID component mới**, cài `nexus-slide.exe`/`nexus-slide-cli.exe` vào `Program Files\Nexus Slide`, ProgId `Sonnytech.NexusSlides.Presentation` (ghi bằng RegistryValue), **`.deckcraft` chỉ "Open with"** (`.deckcraft\OpenWithProgids`, không ghi giá trị mặc định/Content Type của `.deckcraft` — DeckCraft gốc sở hữu đuôi này), `Applications\nexus-slide.exe`, App Paths; bỏ URL ARP getartcraft/Discord (About → repo fork); cài `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` cạnh exe (4 component mới, GUID tự sinh); vòng sửa 1: thư mục `licenses\` + ComponentGroup `DeckcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`) trong feature Main, có điều kiện `FontLicences` (mặc định `no`) | Một phần (biến WiX) |
| `packaging/windows/package.ps1` | `NexusSlide-*.msi/.zip`, icon Nexus; zip portable chứa `nexus-slide(.exe)`/`nexus-slide-cli.exe` như MSI; chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục stage (MSI cài từ đó) và zip portable (Apache-2.0 §4(d)); vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\` | Một phần |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus Slide`, `/du` repo fork | Một phần |
| `packaging/macos/Info.plist.in` | Tên bundle, executable `NexusSlide`, id `vn.sonnytech.nexus.slides`, icon, UTI `.deckcraft` mới, copyright (ghi công) | Một phần |
| `packaging/macos/package.sh` | `Nexus Slide.app`, `NexusSlide-*.dmg`, `NexusSlide-cli-*.zip` chứa `nexus-slide-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store`/nền (gắn tên/icon DeckCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.slides`, `/usr/bin/nexus-slide(-cli)`, `NexusSlide-*`, icon `nexus/assets/hicolor`, maintainer SonnyTech, zsync trỏ fork; AppImage giữ `usr/share/doc/nexus-slide` (giấy phép, `NOTICE`, `ATTRIBUTION.md`) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-slide`, vendor/homepage, binary, ghi công | Một phần |
| `packaging/common.sh` | `copy_docs` chép thêm `NOTICE` (Apache-2.0 §4(d)) và `ATTRIBUTION.md` (README Nexus trỏ tới cả hai) qua hàm mới `copy_notices` — bắt buộc, thiếu file thì lỗi; dùng cho deb/AppImage/tar, zip CLI macOS, zip web (và FreeBSD của upstream); vòng sửa 1: chép thêm `assets/fonts/OFL-*.txt` (font của repo nhúng trong binary, nếu có) sau OFL craft-fonts và báo lỗi khi trùng tên; mọi `cp` có `|| return 1` (gọi qua `||` thì errexit tắt); `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL | Nên PR upstream (gói upstream cũng thiếu NOTICE) |
| `packaging/linux/vn.sonnytech.nexus.slides.desktop` | Đổi tên từ `ai.storyteller.deckcraft.desktop`: Name, Exec/TryExec `nexus-slide`, Icon, StartupWMClass | Một phần |
| `packaging/linux/vn.sonnytech.nexus.slides.metainfo.xml.in` | Đổi tên từ `ai.storyteller.deckcraft.metainfo.xml.in`: id, tên, developer SonnyTech, URL fork, bỏ Discord và ảnh chụp trên storytold, binary, ghi công | Một phần |
| `packaging/linux/vn.sonnytech.nexus.slides.mime.xml` | Đổi tên từ `ai.storyteller.deckcraft.mime.xml`: **giữ MIME `application/x-deckcraft` + glob `*.deckcraft`**, chỉ đổi tên hiển thị và icon | Một phần |
| `packaging/web/package.sh` | Zip `NexusSlide-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web → tên sản phẩm/tên zip/đường dẫn ví dụ | Không (tài liệu đi kèm gói) |
| `omastore.toml` | Icon, tên tài sản `NexusSlide-*`, exec `bin/nexus-slide` | Không |
| `README.md` | README Nexus Slide ngắn (vi trước, có English): sản phẩm, NexusOffice/SonnyTech, câu ghi công "Nexus Slide được phát triển dựa trên DeckCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ, trỏ LICENSE-MIT/LICENSE-APACHE/NOTICE/ATTRIBUTION; được `copy_docs` và zip portable đóng gói | Không (README của fork) |
| `Cargo.toml` | `[workspace.package] repository` → `https://github.com/sonnytech-vn/deckcraft` (brand-check `brand-url`; metadata, không code nào đọc `CARGO_PKG_REPOSITORY`) | Có — metadata của fork |
| `ATTRIBUTION.md` | 14 dòng cho icon Nexus (`nexus/assets/**`, SonnyTech, CC0-1.0) | Không (bắt buộc theo luật asset) |
| `crates/ui-egui/src/nexus_toolbar.rs` | File mới (NX-UI, ROADMAP F2.2 + F2.4): bố cục NexusOffice kiểu WPS — thanh menu sinh từ registry lệnh (mỗi tab ribbon một menu, mỗi nhóm một menu con, vị trí cấp 3 thành menu con lồng; `Edit`/`Arrange`/`Selection Pane`/`Format` gộp vào Home, `Comments`/`Tools` vào Review, `Window` vào View, `Format Background` vào Design, `Animation Pane` vào Animations, `Format Shape` vào Shape Format, `Video Format` vào Playback, tab ngữ cảnh `Layout` thành menu "Table Layout"; lệnh UI xem/thu phóng/khung đặt theo `UI_PLACES`); menu ngữ cảnh (Shape/Picture Format, Table Design/Layout, SmartArt/Chart Design, Playback, Master) hiện khi áp dụng; lệnh cần giá trị mở hộp thoại ribbon dùng, menu con các lựa chọn, hoặc biểu mẫu tham số; một hàng công cụ (slide mới + bố cục, clipboard, phông/cỡ, B/I/U, màu chữ, căn lề, bullet/đánh số, hộp văn bản/hình/ảnh/bảng, sắp xếp, trình chiếu từ đầu/hiện tại, tìm); cửa sổ hẹp thì cụm cuối dồn vào menu "More tools"; đường gradient SonnyTech dưới thanh menu; hàng "Accent color" của Preferences (`accent_row`) | — (file mới) |
| `crates/ui-egui/src/nexus_toolbar_tests.rs` | File mới: test menu đủ lệnh, gộp vị trí, registry rỗng, lệnh cần giá trị, kittest bấm menu/Bold/lựa chọn/biểu mẫu, cửa sổ hẹp, `ui.accent`, prefs, Preferences | — (file mới) |
| `crates/ui-egui/examples/shots.rs` | File mới: chụp màn hình offscreen (egui_kittest + wgpu) cửa sổ Nexus Slide với bố cục NexusOffice: sáng, tối, màu chủ đạo coral, menu mở, Preferences, 1024×700 và More tools | — (file mới, chỉ là example) |
| `crates/ui-egui/src/lib.rs` (thêm) | `pub mod nexus_toolbar;`, trường `SlideApp::toolbar` (mặc định tắt = ribbon upstream), `ui()` vẽ `nexus_toolbar::show` thay `ribbon::show` khi bật và `nexus_toolbar::args_window`; `UiState::accent` (`#RRGGBB`, đọc lỏng: giá trị rác = không có, không ghi khoá khi `None`); lệnh UI `ui.accent` (`{"value": "#RRGGBB"}` đặt, `""`/`null` về màu app, không có `value` thì đọc); `logic()` áp màu qua `theme::set_accent` + `with_accent` | Có — cờ giao diện + màu chủ đạo qua prefs |
| `crates/ui-egui/src/theme.rs` | Bảng màu Nexus Slide: Amber `#F59E0B` (ink `#B45309` cho chữ/nền có chữ trắng: 5.0:1 trên trắng, 4.6:1 trên thanh), màu thứ hai Rose `#F43F5E` (ink `#BE123C` cho tab ngữ cảnh, đường gióng); nền slate sáng `#F8FAFC`/`#F1F5F9`/`#E9EDF2`, viền `#E2E8F0`; tối `#0F172A`/`#1E293B`/`#020617`/`#334155` với Amber và chữ tối trên nó; màu chủ đạo người dùng chọn lưu trong bộ nhớ context egui (`set_accent`/`accent`, `with_accent` đẩy sáng/tối tới khi chữ đạt WCAG 4.5:1), `ACCENT_PRESETS`, `parse_hex`/`to_hex`, `contrast_ratio`, `shade_for`; bỏ cam kiểu Office `#D95A1A`/`#C24E14`/`#F26B1D`; `Tokens` thêm `PartialEq` | Có — bảng màu qua biến build |
| `crates/ui-egui/src/icons.rs` | `ORANGE` của icon = Amber 600 `#D97706` thay cam `#F26B1D` của upstream | Có — bảng màu |
| `crates/ui-egui/src/ribbon.rs` (thêm) | Chỉ đổi phạm vi: `contextual`, `TABS`, `layout_menu`, `scheme`, `tip` thành `pub(crate)` để thanh menu/hàng công cụ dùng lại | Có — nếu upstream mở các hàm này |
| `crates/ui-egui/src/dialogs.rs` (thêm) | Preferences › General: hàng "Accent color" (`nexus_toolbar::accent_row`); ảnh chụp/khôi phục của hộp thoại gồm `accent` để Cancel trả lại màu cũ | Có — giao diện NexusOffice |
| `apps/deckcraft/src/main.rs` (thêm) | Bật `app.toolbar` (bố cục NexusOffice) | Có — cờ giao diện |
| `apps/deckcraft-web/src/web.rs` (thêm) | Bật `app.toolbar` cho bản web | Có — cờ giao diện |
| `crates/ui-egui/src/nexus_wps.rs` | File mới (NX-UI.3, ROADMAP F1.3): bố cục NexusOffice kiểu WPS (`SlideApp::wps`, thắng `toolbar`) — không thanh tiêu đề; hàng trên 36 pt: Tệp (mở trang Tệp), thanh truy cập nhanh Lưu/Hoàn tác/Làm lại (+ nút người dùng thêm, menu Tùy chỉnh), thẻ ribbon căn trái (màu nhấn, đậm, gạch 2 pt; » cho thẻ không vừa, thẻ đang mở và thẻ ngữ cảnh luôn giữ), bên phải trạng thái lưu (menu Lưu/Lưu thành), ô tìm lệnh (mở bảng lệnh), Nhận xét, Trình chiếu từ đầu, Chia sẻ (PDF/pptx/ảnh/dàn ý) và "⋯" (sáng/tối, ngôn ngữ giao diện, Tùy chọn, Giới thiệu); dải lệnh 56 pt: các nhóm của thẻ trên một hàng hai dòng nút nhỏ, nút lớn = biểu tượng 20 px trên một dòng nhãn, nhóm không vừa gập vào bảng "…" (lệnh và menu trong đó chạy được); hàng trên không kéo/phóng to cửa sổ khi nhúng | — (file mới) |
| `crates/ui-egui/src/nexus_wps_tests.rs` | File mới: kittest không thanh tiêu đề + Tệp mở trang Tệp tiếng Việt, QAT Lưu/Hoàn tác/Làm lại + trạng thái lưu, mọi thẻ (cả thẻ ngữ cảnh, trang cái) vừa dải ở 1440×900 và 1024×700, bảng "…", ô tìm/Nhận xét/Chia sẻ/», nhúng không đụng cửa sổ, hook tạo/mở/lưu của host, phông chữ nhúng gộp và tự cài lại, `ui` chờ phông, tên "Trình chiếu n" và lời nhắc chỗ dành sẵn tiếng Việt | — (file mới) |
| `crates/ui-egui/src/i18n/mod.rs`, `crates/ui-egui/src/i18n/vi.rs`, `crates/ui-egui/src/i18n/tests.rs` | File mới: cơ chế danh mục tối thiểu (upstream không có i18n): `Language {En, Vi}` (lưu trong `ui.json`, đọc lỏng), `tr` theo ngôn ngữ của khung hình (widget tự tra nhãn: không phải sửa từng chỗ gọi), nhãn xuống dòng tra theo nhãn nối, "Nhãn (phím tắt)" giữ phím tắt; ~700 nhãn tiếng Việt NFC (hàng trên, mọi thẻ ribbon, trang Tệp, thanh trạng thái, "Lưu thay đổi?"); `prompt_fn` cho lời nhắc chỗ dành sẵn | — (file mới) |
| `crates/ui-egui/src/file_dialogs.rs` | File mới: `Services::file_dialog` không chặn (host trả lời qua channel): Mở, Lưu (chỉ kiểu bản trình chiếu `pptx`/`deckcraft`, thêm `.pptx` khi tên không có), xuất bản sao, chèn ảnh/âm thanh/video, dùng lại trang chiếu; "Lưu thay đổi?" › Lưu đóng bản trình chiếu sau khi Lưu thành xong | — (file mới) |
| `crates/ui-egui/src/lib.rs` (thêm, NX-UI.3) | `pub mod nexus_wps/i18n/file_dialogs`; `Services::{file_dialog,new_document,open_document}`; `UiState::language`; `SlideApp::{wps, embedded}`; `run("file.new")` đưa cho host; mở tệp đưa host trừ khi chỉ có bản mới chưa đụng (`is_untouched_blank`, thay thế nó); nhúng: phông chữ gộp vào phông cửa sổ (`set_fonts`, `restyle`, `fonts_installed`; `ui` không vẽ khi thiếu phông), không hỏi khi cửa sổ đóng; `has_drafts`/`commit_drafts` (ghi chú đang gõ, IME); `has_file_system`/`can_pick_files`; Lưu khi nhúng mặc định pptx; `untitled_prefix` theo ngôn ngữ | Có — cờ giao diện và hook host |
| `crates/ui-egui/src/widgets.rs` | Chế độ dải WPS (`COMPACT_H`, `content_h`): nút lớn một dòng, `group` qua cổng của dải, `rows` chảy hai hàng, gallery/nút thả thu nhỏ; nhãn/tooltip dịch qua `i18n::tr`; `group` trả `()` | Không (bố cục) |
| `crates/ui-egui/src/ribbon.rs` (thêm, NX-UI.3) | `tab_groups` tách khỏi `show` (dải WPS dùng lại); nhóm Editing của Home dùng `rows`; nhãn thẻ dịch | Có — nếu upstream tách `tab_groups` |
| `crates/ui-egui/src/backstage.rs` | Trang Tệp dịch (`tr` trong các helper, `dim_raw` cho đường dẫn/lỗi); kiểu sáng WPS (menu trên nền xám hàng trên, trang trắng, mục đang mở tô nhạt màu nhấn + vạch nhấn, nút "‹ Quay lại" có viền); Tài khoản có "Ngôn ngữ giao diện"; Lưu/In dùng `has_file_system` | Có — giao diện NexusOffice |
| `crates/ui-egui/src/dialogs.rs` (thêm, NX-UI.3) | Tiêu đề hộp thoại dịch, "Lưu thay đổi?" (đóng/thoát) tiếng Việt; Lưu trong hộp đóng chờ Lưu thành của host rồi đóng | Có |
| `crates/ui-egui/src/status.rs`, `crates/ui-egui/src/canvas.rs`, `crates/ui-egui/src/qat.rs`, `crates/ui-egui/src/opening.rs`, `crates/ui-egui/src/menus.rs` | Thanh trạng thái/ghi chú/khung soạn/QAT/cửa sổ đang mở dịch; `qat::show_items`; mở tệp xong đóng bản mới bị thay; Ctrl+N đưa host | Có |
| `crates/ui-egui/src/theme.rs`, `crates/ui-egui/src/uifonts.rs`, `crates/ui-egui/src/textures.rs`, `crates/ui-egui/src/icons.rs` (thêm) | `bold_family`, `merge_font_definitions`; `UiFonts::take_changed`; lời nhắc chỗ dành sẵn theo ngôn ngữ (khóa texture theo ngôn ngữ); icon vẽ bằng code `Hamburger`, `ChevronDoubleRight` | Có |
| `crates/render/src/lib.rs`, `crates/render/src/transition.rs` | `RenderOpts::prompt` (lời nhắc chỗ dành sẵn do giao diện cấp; `None` = tiếng Anh upstream) | Có — nên PR upstream (i18n) |
| `crates/engine/src/lib.rs` (thêm) | `Session::untitled_prefix` ("Trình chiếu " → "Trình chiếu 1"; rỗng = "Presentation") | Có — nên PR upstream |
| `crates/ui-egui/examples/shots.rs` (thêm) | Chụp bố cục WPS tiếng Việt: Trang chủ 1440, các thẻ, trang Tệp, In, 1024, bảng "…", », tối, màu san hô, tiếng Anh | — (example) |
| `apps/deckcraft/src/main.rs`, `apps/deckcraft-web/src/web.rs` (thêm, NX-UI.4) | Bật `app.wps` (thay `toolbar`) và giao diện tiếng Việt; `ui.json` không nêu ngôn ngữ giữ mặc định tiếng Việt | Có — cờ giao diện |
| `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/sorter.rs`, `crates/ui-egui/src/panes.rs` (thêm, NX-UI.7) | Bản nháp đang gõ (ghi chú, Trang ghi chú, Văn bản thay thế, khung chữ SmartArt) nằm trên `SlideApp` (`page_notes`, `alt_draft`) thay bộ nhớ tạm egui: `has_drafts`/`commit_drafts` thấy cả bốn (không mất khi đóng); Lưu ghi bản nháp trước; mở tệp hỏi host trước (`open_document` trả `bool`), chỉ thay bản mới chưa đụng *và* không có bản nháp (`only_untouched_blank`); `refresh_look`; nhúng thì không áp thu phóng giao diện riêng; thông báo trạng thái dịch | Có — giao diện NexusOffice |
| `crates/ui-egui/src/file_dialogs.rs`, `crates/ui-egui/src/opening.rs`, `crates/ui-egui/src/i18n/*` (thêm, NX-UI.7) | Câu trả lời Lưu/Xuất muộn tác động đúng bản trình chiếu đã hỏi (`PendingDialog::doc`); lỗi mở tệp dịch theo loại lỗi, không dịch nhầm tên tệp (`i18n::open_error`); `fill` thay một lượt | Có |
| `crates/engine/src/cmd/text.rs` (NX-UI.7) | `text.edit` vào hình chưa có chữ tạo thân chữ rỗng mà không đánh dấu thay đổi khi bản trình chiếu đang sạch (Enter/F2 rồi Esc không thành "Chưa lưu" không có gì để hoàn tác) | Nên PR upstream (lỗi chung) |
| `crates/engine/tests/nexus_editing.rs` | File mới: test hồi quy cho `text.edit` ở trên | — (file mới) |
| `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/panes.rs`, `crates/ui-egui/src/fillui.rs`, `crates/ui-egui/src/widgets.rs`, `crates/ui-egui/src/nexus_toolbar.rs` (thêm, NX-VI) | Hộp thoại Định dạng (Phông chữ, Đoạn, Điểm dừng tab, Dấu đầu dòng và đánh số, Kích thước và vị trí), Tùy chọn (+ Tự động sửa, Tùy chọn lưới), ngăn Định dạng hình/Định dạng nền, trình sửa chuyển sắc, menu màu: nhãn bọc `i18n::tl` (helper `field`/`check`/`choice`/`labeled_text`/`text_row`/`line_choice`/`fx_slider`/`pic_drag`/`three_d_*` tự dịch nên chỗ gọi không đổi); mục drop-down dịch trước khi mở; preset 3-D và họa tiết nền tra theo mã OOXML (`clear` chất liệu ≠ nút "Clear") | Có — `tl` là no-op ở tiếng Anh |
| `crates/ui-egui/src/lib.rs` (thêm, NX-VI) | `fresh_status` dịch thông báo trạng thái có trong danh mục (kênh điều khiển vẫn đọc `status` gốc) | Có |
| `crates/ui-egui/src/backstage.rs` (thêm, NX-VI) | Thư viện mẫu khởi đầu: tên/tóm tắt tiếng Việt, tìm theo cả hai thứ tiếng (`nexus_vi::matching`), `file.new` kèm `language`; ô "Bản mẫu" mở bản tiếng Việt (`open_sample_in`) | Có |
| `crates/engine/src/templates.rs`, `crates/engine/src/sample.rs`, `crates/engine/src/cmd/file.rs`, `crates/engine/src/lib.rs`, `crates/engine/src/tests.rs` (thêm, NX-VI) | `templates::build_in(name, size, language)` và `sample::open_sample_in(s, language)` (bản cũ gọi với `None`); `file.new {template, language}`; `pub mod nexus_vi`; `every_command_survives_empty_and_hostile_params` thử `language` rác | Có — nên PR upstream nếu upstream có i18n nội dung |
| `apps/deckcraft/src/main.rs` (thêm, NX-VI) | `--sample` mở bản mẫu theo ngôn ngữ giao diện | Có |
| `crates/engine/src/nexus_vi.rs` | File mới (NX-VI): chữ tiếng Việt (NFC, lời của chúng tôi) của 6 mẫu khởi đầu và bản mẫu "Bắc cực quang", giữ dàn ý (dòng/cấp); test mọi chữ có bản dịch | — (file mới) |
| `crates/ui-egui/src/i18n/mod.rs`, `crates/ui-egui/src/i18n/vi.rs`, `crates/ui-egui/src/nexus_wps_tests.rs`, `crates/ui-egui/examples/shots.rs` (thêm, NX-VI) | `tl(ui, …)` + bộ ghi nhãn thiếu (chỉ khi test); 359 nhãn mới (gồm 22 khoá theo nơi `nơi.giá_trị` cho `i18n::tl_in`); test mọi nhãn của các hộp thoại/ngăn trên là tiếng Việt và mẫu/bản mẫu mở bằng tiếng Việt; ảnh chụp hộp thoại/ngăn | — (file Nexus) |
| `crates/engine/src/recovery.rs` (thêm, NX-REC) | Hook cho host nhiều phiên (NexusOffice, mỗi tab một `Session`): `recoverable`, `note_offered`, `take`, `discard_left`, `read_entry` (trả `EntryError::TooLarge`/`Unreadable`), `reopen`, `save_entries` (báo bản quá lớn), `discard_session`, `hold_lock`, `sweep`. Sửa độ bền/an toàn: `write_atomic` **fsync tệp tạm trước khi đổi tên** (+ fsync thư mục trên Unix) và xoá tệp tạm khi ghi lỗi; metadata ghi trước bản trình chiếu; `open()` không mở lại mục của chính tiến trình (bản sống của tab khác); nhận/bỏ mục kiểm lại chủ sở hữu dưới khoá thư mục `recovery.lock`; khoá `owner-<id>.lock` kiểm lại mỗi lần ghi và tạo lại nếu bị xoá, dọn khoá chết khi đang giữ khoá; giới hạn 512 MiB cả ghi lẫn đọc (đọc bằng `try_reserve`); không giữ được khoá thì không ghi/nhận mục nào (lỗi); `recoverable` chỉ liệt kê (không ghi gì), `note_offered` ghi `"offered"` vào metadata cho đúng những mục host đã hiện trong hộp hỏi, khi hộp hỏi đã vẽ (mục vượt giới hạn hiển thị của host không bị đánh dấu, nên không bị dọn); dọn `*.tmp` > 1 giờ và mục do sự cố bỏ lại đã được đề nghị ≥ 30 ngày (mục chưa từng đề nghị giữ nguyên); bản trình chiếu quá lớn thì mục cũ được đánh dấu `"stale"` (bản cũ) | fsync + metadata trước + khoá: **nên PR upstream** (xem `UPSTREAM_PRS.md`); hook: có ích cho host bất kỳ |
| `crates/engine/src/cmd/file.rs` (thêm, NX-REC) | `file.recovery.discard` không có `uid` (Không lưu khi thoát, script/MCP) dùng `recovery::discard_session`: chỉ xoá mục của các bản trình chiếu đang mở trong phiên đó — không đụng mục sống của phiên/tab/cửa sổ khác hay mục do sự cố bỏ lại | Nên PR upstream |
| `crates/ui-egui/src/backstage.rs`, `crates/ui-egui/src/backstage_tests.rs` (thêm, NX-REC) | Nút "Recover Unsaved Presentations" ở trang Mở ẩn khi `embedded` (vỏ NexusOffice tự hỏi khôi phục cho mọi tab); test | Có — cờ `embedded` |
| `crates/ui-egui/src/nexus_save_prompt.rs` | File mới, giống hệt ở 4 fork: hộp «Lưu thay đổi?» chung của NexusOffice khi chạy nhúng (quyết định Tech Lead 2026-10-25) — tiêu đề «Lưu thay đổi?», câu «Lưu thay đổi của “<tên>” trước khi đóng?», nút Lưu (màu chủ đạo) · Không lưu · Hủy căn phải, Enter = Lưu (nút đang có tiêu điểm), Esc = Hủy, tiêu điểm bắt đầu ở Lưu, modal trên tab (làm mờ tab, chặn chuột/phím bên dưới); số đo theo DESIGN_SYSTEM.md §3 (bo góc hộp 12, nút 6, lề 20, nút tối thiểu 88×32, tiêu đề 22, chữ 13), màu thẻ theo §2.4 (nền, viền, chữ giống nhau ở mọi app; chỉ nút Lưu theo màu chủ đạo); Tab/Shift+Tab chỉ đi qua ba nút theo thứ tự đọc; AccessKit: thẻ là `Dialog` mang tên «Lưu thay đổi?» | — (file mới) |
| `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/lib.rs` (thêm) | NX-UI.12: khi `app.embedded`, hộp `close` và `quit` vẽ bằng `nexus_save_prompt` (Lưu/Không lưu/Hủy làm đúng việc upstream; nhiều bản trình chiếu: «Lưu thay đổi của n bản trình chiếu…»); trường `SlideApp::drawn_in` (vùng tab); `crates/ui-egui/src/theme.rs` thêm `has_semibold()` (không có Inter SemiBold từ craft-fonts thì tiêu đề hộp tự vẽ đậm). Không nhúng giữ nguyên hộp upstream. Test `nexus_save_prompt_tests.rs` (file mới) | Có — cờ nhúng |
| `crates/model/src/lib.rs`, `crates/engine/src/lib.rs`, `crates/engine/src/cmd/file.rs`, `crates/engine/src/cmd/review.rs`, `crates/engine/src/spell.rs`, `crates/engine/src/nexus_vi.rs`, `crates/ui-egui/src/squiggles.rs`, `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/i18n/mod.rs`, `crates/ui-egui/src/lib.rs` (thêm) | NX-VI: bản trình chiếu tạo mới khi giao diện tiếng Việt có ngôn ngữ mặc định `vi-VN` (trước báo «Tiếng Anh (Mỹ)»): `Session::new_language` (giao diện đặt mỗi khung, `Language::document_language`), `file.new` ghi vào kiểu chữ mặc định (`Presentation::set_default_lang`, PPTX `p:defaultTextStyle` `lang="vi-VN"`); chữ không ghi ngôn ngữ đọc theo `Presentation::default_lang` (thanh trạng thái, Xem lại › Ngôn ngữ, gạch chính tả, `review.spelling`); tệp mở ra giữ ngôn ngữ của nó. **Tác dụng phụ (cả giao diện tiếng Anh):** tệp mở ra có `lang` trong `p:defaultTextStyle` (PowerPoint thường ghi, ví dụ `de-DE`) giờ báo và kiểm chính tả chữ không ghi ngôn ngữ theo ngôn ngữ đó, thay vì luôn `en-US` như upstream; «Tiếng Việt» (`vi-VN`) trong danh sách ngôn ngữ; `spell::valid_lang` thành `pub(crate)`. Test `new_decks_take_the_interface_language`, `new_presentations_in_the_vietnamese_interface_are_vietnamese`. Ứng viên gửi upstream (phần ngôn ngữ mặc định của bản trình chiếu) | Một phần — tính năng tiếng Việt |
| `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/widgets.rs`, `crates/ui-egui/src/panes.rs`, `crates/ui-egui/src/fillui.rs`, `crates/ui-egui/src/ribbon.rs` (thêm, NX-VI.5 — không thêm file upstream mới) | Việt hoá **mọi** hộp thoại còn lại (31: Chèn bảng, Kích thước trang chiếu, Đầu trang và chân trang, Chèn siêu liên kết, Thiết lập trình chiếu, Thu phóng, Nhận xét mới, Thiết lập hành động, Văn bản thay thế, Trang chiếu từ dàn ý, ba hộp Đổi tên, Xuất (cả GIF), Thay phông chữ, Ký hiệu, Phương trình, Dán đặc biệt, Tìm và thay thế, Chính tả, Kiểm tra khả năng tiếp cận, Ngôn ngữ, Trình chiếu tùy chỉnh, Dữ liệu biểu đồ, Cắt phương tiện, SmartArt, Giới thiệu, Album ảnh, Tạo màu/phông chữ chủ đề mới, Ngày và giờ): literal bọc `i18n::tl`, câu có số/tên qua `Language::fill`; `field` giữ thụt lề; danh sách thả dịch trước khi mở; Ngày và giờ có thêm «Tiếng Việt» (vi-VN, mặc định) khi giao diện tiếng Việt. **Đơn vị vi-VN:** `show` ghi ngôn ngữ của khung vào hộp thoại (`nx.lang`), độ dài hiện cm và số dùng dấu phẩy thập phân ở Kích thước trang chiếu, Kích thước và vị trí, Đoạn văn, Điểm dừng tab (nhận cả «,» và «.», hoặc đơn vị gõ kèm `in`/`cm`/`mm`/`pt`); lưu trong vẫn là point. 16 `Slider`/`DragValue` bọc `i18n::units::{slider, drag}` (custom_formatter/custom_parser), `widgets::spinner` lấy chữ/đọc/bước từ `i18n::units::spinner` (inch `"` → cm ở tiếng Việt). Ô độ dài/số để nguyên khi OK hoặc rời ô (Đoạn văn, Kích thước trang chiếu, ô số của ngăn/ribbon) giữ đúng giá trị cũ, không làm tròn qua chữ hiển thị; Kích thước trang chiếu đọc đơn vị gõ kèm. **Dữ liệu biểu đồ** đọc số theo vi-VN (dấu chấm = nghìn, dấu phẩy = thập phân: `units::parse_grouped`/`format_grouped`); ô không phải số thì báo bằng tiếng Việt và giữ dữ liệu cũ. Tiếng Anh giữ nguyên như upstream | Có — `tl`/`units` là no-op ở tiếng Anh |
| `crates/ui-egui/src/i18n/units.rs` | File mới (NX-VI.5): một chỗ duy nhất định dạng/đọc số và độ dài theo ngôn ngữ giao diện (`format_number`, `format_length`, `parse_number`, `parse_measure`, `parse_length`, `localize`, `spinner`, `drag`, `slider`); đầu vào xấu (rỗng, chữ, «1,2,3», NaN/inf, quá dài) trả `None`, không panic; test vòng lặp hiển thị → đọc lại và đầu vào xấu | — (file mới) |
| `crates/ui-egui/src/nexus.rs`, `crates/ui-egui/src/i18n/vi.rs`, `crates/ui-egui/src/nexus_wps_tests.rs`, `crates/ui-egui/examples/shots.rs` (thêm, NX-VI.5) | Trang Giới thiệu tiếng Việt (câu ghi công/bộ ứng dụng bản vi, phiên bản, mã nguồn; bản quyền giữ nguyên); ~250 nhãn mới, nhãn «(in)» đọc «(cm)»; test phủ mọi hộp thoại (`NOT_YET_VI` rỗng), kittest Kích thước và vị trí hiện cm/dấu phẩy ở vi và inch ở en, đầu vào rác không panic; ảnh chụp các hộp thoại mới và ngăn Kích thước | — (file Nexus) |
| `crates/ui-egui/src/theme.rs`, `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/ribbon.rs`, `crates/ui-egui/src/i18n/vi.rs`, `crates/ui-egui/src/nexus_wps_tests.rs`, `crates/ui-egui/examples/shots.rs`, `crates/text/src/datetime.rs`, `crates/engine/src/cmd/insert.rs`, `crates/render/src/tests.rs` (thêm, NX: một kiểu hộp thoại) | **Một kiểu cho mọi hộp thoại** (file mới `crates/ui-egui/src/nexus_dialog.rs` + `nexus_dialog_tests.rs`): `theme::apply` gọi `nexus_dialog::input_visuals` — ô nhập, hộp kiểm, nút chọn, combo có viền 1 px trung tính (≥3:1 với nền ô và nền hộp thoại), viền đậm hơn khi rê chuột, viền màu chủ đạo khi có tiêu điểm (trước đó `inactive.bg_stroke` = không viền và `selection.stroke` = màu chữ trên nền chủ đạo — trắng ở giao diện sáng — nên ô nhập trắng nằm trên nền hộp thoại gần trắng, không thấy chỗ gõ); nền chọn là sắc nhạt của màu chủ đạo. Thanh tiêu đề, ribbon, thanh trạng thái giữ nút không viền (`lib.rs` bọc bằng `nexus_dialog::chrome_style`, nút có viền to hơn 2 px). `dialogs::show` dùng `nexus_dialog::window` (tự co theo nội dung, lề 16 px, mép trên cố định để hộp lớn lên thì giãn xuống); `buttons` → `nexus_dialog::footer` (Hủy rồi nút chính, căn phải, cùng cao/rộng tối thiểu; hàng nút cao đúng bằng nút — trước đó `right_to_left(Align::Center)` chiếm hết chiều cao còn lại nên hộp thoại cao bằng màn hình, nút lơ lửng giữa). `field` → `nexus_dialog::field_row` (nhãn trái một cột 200 px, nhãn dài lên dòng trên). Danh sách (`list_box`): khung, hover, dòng chọn tô cả hàng. Phương trình: ô nhập có tiêu điểm khi mở, Enter = Chèn, Esc = Hủy, «Xem trước: …», Chèn tắt khi trống; số mũ chỉ lấy dấu + chữ số hoặc một chữ cái (`a^2+b^2` → a²+b², trước là a²⁺b²). Ngày và giờ: giao diện tiếng Việt mặc định «Tiếng Việt» (chọn Tiếng Việt / English); **engine** `datetime::field_text` có định dạng vi (dd/MM/yyyy, «Thứ Hai, 26 tháng 10 năm 2026», 24 giờ; trường tự cập nhật hiện đúng định dạng vi), `insert.dateTime` nhận `vi`/`vi-VN` (trước bị từ chối nên OK không chèn gì). `ribbon.rs`: nhãn lưới Chèn bảng («Bảng 3x2») qua `Language::fill`. Ứng viên gửi upstream: phần vi của `datetime.rs`/`insert.rs` | Có — kiểu NexusOffice |

### GUID WiX mới (Nexus Slide riêng, không dùng chung với DeckCraft hay app Nexus khác)

| Mục | GUID cũ (upstream DeckCraft) | GUID mới (Nexus Slide) |
|---|---|---|
| `UpgradeCode` | `6F9C69E7-7825-40D2-BF26-C274DA676425` | `96D068F9-30CC-4E22-808D-242613C1E158` |
| Component `DeckcraftApp` | `D87EA444-41A4-4C19-922E-66D1A29461FF` | `F3911F7B-4F9F-4BA6-8D06-B513AB237AF1` |
| Component `DeckcraftCli` | (tự sinh theo đường dẫn) | tự sinh — thư mục cài (`Nexus Slide`) và tên file (`nexus-slide-cli.exe`) đã khác |
| Component `DeckcraftMitLicense`, `DeckcraftApacheLicense`, `DeckcraftNotice`, `DeckcraftAttribution` | (không có ở upstream) | tự sinh — file giấy phép/NOTICE/ATTRIBUTION.md cài cạnh exe |
| ComponentGroup `DeckcraftFontLicences` (mỗi file trong `licenses\` một component) | (không có ở upstream) | tự sinh — OFL của font nhúng, cài vào `licenses\` |

Sinh bằng `python3 -c "import uuid; print(uuid.uuid4())"` ngày 2026-10-17.

### File mới (không tính là patch)
`crates/engine/src/nexus.rs`, `crates/ui-egui/src/nexus.rs`, `crates/engine/tests/nexus_brand.rs`
(tác giả mặc định, `<Application>`, theme `Nexus`, generator `.deckcraft`, bí danh theme "DeckCraft",
deck upstream giữ tên theme, PDF Creator/Producer, tên table style "Nexus …"),
`crates/mcp/tests/nexus_brand.rs` (serverInfo, tool_definitions không có `deckcraft-cli`),
`apps/deckcraft-cli/tests/nexus_brand.rs` (`--version`/usage/help/links/app nêu `nexus-slide(-cli)`),
`nexus/assets/*` (logo tạm `nexus-slide.svg` + PNG/ICO/ICNS/hicolor sinh bằng
`nexus/assets/render-icons.py`).

## File khác

| File | Lý do | Thay được bằng white-label? |
|---|---|---|

## Rà soát gọi mạng (guard §5)
`rg -n -i "artcraft|storytold|storyteller|discord|telemetry|update|crash|sentry|posthog|https?://" --type rust`:
không có telemetry/crash report/cập nhật tự động trong app. Link ArtCraft app tự mở: About (Discord,
getartcraft), Backstage Home/Account (Discord), nút Slide Show › Learn More (`APP_PAGE`), MCP
instructions/`websiteUrl`, CLI `links` — đã bỏ hoặc trỏ repo fork qua `links.rs`. MSI: URL ARP
getartcraft/Discord đã bỏ. AppImage zsync trỏ GitHub Releases của fork (`GITHUB_REPOSITORY` hoặc
`nhdai111983/deckcraft`). Dependency git `storytold/craft-libs` chỉ là nguồn build (T5/§4.8), không
phải gọi mạng lúc chạy.

## Việc còn nợ (đã allowlist có lý do, hoặc không bị brand-check bắt)
- Đề xuất PR upstream (white-label): hằng `APP_NAME`/`SETTINGS_DIR`/`APP_ID`/tên theme đầu lấy từ biến
  build (`DECKCRAFT_APP_NAME`…) → fork chỉ còn đặt biến, bỏ phần lớn patch ở trên.
- Theme dựng sẵn đầu tiên tên `Nexus`; tên upstream "DeckCraft" vẫn được chấp nhận khi tra theo tên
  (`nexus::theme_alias`) và cho ra `Nexus`. File cũ có theme tên `DeckCraft` mở và lưu lại giữ nguyên
  tên; chỉ Design › Variants (`theme_variants`, so tên chính xác trong `deckcraft-model`) không nhận ra
  nó là theme dựng sẵn và dựng biến thể từ màu của chính file (kết quả màu như nhau).
- `crates/matroska/src/mux.rs:81` (`WritingApp = "deckcraft"`) và `:169` (`MuxingApp =
  "deckcraft-matroska"`) ghi tên upstream (chữ thường, brand-check không bắt) vào tệp Matroska do muxer
  tạo; hiện app không xuất video nên không tới tay người dùng — đổi khi có tính năng xuất video.
- Tác giả mặc định đổi từ "Presenter" (upstream, không mang thương hiệu) sang "Nexus Slide User"
  theo quy ước chung của bộ (ruling T7).
- Gói FreeBSD (`packaging/freebsd/package.sh`) và Flatpak (`packaging/linux/flatpak*`) chưa đổi —
  chưa phát hành (workflow tắt theo spec §4.6). **Chúng trỏ tới tên file `ai.storyteller.deckcraft.*`
  đã đổi tên**, nên sẽ hỏng nếu chạy; làm lại khi phát hành các kênh này.
- DMG macOS: `dmg-layout.DS_Store`/`background.tiff`/`generate.py` gắn với volume và icon DeckCraft →
  DMG Nexus mở với layout mặc định của Finder; sinh lại trên macOS (Task 12).
- `packaging/icons.sh`, `xtask ico` và `assets/app-icon/` (icon upstream) vẫn còn trong repo nhưng
  bản build Nexus không dùng.
- About › Contributors/Models liệt kê người đóng góp upstream (giữ nguyên — ghi công).
- Workflow upstream `release.yml`/`windows-arm64.yml` (đã tắt) còn `DeckCraft`; phát hành Nexus ở Task 12.
- i18n tối thiểu (NX-UI.3, `crates/ui-egui/src/i18n/`): nhãn chưa có trong danh mục hiện tiếng Anh;
  mọi hộp thoại đã có tiếng Việt (NX-VI.5); một số menu thả/khung tác vụ khác có thể còn tiếng Anh. Chuỗi mới của upstream có "DeckCraft" vẫn
  hiện nguyên cho tới khi sửa tại chỗ (brand-check bắt được).
