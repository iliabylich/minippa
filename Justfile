default:
    @just --list

clean:
    rm -f data-dir/InRelease data-dir/Packages data-dir/Packages.gz data-dir/Release data-dir/Release.gpg data-dir/*.deb

key-delete:
    gpg --delete-secret-and-public-keys "owner@this-repo.org"

key-generate:
    RUST_LOG=info cargo run -- --generate-key

run:
    RUST_LOG=info cargo run -- --start-server
