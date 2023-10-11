cargo test
rm -rf temp/
cargo run -- --root demo/ --pages demo/pages/ --outdir temp/
echo "---------------------------------"
diff -r demo_site temp
