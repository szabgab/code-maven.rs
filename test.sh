set -e
cargo test
rm -rf temp/
cargo build --release
./target/release/code-maven-web --root demo/ --pages demo/pages/ --outdir temp/
rm -rf temp/img
echo "---------------------------------"
diff -r demo_site temp
