



PHONY: mod5_1.2_check_build_broken_app
mod5_1.2_check_build_broken_app:
	cargo check \
		--manifest-path ./repos/origin-of-broken-app/Cargo.toml
	cargo test \
		--manifest-path ./repos/origin-of-broken-app/Cargo.toml

# cargo test \
# 		--manifest-path ./repos/origin-of-broken-app/Cargo.toml sums_even_numbers




PHONY: mod5_run_sanitize
mod5_run_sanitize:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly run --bin demo


PHONY: mod5_2_3_run_miri
mod5_2_3_run_miri:
# 	MIRIFLAGS=-Zmiri-env-forward=RUST_BACKTRACE cargo +nightly miri test 
	MIRIFLAGS=-Zmiri-backtrace=full cargo  miri test --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml  2>&1 | tee artifacts/generated/2_3_miri_after_hot_fix.log

PHONY: mod5_2_4_fix_after_miri
mod5_2_4_fix_after_miri:
	cargo  miri test --manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml 2>&1 | tee artifacts/generated/2_4_fix_leak_buffer_after_miri.log

PHONY: mod5_2_5_1_valgrind_asan
mod5_2_5_1_valgrind_asan:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml 2>&1 | tee artifacts/generated/mod5_2_5_1_valgrind_asan_first_one.log

PHONY: mod5_2_5_2_valgrind_tsan
mod5_2_5_2_valgrind_tsan:
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly test \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml  2>&1 \
		| tee artifacts/generated/mod5_2_5_2_valgrind_tsan_first_one.log 


PHONY: mod5_2_6_1_prove_leak_is_not_by_demo
mod5_2_6_1_prove_leak_is_not_by_demo:
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly run --bin demo \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml  2>&1 \
		| tee artifacts/generated/mod5_2_6_1_prove_leak_is_not_by_demo.log

PHONY: mod5_2_6_1_prove_leak_is_not_by_lib_rs___via_benches
mod5_2_6_1_prove_leak_is_not_by_lib_rs___via_benches:
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly bench --bench baseline \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml  2>&1 \
		| tee artifacts/generated/mod5_2_6_1_prove_leak_is_not_by_lib_rs___via_benches.log 

PHONY: mod5_3_1_1_add_unit_tests_for_leak_buff
mod5_3_1_1_add_unit_tests_for_leak_buff:
	cargo test \
		--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml 2>&1 \
		| tee artifacts/generated/broken-app-3.1.1-add-unit-tests-for-leak-buff.log

PHONY: mod5_3_1_2_check_after_hot_fix
mod5_3_1_2_check_after_hot_fix:
	RUSTFLAGS="-Awarnings" cargo  miri test --manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml && \
	RUSTFLAGS="-Zsanitizer=address -Awarnings" cargo +nightly test  \
			--target x86_64-unknown-linux-gnu \
			--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml && \
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" cargo +nightly run --bin demo \
			--target x86_64-unknown-linux-gnu \
			--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml 2>&1 \
		| tee artifacts/generated/mod5_3_1_2_check_after_hot_fix.log


PHONY: mod5_3.3-extra-tests-for-normalyze-and-threat
mod5_3.3-extra-tests-for-normalyze-and-threat:
	cargo test \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 2>&1 \
		| tee artifacts/generated/broken-app-3.3-extra-tests-for-normalyze-and-threat.log

PHONY: mod5_3.3-run-bin-demo_for_threats
mod5_3.3-run-bin-demo_for_threats:
	cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 

PHONY: mod5_3.3.2-run-bin-demo_for_threats-via-tsan
mod5_3.3.2-run-bin-demo_for_threats-via-tsan:
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" \
		cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 2>&1 \
		| tee artifacts/generated/broken-app-3.3-run-bin-demo_for_threats-via-tsan.log

PHONY: mod5_3.3.3-fix-tsan-for-demo-for-threats
mod5_3.3.3-fix-tsan-for-demo-for-threats:
	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" \
		cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml 2>&1 \
		| tee artifacts/generated/broken-app-3.3.3-fix-tsan-for-demo-for-threats.log
		

PHONY: mod5_4_1_profile_for_demo_bin
mod5_4_1_profile_for_demo_bin:
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
    --manifest-path ./repos/reference-app/Cargo.toml \
    --release --root --bench baseline \
    --output artifacts/generated/reference-app-flamegraph-test-integration.svg

PHONY: mod5_4_1_profile_for_bench
mod5_4_1_profile_for_bench:
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
    --manifest-path ./repos/reference-app/Cargo.toml \
    --release --root --bench baseline \
    --output artifacts/generated/reference-app-flamegraph-test-integration.svg

PHONY: mod5_5_1_1_fixup_criterion
mod5_5_1_1_fixup_criterion:
	RUSTFLAGS="-Awarnings" cargo bench  --bench criterion \
		--manifest-path ./repos/broken-app-5.1-fixup-criterion-for-broken-app/Cargo.toml 2>&1 \
		| tee artifacts/generated/broken-app-5.1.1-fixup-criterion.log

PHONY: mod5_6_1_1_optimyze_sum_even_get_criterion
mod5_6_1_1_optimyze_sum_even_get_criterion:
	RUSTFLAGS="-Awarnings" cargo bench  --bench criterion \
		--manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- sum_even 2>&1 \
		| tee artifacts/generated/6_1_1_optimyze_sum_even_get_criterion.log



# 	RUSTFLAGS="-Awarnings" cargo  miri test --manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml && \
# 	RUSTFLAGS="-Zsanitizer=address -Awarnings" cargo +nightly test  \
# 			--target x86_64-unknown-linux-gnu \
# 			--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml && \
# 	RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" cargo +nightly run --bin demo \
# 			--target x86_64-unknown-linux-gnu \
# 			--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml 2>&1 \
# 		| tee artifacts/generated/mod5_3_1_2_check_after_hot_fix.log



PHONY: mod5_run_sanitize_sums_even_numbers
mod5_run_sanitize_sums_even_numbers:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
# 	RUST_BACKTRACE=1 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
	
PHONY: mod5_run_sanitize_sums_even_numbers
mod5_run_sanitize_sums_even_numbers:
	cargo +nightly build --lib
# 	cargo +nightly test --target x86_64-unknown-linux-gnu sums_even_numbers
	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
# 	RUST_BACKTRACE=1 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers




PHONY: mod5_run_sanitize_counts_non_zero_bytes
mod5_run_sanitize_counts_non_zero_bytes:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly build --lib -- -Zsanitizer=address
# 	cargo +nightly test --target x86_64-unknown-linux-gnu sums_even_numbers
	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  counts_non_zero_bytes

PHONY: mod5_test_counts_non_zero_bytes_valgrind
mod5_test_counts_non_zero_bytes_valgrind:
	cargo valgrind test counts_non_zero_bytes
# 	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  counts_non_zero_bytes






PHONY: mod5_get_bench_of_fixed_broken_app
mod5_get_bench_of_fixed_broken_app:
	cargo bench  --bench baseline  > artifacts/generated/broken-app-benches.txt

PHONY: mod5_get_criterion_of_fixed_broken_app
mod5_get_criterion_of_fixed_broken_app:
	cargo bench  --bench criterion  > artifacts/generated/broken-app-criterion.txt


