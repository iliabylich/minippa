use axum::http::HeaderMap;

pub(crate) async fn install_sh(headers: HeaderMap) -> String {
    let Some(host) = headers.get("Host") else {
        return r#"echo "no Host header given""#.to_string();
    };
    let Ok(host) = host.to_str() else {
        return r#"echo "non-utf8 Host header""#.to_string();
    };

    let protocol = if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    };

    let url = format!("{protocol}://{host}");
    let gpg_install_path = "/etc/apt/trusted.gpg.d/minippa.gpg";

    format!(
        r#"set -euo pipefail

echo "Installing mini PPA {url}"

echo "Installing GPG key"
curl --silent "{url}/public.gpg" | gpg --dearmor | tee "{gpg_install_path}" > /dev/null

echo "Installing APT sources file"
cat << EOF > /etc/apt/sources.list.d/minippa.sources
Types: deb
URIs: {url}/
Suites: ./
Components:
Signed-By: {gpg_install_path}
EOF
"#
    )
}
