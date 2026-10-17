# BRANDING_PATCH

Danh sách file mà nhánh `nexus` sửa so với upstream vì lý do thương hiệu.
Mỗi mục là đường dẫn file chính xác trong dấu backtick (không dùng glob/thư mục).

Giá trị: sản phẩm **Nexus Slides**, bộ **NexusOffices**, nhà phát hành **Sonnytech**, app id
`vn.sonnytech.nexus.slides`, tài sản phát hành `NexusSlides-*`, binary cài đặt
`nexus-slides`/`nexus-slides-cli` (MSI, zip portable, deb/rpm/AppImage/tar, zip CLI macOS; tên build
Cargo vẫn là `deckcraft`/`deckcraft-cli`; bundle macOS `Nexus Slides.app`, executable `NexusSlides`).

Không đổi (spec §4.4): tên crate `deckcraft-*`, biến `DECKCRAFT_*`, đuôi `.deckcraft` và MIME
`application/x-deckcraft` (khoá định dạng; chỉ đổi tên hiển thị), `"format": "deckcraft"` và MIMETYPE
trong file `.deckcraft`, `serverInfo.name = "deckcraft"` của MCP (định danh máy), khoá plist
`DeckCraftVersion`/`DeckCraftBuildCommit`, id DOM `deckcraft_canvas`, tên file log `deckcraft.log`
(nằm trong thư mục cấu hình riêng của Nexus).

Giữ / đổi trong khai báo kiểu tệp macOS (`packaging/macos/Info.plist.in`): **đổi** UTI xuất
`ai.storyteller.deckcraft.presentation` → `vn.sonnytech.nexus.slides.presentation` (UTI gắn với bundle
id của app khai báo nó) và mô tả "Nexus Slides Presentation"; **giữ** đuôi `deckcraft`
(`public.filename-extension`), thẻ MIME `application/x-deckcraft` (`public.mime-type`) và
`UTTypeConformsTo`. Linux giữ cùng MIME type; Windows giữ đuôi `.deckcraft` (MIME `application/x-deckcraft` là định danh định dạng dùng chung).

## Cùng tồn tại với DeckCraft gốc (Review Focus #1)
- **Windows:** Nexus Slides **không sở hữu** `.deckcraft` (DeckCraft gốc sở hữu qua ProgId riêng). MSI chỉ
  ghi ProgId `Sonnytech.NexusSlides.Presentation` và giá trị có tên dưới `.deckcraft\OpenWithProgids`
  (như `.pptx/.potx/.ppsx`); không ghi giá trị mặc định hay `Content Type` của `HKLM\Software\Classes\.deckcraft`,
  nên cài/gỡ Nexus Slides không đổi hay xoá đăng ký của DeckCraft. Thư mục cài, khoá registry, UpgradeCode,
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
| `crates/engine/src/lib.rs` | `pub mod nexus;`; tác giả mặc định `nexus::DEFAULT_AUTHOR` = "Nexus Slides User" (ký bình luận ghi vào file; upstream là "Presenter") | Có — hằng qua biến build |
| `crates/engine/src/links.rs` | Bỏ `DISCORD`/`WEBSITE`/`APPS` (hạ tầng ArtCraft); `APP_PAGE`/`GITHUB`/`ISSUES` trỏ repo fork | Có |
| `crates/engine/src/recovery.rs` | Thư mục AutoRecover riêng (`Nexus Slides/Recovery`, `nexus-slides/recovery`) — không mở bản phục hồi của DeckCraft | Có — `SETTINGS_DIR` |
| `crates/engine/src/spell.rs` | Thư mục từ điển riêng + `user.dic` (Add to Dictionary) trong thư mục Nexus | Có — `SETTINGS_DIR` |
| `crates/engine/src/sample.rs` | Deck mẫu: theme `Nexus`, "made in Nexus Slides", "Made with Nexus Slides" | Có |
| `crates/engine/src/cmd/file.rs` | Lỗi mở file (mật khẩu, không đọc được) nêu Nexus Slides; `theme_named` tra qua `nexus::theme_alias` | Có |
| `crates/engine/src/cmd/format.rs` | Lỗi `font.add` nêu Nexus Slides | Có |
| `crates/engine/src/cmd/insert.rs` | Theme mặc định của Photo Album = `nexus::THEME_NAME`; tra theme qua `nexus::theme_alias`; 2 assert test theo tên mới | Có |
| `crates/engine/src/cmd/design.rs` | `design.theme`/`design.colors`/`design.effects` tra tên qua `nexus::theme_alias` (design.colors kèm màu giữ nguyên tên người dùng đặt, kể cả "DeckCraft"); test: tên theme/effect scheme `Nexus` | Đi cùng `theme.rs` |
| `crates/engine/src/templates.rs` | Starter template tra theme qua `nexus::theme_alias` (starter mới của upstream ghi "DeckCraft" vẫn chạy) | Đi cùng `theme.rs` |
| `crates/engine/src/tests.rs` | Test caret dùng theme `Nexus` | Đi cùng `theme.rs` |
| `crates/model/src/theme.rs` | Theme/colour/format scheme dựng sẵn đầu tiên tên `Nexus` (ghi vào file) | Có — tên theme qua biến build |
| `crates/model/src/tests.rs` | Assert tên theme mới | Đi cùng `theme.rs` |
| `crates/format/src/lib.rs` | `generator: "Nexus Slides <ver>"` ghi vào `.deckcraft`; lỗi "not a Nexus Slides file", media quá lớn (`format` giữ `deckcraft`) | Có |
| `crates/pdf/src/lib.rs` | PDF `/Creator` và `/Producer` = "Nexus Slides" | Có — tên app qua tham số |
| `crates/pdf/src/tests.rs` | Assert Creator/Producer mới | Đi cùng `lib.rs` |
| `crates/pptx/src/write/mod.rs` | `docProps/app.xml` `<Application>Nexus Slides</Application>` | Có |
| `crates/pptx/src/tables.rs` | Tên table style tự sinh "Nexus {n}" ghi vào `tableStyles.xml` (GUID không đổi) | Có |
| `crates/mcp/src/server.rs` | `serverInfo.title` = "Nexus Slides", `websiteUrl` repo fork, INSTRUCTIONS; bỏ link Discord; **`serverInfo.name` giữ `deckcraft`** | Có |
| `crates/mcp/src/backend.rs` | Thông báo kết nối app nêu Nexus Slides | Có |
| `crates/mcp/src/headless.rs` | Gợi ý `nexus-slides --control 7979` / `nexus-slides-cli mcp --connect` | Có |
| `crates/mcp/src/tools.rs` | Mô tả tool: `nexus-slides-cli mcp --connect`, theme `Nexus`, cửa sổ Nexus Slides | Có |
| `crates/ui-egui/src/lib.rs` | `pub mod nexus;`; nhãn `app.about` "About Nexus Slides"; `app.links` "Project Links" trả {appPage, github, issues}; 2 thông báo trạng thái | Có — `APP_NAME` |
| `crates/ui-egui/src/dialogs.rs` | Tiêu đề About/quit/mặc định; About → `nexus::about_ui` (bỏ link Discord/getartcraft, ghi công chữ); `paint_logo` → logo Nexus; theme album `Nexus`; Find "Nexus Slides can't find" | Có |
| `crates/ui-egui/src/backstage.rs` | Home/Account: tên + logo Nexus, bỏ 2 link Discord ArtCraft, thêm câu ghi công; định dạng "Nexus Slides Presentation (*.deckcraft)"; câu in qua PDF; ô mẫu dùng theme `Nexus` | Có |
| `crates/ui-egui/src/menus.rs` | Help › About Nexus Slides | Có |
| `crates/ui-egui/src/ribbon.rs` | Tiêu đề thanh tiêu đề khi chưa mở bài = `APP_NAME` | Có |
| `crates/ui-egui/src/show.rs` | Cửa sổ "Presenter View — Nexus Slides" | Có |
| `crates/ui-egui/src/backstage_tests.rs` | Assert nhãn/thông báo mới | Đi cùng |
| `crates/ui-egui/src/effects_tests.rs` | Assert theme `Nexus` | Đi cùng |
| `crates/ui-egui/src/theme_colors_tests.rs` | Assert colour scheme `Nexus` | Đi cùng |
| `apps/deckcraft/src/main.rs` | Tiêu đề cửa sổ, app id, tên eframe, USAGE/`--version` (`nexus-slides`), log, bộ lọc file, icon cửa sổ từ `nexus/assets/`, thư mục cấu hình riêng `Nexus Slides` / `nexus-slides` (không đụng cấu hình DeckCraft) | Có |
| `apps/deckcraft/build.rs` | Resource Windows: icon Nexus, ProductName, CompanyName Sonnytech, LegalCopyright (ghi công), OriginalFilename `nexus-slides.exe` | Có |
| `apps/deckcraft-cli/src/main.rs` | `--version`, tiền tố lỗi, gợi ý `nexus-slides-cli`/`nexus-slides --control`, `links` (bỏ Discord/website), dòng "Source:" thay "Community:" | Có |
| `apps/deckcraft-cli/src/help.rs` | Trợ giúp dùng `nexus-slides-cli`, "Nexus Slides presentations", `links` mô tả mới | Có |
| `apps/deckcraft-cli/tests/cli.rs` | Assert trợ giúp theo tên `nexus-slides-cli` | Đi cùng `help.rs` |
| `apps/deckcraft-web/index.html` | `<title>Nexus Slides</title>`, favicon Nexus, "Loading Nexus Slides…" | Có |
| `apps/deckcraft-web/src/web.rs` | Lỗi khởi động bản web | Có |
| `packaging/windows/deckcraft.wxs` | MSI: Name/Manufacturer Sonnytech, **UpgradeCode + GUID component mới**, cài `nexus-slides.exe`/`nexus-slides-cli.exe` vào `Program Files\Nexus Slides`, ProgId `Sonnytech.NexusSlides.Presentation` (ghi bằng RegistryValue), **`.deckcraft` chỉ "Open with"** (`.deckcraft\OpenWithProgids`, không ghi giá trị mặc định/Content Type của `.deckcraft` — DeckCraft gốc sở hữu đuôi này), `Applications\nexus-slides.exe`, App Paths; bỏ URL ARP getartcraft/Discord (About → repo fork); cài `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` cạnh exe (4 component mới, GUID tự sinh) | Một phần (biến WiX) |
| `packaging/windows/package.ps1` | `NexusSlides-*.msi/.zip`, icon Nexus; zip portable chứa `nexus-slides(.exe)`/`nexus-slides-cli.exe` như MSI; chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào thư mục stage (MSI cài từ đó) và zip portable (Apache-2.0 §4(d)) | Một phần |
| `packaging/windows/sign.ps1` | Mô tả chữ ký `/d Nexus Slides`, `/du` repo fork | Một phần |
| `packaging/macos/Info.plist.in` | Tên bundle, executable `NexusSlides`, id `vn.sonnytech.nexus.slides`, icon, UTI `.deckcraft` mới, copyright (ghi công) | Một phần |
| `packaging/macos/package.sh` | `Nexus Slides.app`, `NexusSlides-*.dmg`, `NexusSlides-cli-*.zip` chứa `nexus-slides-cli`, icns Nexus, tên volume; không chép `dmg-layout.DS_Store`/nền (gắn tên/icon DeckCraft); `copy_notices` chép `LICENSE-MIT`/`LICENSE-APACHE`/`NOTICE`/`ATTRIBUTION.md` vào `Contents/Resources` của app (trước khi ký) | Một phần |
| `packaging/linux/package.sh` | `APP_ID=vn.sonnytech.nexus.slides`, `/usr/bin/nexus-slides(-cli)`, `NexusSlides-*`, icon `nexus/assets/hicolor`, maintainer Sonnytech, zsync trỏ fork; AppImage giữ `usr/share/doc/nexus-slides` (giấy phép, `NOTICE`, `ATTRIBUTION.md`) thay vì bỏ ra ngoài | Một phần |
| `packaging/linux/nfpm.yaml` | Gói `nexus-slides`, vendor/homepage, binary, ghi công | Một phần |
| `packaging/common.sh` | `copy_docs` chép thêm `NOTICE` (Apache-2.0 §4(d)) và `ATTRIBUTION.md` (README Nexus trỏ tới cả hai) qua hàm mới `copy_notices` — bắt buộc, thiếu file thì lỗi; dùng cho deb/AppImage/tar, zip CLI macOS, zip web (và FreeBSD của upstream) | Nên PR upstream (gói upstream cũng thiếu NOTICE) |
| `packaging/linux/vn.sonnytech.nexus.slides.desktop` | Đổi tên từ `ai.storyteller.deckcraft.desktop`: Name, Exec/TryExec `nexus-slides`, Icon, StartupWMClass | Một phần |
| `packaging/linux/vn.sonnytech.nexus.slides.metainfo.xml.in` | Đổi tên từ `ai.storyteller.deckcraft.metainfo.xml.in`: id, tên, developer Sonnytech, URL fork, bỏ Discord và ảnh chụp trên storytold, binary, ghi công | Một phần |
| `packaging/linux/vn.sonnytech.nexus.slides.mime.xml` | Đổi tên từ `ai.storyteller.deckcraft.mime.xml`: **giữ MIME `application/x-deckcraft` + glob `*.deckcraft`**, chỉ đổi tên hiển thị và icon | Một phần |
| `packaging/web/package.sh` | Zip `NexusSlides-web-<ver>.zip` | Một phần |
| `packaging/web/README.md` | Được đóng gói thành `HOSTING.md` trong zip web → tên sản phẩm/tên zip/đường dẫn ví dụ | Không (tài liệu đi kèm gói) |
| `omastore.toml` | Icon, tên tài sản `NexusSlides-*`, exec `bin/nexus-slides` | Không |
| `README.md` | README Nexus Slides ngắn (vi trước, có English): sản phẩm, NexusOffices/Sonnytech, câu ghi công "Nexus Slides được phát triển dựa trên DeckCraft của ArtCraft (MIT/Apache-2.0)" chỉ bằng chữ, trỏ LICENSE-MIT/LICENSE-APACHE/NOTICE/ATTRIBUTION; được `copy_docs` và zip portable đóng gói | Không (README của fork) |
| `ATTRIBUTION.md` | 14 dòng cho icon Nexus (`nexus/assets/**`, Sonnytech, CC0-1.0) | Không (bắt buộc theo luật asset) |

### GUID WiX mới (Nexus Slides riêng, không dùng chung với DeckCraft hay app Nexus khác)

| Mục | GUID cũ (upstream DeckCraft) | GUID mới (Nexus Slides) |
|---|---|---|
| `UpgradeCode` | `6F9C69E7-7825-40D2-BF26-C274DA676425` | `96D068F9-30CC-4E22-808D-242613C1E158` |
| Component `DeckcraftApp` | `D87EA444-41A4-4C19-922E-66D1A29461FF` | `F3911F7B-4F9F-4BA6-8D06-B513AB237AF1` |
| Component `DeckcraftCli` | (tự sinh theo đường dẫn) | tự sinh — thư mục cài (`Nexus Slides`) và tên file (`nexus-slides-cli.exe`) đã khác |
| Component `DeckcraftMitLicense`, `DeckcraftApacheLicense`, `DeckcraftNotice`, `DeckcraftAttribution` | (không có ở upstream) | tự sinh — file giấy phép/NOTICE/ATTRIBUTION.md cài cạnh exe |

Sinh bằng `python3 -c "import uuid; print(uuid.uuid4())"` ngày 2026-10-17.

### File mới (không tính là patch)
`crates/engine/src/nexus.rs`, `crates/ui-egui/src/nexus.rs`, `crates/engine/tests/nexus_brand.rs`
(tác giả mặc định, `<Application>`, theme `Nexus`, generator `.deckcraft`, bí danh theme "DeckCraft",
deck upstream giữ tên theme, PDF Creator/Producer, tên table style "Nexus …"),
`crates/mcp/tests/nexus_brand.rs` (serverInfo, tool_definitions không có `deckcraft-cli`),
`apps/deckcraft-cli/tests/nexus_brand.rs` (`--version`/usage/help/links/app nêu `nexus-slides(-cli)`),
`nexus/assets/*` (logo tạm `nexus-slides.svg` + PNG/ICO/ICNS/hicolor sinh bằng
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
- Tác giả mặc định đổi từ "Presenter" (upstream, không mang thương hiệu) sang "Nexus Slides User"
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
