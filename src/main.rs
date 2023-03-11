use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::path::Path;

const HTML_CONTENT_BEGIN: &str = "\r\n\r\n<!DOCTYPE html><html><head><meta http-equiv=\"content-type\" content=\"text/html;charset=utf-8\"><title>Moved</title></head><body>This page has moved <a href=\"";
const HTML_CONTENT_END: &str = "\">here</a>.</body></html>";
// This not contains begin 2 newline.
// Because that is part of header.
const HTML_CONTENT_SIZE: usize = 180;

fn main() {
    let mut bind_addr = "0.0.0.0:80";

    let raw_cfg:Vec<String>;

    if Path::new("./config").exists() {
        raw_cfg = read_file_lines("./config");

        for i in 0..raw_cfg.len() {
            if raw_cfg[i].starts_with("BIND_ADDR=") {
                (_, bind_addr) = raw_cfg[i].split_at(10);
            }
        }
    }

    let listener = TcpListener::bind(bind_addr).unwrap();

    println!("Bind {}", bind_addr);

    // accept connections and process them serially
    for stream in listener.incoming() {
        thread::spawn(|| {
            handle_client(stream.unwrap());
        });
    }
}

fn read_file_lines(filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    return BufReader::new(file)
        .lines()
        .map(|result|result.unwrap())
        .take_while(|line|!line.is_empty())
        .collect();
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut cmd:[&str; 3] = [&http_request.to_owned()[0], "", ""];

    for i in 0..2 {
        let split_point = cmd[i].find(' ');
        // Invalid type
        if split_point == None {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            return;
        }
        (cmd[i], cmd[i+1]) = cmd[i].split_at(split_point.unwrap());
        cmd[i+1] = cmd[i+1].trim_start();
    }

    // Required information store
    let mut host_information:Option<&str> = None;

    // Get some required information from http header
    for i in 0..http_request.len() {
        if http_request[i].starts_with("Host: ") {
            let (_, info) = http_request[i].split_at(6);
            host_information = Some(&info);
        }
    }

    if host_information == None {
        stream.write_all(
            "HTTP/1.1 400 Bad Request\r\n".as_bytes()
        ).unwrap();
        stream.shutdown(std::net::Shutdown::Both).unwrap();
        return;
    }

    let new_url:&str = &["https://", host_information.unwrap(), cmd[1]].concat();
    stream.write_all(
        (
            "HTTP/1.1 301 Moved Permanently\r\nLocation: ".to_owned() + new_url +
            "\r\nContent-Length: " + &format!("{}", HTML_CONTENT_SIZE + new_url.len()) +
            HTML_CONTENT_BEGIN + new_url + HTML_CONTENT_END
        ).as_bytes()
    ).unwrap();

    println!("{} http://{}{} => {}", cmd[0], host_information.unwrap(), cmd[1], new_url);

    stream.shutdown(std::net::Shutdown::Both).unwrap();
    return;
}