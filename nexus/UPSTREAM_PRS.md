# UPSTREAM_PRS

PR gửi lên upstream (storytold) để giảm số patch của nhánh `nexus`.

| PR | Repo | Nội dung | Trạng thái | Đã cherry-pick vào nexus? |
|---|---|---|---|---|
| (chưa gửi — chờ Henry duyệt) | storytold/deckcraft | Ứng viên: `crates/model/src/trendline.rs:468` `if a >= 1.0e6 \|\| a < 1.0e-3` bị `clippy::manual_range_contains` (clippy 1.97, `-D warnings`) → `!(1.0e-3..1.0e6).contains(&a)`. Lỗi có sẵn của upstream, không phải do fork; fork **không** thêm allow | Đề xuất | Không |
