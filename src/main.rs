use alfred;
use chrono_humanize::HumanTime;
use serde_json::json;
use std::io::Write;
use std::{env, io};

mod torrent;
mod transmission;

fn write_output(rerun: i32, items: &[alfred::Item]) {
    let mut alfred_items = alfred::json::Builder::with_items(items).into_json();
    let alfred_output = alfred_items.as_object_mut().unwrap();
    // we add `rerun` to the output to re-run the script after `rerun` seconds
    alfred_output.insert(String::from("rerun"), serde_json::to_value(rerun).unwrap());
    io::stdout()
        .write_all(json!(alfred_output).to_string().as_bytes())
        .unwrap();
}

fn main() {
    let pattern = env::args().nth(1).unwrap_or(String::from(""));
    let transmission_url = env::var("transmission_web_url")
        .unwrap_or(String::from("http://localhost:9091/transmission/rpc"));
    let rerun = match std::env::var("rerun") {
        Ok(value) => value.parse::<i32>().unwrap_or(1),
        Err(_) => 1,
    };
    let mut client = transmission::Client::new(transmission_url);
    match client.search(&pattern) {
        Ok(torrents) => {
            let output = torrents
                .iter()
                .rev()
                .map(|torrent| {
                    use crate::torrent::TorrentStatus::*;
                    let location = format!("{}/{}", torrent.download_dir, torrent.name);
                    match torrent.status() {
                        Finished | Paused => alfred::ItemBuilder::new(&torrent.name)
                            .subtitle(torrent.status().to_string())
                            .arg(location)
                            .into_item(),
                        Active => {
                            let remaining = HumanTime::from(torrent.eta);
                            let speed = byte_unit::Byte::from_bytes(torrent.rate_download)
                                .get_appropriate_unit(false);
                            alfred::ItemBuilder::new(&torrent.name)
                                .subtitle(format!(
                                    "{:.1}% - {} ({}/s)",
                                    torrent.percent_done * 100.0,
                                    remaining,
                                    speed
                                ))
                                .arg(location)
                                .into_item()
                        }
                    }
                })
                .collect::<Vec<alfred::Item>>();
            write_output(rerun, &output);
        }
        Err(e) => {
            write_output(
                rerun,
                &[alfred::ItemBuilder::new("Error :(")
                    .subtitle(e.to_string())
                    .into_item()],
            );
        }
    }
}
