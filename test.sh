set -e
cargo test
rm -rf temp/
cargo build --release
./target/release/code-maven web --root test_cases/demo/ --outdir temp/
rm -rf temp/img
echo "---------------------------------"
diff -r demo_site temp
