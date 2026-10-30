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
- **macOS:** upstream (2026-10) thêm `CFBundleDocumentTypes`. Nexus giữ khối đó nhưng loại `.deckcraft` là
  "Nexus Slide Presentation" với UTI riêng `vn.sonnytech.nexus.slides.presentation` và **`LSHandlerRank` =
  `Alternate`** (upstream là `Owner`): Nexus Slide chỉ được đề xuất trong Open With, không giành quyền mở
  `.deckcraft` của DeckCraft gốc. `.pptx/.potx/.ppsx` giữ `Alternate` như upstream. Vẫn còn hai UTI xuất cùng
  một đuôi khi cài cả hai app (Launch Services chọn một) — chuyển sang `UTImportedTypeDeclarations` với UTI
  của upstream nếu điều đó gây lỗi.

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
| `crates/engine/src/cmd/format/painter.rs` | Lỗi `font.add` nêu Nexus Slide (upstream 2026-10 tách `cmd/format.rs` thành `cmd/format/*`) | Có |
| `crates/engine/src/cmd/insert/album.rs` | Theme mặc định của Photo Album = `nexus::THEME_NAME`; tra theme qua `nexus::theme_alias` (upstream tách `cmd/insert.rs`) | Có |
| `crates/engine/src/cmd/insert/icon_and_picture_tests.rs` | 2 assert test theo tên theme mới | Đi cùng `album.rs` |
| `crates/engine/src/cmd/design.rs` | `design.theme` tra tên qua `nexus::theme_alias` | Đi cùng `theme.rs` |
| `crates/engine/src/cmd/design/colors.rs` | `design.colors`/`design.effects` tra tên qua `nexus::theme_alias` (design.colors kèm màu giữ nguyên tên người dùng đặt, kể cả "DeckCraft") (upstream tách `cmd/design.rs`) | Đi cùng `theme.rs` |
| `crates/engine/src/cmd/design/tests.rs`, `crates/engine/src/cmd/design/browse_theme_tests.rs` | Test: tên theme/effect scheme `Nexus` | Đi cùng `theme.rs` |
| `crates/engine/src/templates.rs` | Starter template tra theme qua `nexus::theme_alias` (starter mới của upstream ghi "DeckCraft" vẫn chạy) | Đi cùng `theme.rs` |
| `crates/engine/src/tests.rs` | Test caret dùng theme `Nexus` | Đi cùng `theme.rs` |
| `crates/model/src/theme.rs` | Theme/colour/format scheme dựng sẵn đầu tiên tên `Nexus` (ghi vào file) | Có — tên theme qua biến build |
| `crates/model/src/tests.rs` | Assert tên theme mới | Đi cùng `theme.rs` |
| `crates/format/src/lib.rs` | `generator: "Nexus Slide <ver>"` ghi vào `.deckcraft`; lỗi "not a Nexus Slide file", media quá lớn (`format` giữ `deckcraft`) | Có |
| `crates/format/src/chart_parts.rs` | 2 lỗi "larger than Nexus Slide can open" (file mới của upstream 2026-10) | Có |
| `crates/pdf/src/lib.rs` | PDF `/Creator` và `/Producer` = "Nexus Slide" | Có — tên app qua tham số |
| `crates/pdf/src/tests.rs` | Assert Creator/Producer mới | Đi cùng `lib.rs` |
| `crates/pptx/src/write.rs` | `docProps/app.xml` `<Application>Nexus Slide</Application>` | Có |
| `crates/pptx/src/tables.rs` | Tên table style tự sinh "Nexus {n}" ghi vào `tableStyles.xml` (GUID không đổi) | Có |
| `crates/mcp/src/server.rs` | `serverInfo.title` = "Nexus Slide", `websiteUrl` repo fork, INSTRUCTIONS; bỏ link Discord; **`serverInfo.name` giữ `deckcraft`** | Có |
| `crates/mcp/src/backend.rs` | Thông báo kết nối app nêu Nexus Slide | Có |
| `crates/mcp/src/headless.rs` | Gợi ý `nexus-slide --control 7979` / `nexus-slide-cli mcp --connect` | Có |
| `crates/mcp/src/tools.rs` | Mô tả tool: `nexus-slide-cli mcp --connect`, theme `Nexus`, cửa sổ Nexus Slide | Có |
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; nhãn `app.about` "About Nexus Slide"; `app.links` "Project Links" | Có — `APP_NAME` |
| `crates/ui-egui/src/app/commands.rs` | `app.links` trả {appPage, github, issues} (upstream tách `lib.rs` → `app/*`) | Có |
| `crates/ui-egui/src/app/files.rs` | 2 thông báo trạng thái (không mở được / không xuất được) nêu Nexus Slide | Có — `APP_NAME` |
| `crates/ui-egui/src/dialogs/updates_ui.rs` | **Tắt trình cập nhật của upstream** (`craft_update`, thêm 2026-10): nó kiểm tra và tải bản phát hành `storytold/deckcraft` (DeckCraft, không phải Nexus Slide) lúc khởi động. Nexus: `enable()` không làm gì, Help › Check for Updates… chỉ báo bản mới đến qua bộ cài NexusOffice (câu en + vi trong `NEXUS_UPDATES`; hiện hiển thị en vì fork chưa có chọn ngôn ngữ giao diện), không gọi mạng; test `nexus_never_checks_upstream_releases`, `nexus_update_message_names_the_installer_in_en_and_vi` | Có — nên PR upstream một cờ build tắt updater / chọn repo |
| `crates/ui-egui/src/dialogs.rs` | Tiêu đề About/quit/mặc định | Có |
| `crates/ui-egui/src/dialogs/about.rs` | About → `nexus::about_ui` (bỏ link Discord/getartcraft, ghi công chữ); `paint_logo` → logo Nexus (upstream tách `dialogs.rs` → `dialogs/*`) | Có |
| `crates/ui-egui/src/dialogs/photo_album.rs` | Theme album `Nexus` | Có |
| `crates/ui-egui/src/dialogs/find.rs` | "Nexus Slide can't find" | Có |
| `crates/ui-egui/src/backstage/files.rs` | Home: tên + logo Nexus, bỏ link Discord ArtCraft; ô mẫu dùng theme `Nexus` (upstream tách `backstage.rs` → `backstage/*`) | Có |
| `crates/ui-egui/src/backstage/info.rs` | Account: tên Nexus, bỏ link Discord, thêm câu ghi công, "About Nexus Slide"; định dạng "Nexus Slide Presentation (*.deckcraft)" | Có |
| `crates/ui-egui/src/backstage/print.rs` | Câu in qua PDF nêu Nexus Slide | Có |
| `crates/ui-egui/src/menus.rs` | Help › About Nexus Slide | Có |
| `crates/ui-egui/src/ribbon/title_bar.rs` | Tiêu đề thanh tiêu đề khi chưa mở bài = `APP_NAME` (upstream tách `ribbon.rs`) | Có |
| `crates/ui-egui/src/show/presenter.rs` | Cửa sổ "Presenter View — Nexus Slide" (upstream tách `show.rs`) | Có |
| `crates/ui-egui/src/backstage/tests.rs` | Assert nhãn/thông báo mới | Đi cùng |
| `crates/ui-egui/src/ribbon/effects_tests.rs` | Assert theme `Nexus` | Đi cùng |
| `crates/ui-egui/src/dialogs/theme_colors_tests.rs` | Assert colour scheme `Nexus` | Đi cùng |
| `apps/deckcraft/src/main.rs` | Tiêu đề cửa sổ, app id, tên eframe, USAGE/`--version` (`nexus-slide`), log, bộ lọc file, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình riêng `Nexus Slide` / `nexus-slide` (không đụng cấu hình DeckCraft); lỗi update helper nêu `APP_NAME`; test "mọi desktop cùng loại tệp" đọc `vn.sonnytech.nexus.slides.desktop` | Có |
| `apps/deckcraft/build.rs` | Resource Windows: icon Nexus, ProductName, CompanyName SonnyTech, LegalCopyright (ghi công), OriginalFilename `nexus-slide.exe` | Có |
| `apps/deckcraft-cli/src/main.rs` | `--version`, tiền tố lỗi, gợi ý `nexus-slide-cli`/`nexus-slide --control`, `links` (bỏ Discord/website), dòng "Source:" thay "Community:" | Có |
| `apps/deckcraft-cli/src/help.rs` | Trợ giúp dùng `nexus-slide-cli`, "Nexus Slide presentations", `links` mô tả mới | Có |
| `apps/deckcraft-cli/tests/it/cli.rs` | Assert trợ giúp theo tên `nexus-slide-cli` | Đi cùng `help.rs` |
| `apps/deckcraft-web/index.html` | `<title>Nexus Slide</title>`, favicon Nexus, "Loading Nexus Slide…" | Có |
| `apps/deckcraft-web/src/web.rs` | Lỗi khởi động bản web | Có |
| `packaging/windows/deckcraft.wxs` | MSI: Name/Manufacturer SonnyTech, **UpgradeCode + GUID component mới**, cài `nexus-slide.exe`/`nexus-slide-cli.exe` vào `Program Files\Nexus Slide`, ProgId `Sonnytech.NexusSlides.Presentation` (ghi bằng RegistryValue), **`.deckcraft` chỉ "Open with"** (`.deckcraft\OpenWithProgids`, không ghi giá trị mặc định/Content Type của `.deckcraft` — DeckCraft gốc sở hữu đuôi này), `Applications\nexus-slide.exe`, App Paths; bỏ URL ARP getartcraft/Discord (About → repo fork); cài `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` cạnh exe (4 component mới, GUID tự sinh); vòng sửa 1: thư mục `licenses\` + ComponentGroup `DeckcraftFontLicences` (`<Files Include="$(var.BinDir)\licenses\*">`) trong feature Main, có điều kiện `FontLicences` (mặc định `no`) | Một phần (biến WiX) |
| `packaging/windows/package.ps1` | `NexusSlide-*.msi/.zip`, icon Nexus; zip portable chứa `nexus-slide(.exe)`/`nexus-slide-cli.exe` như MSI; chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục stage (MSI cài từ đó) và zip portable (Apache-2.0 §4(d)); vòng sửa 1: dựng `licenses\` = OFL craft-fonts (`OFL-<family>.txt`) + `assets\fonts\OFL-*.txt` (tên duy nhất không phân biệt hoa thường; ném lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL), truyền `-d FontLicences=yes|no` cho WiX, zip portable lấy OFL từ `licenses\`; 2026-10: giữ kiểm tra icon nhúng (#1114) của upstream (namespace PowerShell `NexusSlide`), **không** gọi `check-install-policy.ps1`/WixUI của upstream (hợp đồng cài đặt ArtCraft, xem `deckcraft.wxs`) | Một phần |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus Slide`, `/du` repo fork (thư mục tạm ngẫu nhiên của upstream giữ nguyên) | Một phần |
| `packaging/macos/Info.plist.in` | Tên bundle, executable `NexusSlide`, id `vn.sonnytech.nexus.slides`, icon, UTI `.deckcraft` mới, copyright (ghi công); `CFBundleDocumentTypes` của upstream với tên/icon/UTI Nexus và `.deckcraft` ở hạng `Alternate` | Một phần |
| `packaging/macos/package.sh` | `Nexus Slide.app`, `NexusSlide-*.dmg`, `NexusSlide-cli-*.zip` chứa `nexus-slide-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store`/nền (gắn tên/icon DeckCraft); giấy phép/`NOTICE`/`ATTRIBUTION.md` vào `Contents/Resources`: từ 2026-10 dùng `copy_docs` của upstream (đã gọi `copy_notices` của Nexus), bỏ lời gọi `copy_notices` riêng | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.slides`, `/usr/bin/nexus-slide(-cli)`, `NexusSlide-*`, icon `nexus/assets/hicolor`, maintainer SonnyTech, zsync trỏ fork; AppImage giữ `usr/share/doc/nexus-slide` (giấy phép, `NOTICE`, `ATTRIBUTION.md`) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-slide`, vendor/homepage, binary, ghi công | Một phần |
| `packaging/common.sh` | `copy_docs` chép thêm `NOTICE` (Apache-2.0 §4(d)) và `ATTRIBUTION.md` (README Nexus trỏ tới cả hai) qua hàm mới `copy_notices` — bắt buộc, thiếu file thì lỗi; dùng cho deb/AppImage/tar, app + zip CLI macOS, zip web (và FreeBSD của upstream); 2026-10: upstream đã thêm `NOTICE` vào `copy_docs` (phần đó **đã có ở upstream, bỏ patch**), Nexus vẫn cần `ATTRIBUTION.md`, `LICENSE-*` bắt buộc, OFL của `assets/fonts` và `CRAFT_FONTS_REQUIRED`; viết lại theo cú pháp `[[ ]]`/`return` của upstream; vòng sửa 1: chép thêm `assets/fonts/OFL-*.txt` (font của repo nhúng trong binary, nếu có) sau OFL craft-fonts và báo lỗi khi trùng tên; mọi `cp` có `|| return 1` (gọi qua `||` thì errexit tắt); `copy_font_licences` báo lỗi khi `CRAFT_FONTS_REQUIRED=1` mà không có OFL | Nên PR upstream (ATTRIBUTION.md, lỗi khi thiếu file) |
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
| `.github/workflows/sonarcloud.yml` | Upstream (2026-10) thêm SonarCloud chạy cả trên PR với `secrets: inherit` gọi workflow `storytold/craft-common-actions/...@main` — trên fork sẽ trao mọi secret của fork (vd. `NEXUS_TOOLS_TOKEN`) cho code fork không kiểm soát. Thêm `if: github.repository == 'storytold/deckcraft'` | Không (an toàn bí mật) |
| `crates/model/src/trendline.rs` | Sửa tạm lỗi `clippy::manual_range_contains` (clippy 1.97) có sẵn ở upstream, để `clippy -D warnings` sạch (xem `nexus/UPSTREAM_PRS.md`); bỏ khi upstream sửa | Không (không phải thương hiệu) |

## Rà soát gọi mạng (guard §5)
`rg -n -i "artcraft|storytold|storyteller|discord|telemetry|update|crash|sentry|posthog|https?://" --type rust`:
không có telemetry/crash report/cập nhật tự động trong app. Link ArtCraft app tự mở: About (Discord,
getartcraft), Backstage Home/Account (Discord), nút Slide Show › Learn More (`APP_PAGE`), MCP
instructions/`websiteUrl`, CLI `links` — đã bỏ hoặc trỏ repo fork qua `links.rs`. MSI: URL ARP
getartcraft/Discord đã bỏ. AppImage zsync trỏ GitHub Releases của fork (`GITHUB_REPOSITORY` hoặc
`sonnytech-vn/deckcraft`). Dependency git `storytold/craft-libs` chỉ là nguồn build (T5/§4.8), không
phải gọi mạng lúc chạy.

## Việc còn nợ (đã allowlist có lý do, hoặc không bị brand-check bắt)
- Đề xuất PR upstream (white-label): hằng `APP_NAME`/`SETTINGS_DIR`/`APP_ID`/tên theme đầu lấy từ biến
  build (`DECKCRAFT_APP_NAME`…) → fork chỉ còn đặt biến, bỏ phần lớn patch ở trên.
- Theme dựng sẵn đầu tiên tên `Nexus`; tên upstream "DeckCraft" vẫn được chấp nhận khi tra theo tên
  (`nexus::theme_alias`) và cho ra `Nexus`. File cũ có theme tên `DeckCraft` mở và lưu lại giữ nguyên
  tên; chỉ Design › Variants (`theme_variants`, so tên chính xác trong `deckcraft-model`) không nhận ra
  nó là theme dựng sẵn và dựng biến thể từ màu của chính file (kết quả màu như nhau).
- ~~`crates/matroska/src/mux.rs` ghi `WritingApp = "deckcraft"`~~ — upstream (2026-10) bỏ crate `matroska`/
  `isobmff` khỏi repo (chuyển sang `craft-media` trong craft-libs): mục nợ này không còn trong fork.
- Upstream (2026-10) thêm trình tự cập nhật `craft_update` (kiểm tra/tải bản phát hành `storytold/deckcraft`):
  Nexus tắt trong `dialogs/updates_ui.rs` (xem bảng). `craft_update::self_update::finish` trong `main.rs` để
  nguyên: chỉ chạy khi được gọi với cờ helper, và trên Windows từ chối exe không tên `deckcraft.exe` (bản cài
  Nexus là `nexus-slide.exe`). Đề xuất PR upstream: cờ build tắt updater hoặc chọn `App` khác.
- Upstream (2026-10) vá egui 0.37/winit bằng fork git `storytold/egui`, `storytold/winit` (`[patch.crates-io]`):
  allowlist brand-check như nguồn build. **Bộ NexusOffice (nexus-suite, nexusmail) phải dùng đúng cùng
  rev egui khi nhúng app này.**
- Các file `packaging/windows/install-policy.*`, `check-install-policy.ps1`, `test-install-policy.ps1`,
  `INSTALLATION.md` của upstream (MSI dual-scope, thư mục ArtCraft) nằm trong repo nhưng Nexus không dùng
  (allowlist).
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
  menu thả/hộp thoại/khung tác vụ phần lớn còn tiếng Anh. Chuỗi mới của upstream có "DeckCraft" vẫn
  hiện nguyên cho tới khi sửa tại chỗ (brand-check bắt được).
