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
- Không có hệ thống i18n: chuỗi giao diện mới của upstream có "DeckCraft" sẽ hiện nguyên cho tới khi sửa
  tại chỗ (brand-check bắt được vì allowlist chỉ cho chú thích/test ở file cụ thể).
