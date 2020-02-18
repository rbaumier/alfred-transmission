use crate::torrent::Torrent;
use serde::Deserialize;

const SESSION_ID_HEADER: &str = "X-Transmission-Session-Id";
const TRANSMISSION_REQUEST_PAYLOAD: &str = r#"{
    "method": "torrent-get",
    "arguments": {
        "fields": [
            "id",
            "name",
            "eta",
            "rateDownload",
            "percentDone",
            "downloadDir",
            "addedDate",
            "status",
            "isFinished"
        ]
    }
}"#;

#[derive(Deserialize, Debug, Clone)]
pub struct Torrents {
    torrents: Vec<Torrent>,
}

#[derive(Deserialize, Debug)]
struct TransmissionResponse {
    arguments: Torrents,
}

pub struct Client {
    url: String,
    session_id: String,
}

impl Client {
    pub fn new(url: String) -> Client {
        Client {
            url,
            session_id: String::from(""),
        }
    }

    pub fn search(&mut self, pattern: &str) -> Result<Vec<Torrent>, reqwest::Error> {
        reqwest::blocking::Client::new()
            .post(&self.url)
            .header(SESSION_ID_HEADER, &self.session_id)
            .body(TRANSMISSION_REQUEST_PAYLOAD)
            .send()
            .and_then(|res| {
                // https://github.com/transmission/transmission/blob/master/extras/rpc-spec.txt#L56
                // a possible improvement could be to store the session_id into alfred cache and reuse it,
                // but since the request is made locally (by default) this is not much of a problem
                if res.status() == 409 {
                    self.session_id = res
                        .headers()
                        .get(SESSION_ID_HEADER)
                        .expect(&format!("transmission web client didn't return a valid {} header, cannot proceed", SESSION_ID_HEADER))
                        .to_str()
                        .unwrap()
                        .to_string();
                    return self.search(pattern);
                }
                res.json::<TransmissionResponse>()
                    .map(|data| data.arguments.torrents)
                    .map(|torrents| {
                        torrents
                            .into_iter()
                            .filter(|torrent| {
                                torrent
                                    .name
                                    .to_lowercase()
                                    .contains(&pattern.to_lowercase())
                            })
                            .collect::<Vec<Torrent>>()
                    })
            })
    }
}
