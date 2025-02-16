use crate::display::menuopts::{
    DISPLAY_OPT_MAX_REC, DISPLAY_OPT_MAX_REDIRECTS, DISPLAY_OPT_REFERRER,
};

use self::menuopts::{
    DISPLAY_OPT_AUTH, DISPLAY_OPT_BODY, DISPLAY_OPT_CA_PATH, DISPLAY_OPT_CERT_INFO,
    DISPLAY_OPT_COMMAND_SAVED, DISPLAY_OPT_COOKIE, DISPLAY_OPT_FAIL_ON_ERROR,
    DISPLAY_OPT_FOLLOW_REDIRECTS, DISPLAY_OPT_HEADERS, DISPLAY_OPT_MATCH_WILDCARD,
    DISPLAY_OPT_OUTFILE, DISPLAY_OPT_PROGRESS_BAR, DISPLAY_OPT_PROXY_TUNNEL,
    DISPLAY_OPT_TCP_KEEPALIVE, DISPLAY_OPT_TOKEN_SAVED, DISPLAY_OPT_UNIX_SOCKET,
    DISPLAY_OPT_UNRESTRICTED_AUTH, DISPLAY_OPT_UPLOAD, DISPLAY_OPT_URL, DISPLAY_OPT_USERAGENT,
    DISPLAY_OPT_VERBOSE,
};

/*
* Display - This is For Structures That Represent Display Items
* Or Are Related To Display Items In Some Way
 */
// Input Options
pub mod inputopt;

// Menu Options
pub mod menuopts;

/// Here are the options that require us to display a box letting
/// the user know that they have selected that option.
#[derive(Debug, Clone, PartialEq)]
pub enum AppOptions {
    Verbose,
    // TODO: support more headers
    Headers(String),
    URL(String),
    Outfile(String),
    SaveCommand,
    Response(String),
    RecDownload(usize),
    Auth(String),
    SaveToken,
    UnixSocket(String),
    FollowRedirects,
    Cookie(String),
    EnableHeaders,
    ProgressBar,
    FailOnError,
    ProxyTunnel,
    CaPath(String),
    CertInfo,
    UserAgent(String),
    Referrer(String),
    MatchWildcard,
    TcpKeepAlive,
    UnrestrictedAuth,
    MaxRedirects(usize),
    UploadFile(String),
    RequestBody(String),
}

impl AppOptions {
    pub fn replace_value(&mut self, val: String) {
        match self {
            AppOptions::Headers(ref mut key) => {
                *key = val;
            }
            AppOptions::URL(ref mut url) => {
                *url = val;
            }
            AppOptions::Outfile(ref mut outfile) => {
                *outfile = val;
            }
            AppOptions::Response(ref mut response) => {
                *response = val;
            }
            AppOptions::RecDownload(ref mut level) => {
                *level = val.parse::<usize>().unwrap();
            }
            AppOptions::Auth(ref mut auth) => {
                *auth = val;
            }
            AppOptions::UnixSocket(ref mut socket) => {
                *socket = val;
            }
            AppOptions::Cookie(ref mut cookie) => {
                *cookie = val;
            }
            AppOptions::Referrer(ref mut referrer) => {
                *referrer = val;
            }
            AppOptions::CaPath(ref mut ca_cert) => {
                *ca_cert = val;
            }
            AppOptions::MaxRedirects(ref mut max_redirects) => {
                *max_redirects = val.parse::<usize>().unwrap();
            }
            AppOptions::UserAgent(ref mut ua) => {
                *ua = val;
            }
            AppOptions::UploadFile(ref mut file) => {
                *file = val;
            }
            AppOptions::RequestBody(ref mut body) => {
                *body = val;
            }
            _ => {}
        }
    }

    pub fn get_value(&self) -> String {
        match self {
            AppOptions::Verbose => String::from(DISPLAY_OPT_VERBOSE),
            AppOptions::URL(url) => format!("{}{}", DISPLAY_OPT_URL, url.clone()),
            AppOptions::Headers(val) => format!("{}{}", DISPLAY_OPT_HEADERS, val),
            AppOptions::Outfile(outfile) => format!("{}{}", DISPLAY_OPT_OUTFILE, outfile.clone()),
            AppOptions::SaveCommand => String::from(DISPLAY_OPT_COMMAND_SAVED),
            AppOptions::Response(response) => String::from(response),
            AppOptions::RecDownload(level) => {
                format!("{}{}", DISPLAY_OPT_MAX_REC, level)
            }
            AppOptions::Auth(auth) => format!("{}{}", DISPLAY_OPT_AUTH, auth.clone()),
            AppOptions::SaveToken => String::from(DISPLAY_OPT_TOKEN_SAVED),
            AppOptions::UnixSocket(socket) => {
                format!("{}{}", DISPLAY_OPT_UNIX_SOCKET, socket.clone())
            }
            AppOptions::EnableHeaders => DISPLAY_OPT_HEADERS.to_string(),
            AppOptions::ProgressBar => String::from(DISPLAY_OPT_PROGRESS_BAR),
            AppOptions::FailOnError => String::from(DISPLAY_OPT_FAIL_ON_ERROR),
            AppOptions::ProxyTunnel => DISPLAY_OPT_PROXY_TUNNEL.to_string(),
            AppOptions::UserAgent(ua) => format!("{}{}", DISPLAY_OPT_USERAGENT, ua),
            AppOptions::MaxRedirects(max_redirects) => {
                format!("{}{}", DISPLAY_OPT_MAX_REDIRECTS, max_redirects)
            }
            AppOptions::Cookie(cookie) => format!("{}{}", DISPLAY_OPT_COOKIE, cookie.clone()),
            AppOptions::Referrer(referrer) => {
                format!("{}{}", DISPLAY_OPT_REFERRER, referrer.clone())
            }
            AppOptions::CaPath(path) => format!("{}{}", DISPLAY_OPT_CA_PATH, path.clone()),
            AppOptions::CertInfo => DISPLAY_OPT_CERT_INFO.to_string(),
            AppOptions::FollowRedirects => DISPLAY_OPT_FOLLOW_REDIRECTS.to_string(),
            AppOptions::MatchWildcard => DISPLAY_OPT_MATCH_WILDCARD.to_string(),
            AppOptions::TcpKeepAlive => DISPLAY_OPT_TCP_KEEPALIVE.to_string(),
            AppOptions::UnrestrictedAuth => format!("{}{}", DISPLAY_OPT_UNRESTRICTED_AUTH, "󰄨"),
            AppOptions::UploadFile(file) => format!("{}{}", DISPLAY_OPT_UPLOAD, file.clone()),
            AppOptions::RequestBody(body) => format!("{}{}", DISPLAY_OPT_BODY, body.clone()),
        }
    }
}
