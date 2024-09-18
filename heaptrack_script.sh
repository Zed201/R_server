# Usando o heaptrack do KDE e o zstd(compressor de arquivos)
cargo build --release
heaptrack ./target/release/r_server 8000
unzstd *.zst
rm *.zst
heaptrack_print heaptrack.r_server* > result.txt
tail result.txt
rm heaptrack.r_server*