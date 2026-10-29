# UPSTREAM_PRS

PR gửi lên upstream (storytold) để giảm số patch của nhánh `nexus`.

| PR | Repo | Nội dung | Trạng thái | Đã cherry-pick vào nexus? |
|---|---|---|---|---|
| (chưa gửi — chờ Henry duyệt) | storytold/deckcraft | Ứng viên: `crates/model/src/trendline.rs:468` `if a >= 1.0e6 \|\| a < 1.0e-3` bị `clippy::manual_range_contains` (clippy 1.97, `-D warnings`) → `!(1.0e-3..1.0e6).contains(&a)`. Lỗi có sẵn của upstream (vẫn còn ở upstream f893f6a, 2026-10-28), không phải do fork. Từ đồng bộ 2026-10-29 fork **sửa tạm** dòng này (`v` đã hữu hạn và khác 0 ở đó nên hai cách viết tương đương, NaN không tới được) để `clippy -D warnings` sạch với clippy 1.97; bỏ khi upstream sửa | Đề xuất | Có (patch tạm trong `crates/model/src/trendline.rs`) |
| (chưa gửi — chờ Henry duyệt) | storytold/deckcraft | Ứng viên: cờ build/biến tắt trình cập nhật `craft_update` hoặc chọn `App`/repo khác, để fork không phải sửa `crates/ui-egui/src/dialogs/updates_ui.rs` | Đề xuất | — |
