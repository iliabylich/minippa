pub(crate) fn make_install_script(base_url: String) -> String {
    let key_path = "/etc/apt/trusted.gpg.d/minippa.gpg";

    format!(
        r#"set -euo pipefail

echo "Installing mini PPA {base_url}"

echo "Installing GPG key"
curl --silent "{base_url}/public.gpg" | gpg --dearmor | tee "{key_path}" > /dev/null

echo "Installing APT sources file"
cat << EOF > /etc/apt/sources.list.d/minippa.sources
Types: deb
URIs: {base_url}/
Suites: ./
Components:
Signed-By: {key_path}
EOF
"#
    )
}
