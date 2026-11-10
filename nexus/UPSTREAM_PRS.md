# UPSTREAM_PRS

PR gửi lên upstream (storytold) để giảm số patch của nhánh `nexus`.

| PR | Repo | Nội dung | Trạng thái | Đã cherry-pick vào nexus? |
|---|---|---|---|---|
| — (upstream tự sửa) | storytold/deckcraft | `crates/model/src/trendline.rs`: `clippy::manual_range_contains` (`!(1.0e-3..1.0e6).contains(&a)`). Upstream đã sửa trong #1561 (`505a6f28`); đồng bộ 2026-11-09 xác nhận file bằng hệt upstream, fork không còn patch | Xong ở upstream | Không cần |
| (chưa gửi — chờ Henry duyệt) | storytold/deckcraft | Ứng viên: cờ build/biến tắt trình cập nhật `craft_update` hoặc chọn `App`/repo khác, để fork không phải sửa `crates/ui-egui/src/dialogs/updates_ui.rs` | Đề xuất | — |
| (chưa gửi — chờ Henry duyệt) | storytold/deckcraft | Ứng viên (độ bền dữ liệu AutoRecover, `crates/engine/src/recovery.rs`): `write_atomic` gọi `sync_all` tệp tạm **trước** `rename` (và fsync thư mục trên Unix), xoá tệp tạm khi ghi lỗi — trước đây mất điện có thể để lại mục rỗng vì đổi tên tới đĩa trước dữ liệu. Kèm theo nếu upstream muốn: ghi metadata trước bản trình chiếu (instance khác không coi mục đang ghi là của sự cố), `file.recovery.discard` không xoá mục của instance khác đang chạy | Đề xuất | Không (nằm ở nhánh `nx/recovery-hooks`, PR #14) |
