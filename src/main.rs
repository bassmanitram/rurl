use std::{fs::{self, File}, io::{stdin, BufRead, Read, Write}};

use curl::{easy::{Easy2, Handler, List, ReadError, /*WriteError*/}, Error};
use indexmap::IndexMap;
use utils::trim_ascii;
type StdError = Box<dyn std::error::Error>;

mod utils;

trait PerformAndReset {
    fn perform_and_reset(&mut self) -> Result<(),StdError>;
    fn is_performable(&self) -> bool;
}

impl PerformAndReset for Easy2<PayloadHandler> 
{
    fn perform_and_reset(&mut self) -> Result<(),StdError> {

        fn perform(this: &mut Easy2<PayloadHandler>)  -> Result<(),Error> {
            let handler = this.get_ref();
            let in_size = handler.input_file_len;
            if ! handler.request_headers.is_empty() {
                let mut list = List::new();
                for header in &handler.request_headers {
                    list.append(header)?;
                }
                this.http_headers(list)?
            }
            if in_size > 0 {
                this.post_field_size(in_size)?;
            }
            //eprintln!("Performing request: {:?}", this);
            //this.verbose(true)?;
            this.perform()
        }
        let r = if self.is_performable() { 
            let r = perform(self);
            let handler = self.get_mut();
            let mut ok = String::new(); 
            if r.is_ok() {
                handler.output_fields.values().for_each(|v|{if let Some(v) = v {ok.push_str(v)}; ok.push(';')});
                ok.pop(); 
            } else {
                ok.push_str("KO");
            };
            if ! handler.request_id.is_empty() {
                eprintln!("output: {} {}",ok,handler.request_id);
                println!("{} {}",ok,handler.request_id);
            } else {
                eprintln!("output: {} {}",ok,handler.request_number);
                println!("{} {}",ok,handler.request_number);
                handler.request_number += 1;
            }
            r
        } else { 
            //eprintln!("Not performable");
            Ok(()) 
        };
        self.get_mut().reset();
        self.reset();
        Ok(r?)
    }
    #[inline]
    fn is_performable(&self) -> bool {
        self.get_ref().url_set
    }
}

#[derive(Default,Debug)]
struct PayloadHandler {
    request_number: usize,
    output_file: Option<File>,
    input_file: Option<File>,
    input_file_len: u64,
    header_file: Option<File>,
    request_headers: Vec<String>,
    output_fields: IndexMap<String,Option<String>>,
    request_id: String,
    url_set: bool,
}
impl PayloadHandler {
    fn reset(&mut self) {
        self.url_set = false;
        self.input_file = None;
        self.input_file_len = 0;
        self.output_file = None;
        self.header_file = None;
        self.request_id.clear();
        self.request_headers.clear();
        self.output_fields.clear();
        self.output_fields.insert("status".to_string(), None);
    }
}
impl Handler for PayloadHandler {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, curl::easy::ReadError> {
        let size = if let Some(ref mut f) = self.input_file {
            match f.read(data) {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("failed input read: {}", e);
                    return Err(ReadError::Abort);
                }
            }
        } else {
            0
        };
        Ok(size)
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, curl::easy::WriteError> {
        if let Some(ref mut f) = self.output_file {
            if let Err(e) = f.write_all(data) {
                eprintln!("failed input read: {}", e);
                //return Err(WriteError::Abort)
                return Ok(0)
            }
        }
        Ok(data.len())
    }

    fn header(&mut self, data: &[u8]) -> bool {
        let data_str = unsafe{ std::str::from_utf8_unchecked(data) }; //safe - we would not have been given it if it wasn't a valid header string
        if data_str.starts_with("HTTP") {
            let mut els = data_str.splitn(3,' ');
            els.next();
            if let Some(status) = els.next() {
                self.output_fields.insert("status".to_string(),Some(status.to_string()));
            }
        } else if let Some((name,value)) = data_str.split_once(':') {
            self.output_fields.entry(name.trim().to_lowercase()).and_modify(|v| *v = Some(value.trim().to_string()));
        }
        if let Some(ref mut f) = self.header_file {
            f.write_all(trim_ascii(data)).expect("Failed header write");
            f.write_all(&[b'\n']).expect("Failed header write");
        }
        true
    }
}


fn main() {
    let mut stdin = stdin().lock();
    let mut line = String::new();
    let mut request: Easy2<PayloadHandler> = Easy2::new(PayloadHandler::default());

    loop {
        line.clear();
        eprintln!("awaiting line on stdin");
        match stdin.read_line(&mut line) {
            Ok(len) => {
                if len == 0 { 
                    eprintln!("executing and end");
                    handle_result(request.perform_and_reset(), "Request ended in error");
                    break;
                }
                let line = line.trim_end();
                if line.is_empty() {
                    eprintln!("executing and iterate");
                    handle_result(request.perform_and_reset(), "Request ended in error");
                    continue;
                }
                if line.starts_with(' ') {
                    if request.get_ref().url_set {
                        handle_result(add_option(&mut request, line.trim_start()), "Ignoring invalid option");
                    } else {
                        eprintln!("Option outside of request context: {}", line);
                    }
                } else {
                    eprintln!("executing");
                    handle_result(request.perform_and_reset(), "Request ended in error");
                    eprintln!("url: {}",line);
                    if handle_result(request.url(line).map_err(|e| e.into()),"Invalid URL") {
                        eprintln!("url set");
                        request.get_mut().url_set = true;
                    }
                }
            },
            Err(e) => {
                eprintln!("exiting on read error: {:?}",e);
                break
            }
        }
    }
}

fn handle_result(result: Result<(), StdError>, msg: &str) -> bool {
    if let Err(e) = result {
        eprintln!("{msg}: {:?}", e);
        false
    } else {
        true
    }
}

fn add_option(request: &mut Easy2<PayloadHandler>, option: &str) -> Result<(),Box<dyn std::error::Error>> {
    if let Some((name,value)) = option.split_once(':') {
        let name = name.trim().to_lowercase();
        let value = value.trim();
        eprintln!("add option {name}:{value}");
        match &name[..] {
            "method" => { 
                let method = value.to_uppercase();
                match &method[..] {
                    "GET" =>  { request.get(true)? },
                    "PUT" =>  { request.put(true)? },
                    "POST" => { request.post(true)?},
                    _ => {request.custom_request(&method)?}
                }
            },
            "header" => {
                request.get_mut().request_headers.push(value.to_string());
            },
            "request_id" => {
                request.get_mut().request_id.push_str(value);
            },
            "input_file" => { 
                let f = fs::File::open(value.trim_start())?;
                let payload_handler = request.get_mut();
                payload_handler.input_file_len = f.metadata()?.len();
                payload_handler.input_file = Some(f);
            },
            "output_file" => {
                let f = fs::OpenOptions::new().create(true).write(true).read(false).truncate(true).open(value.trim_start())?;
                request.get_mut().output_file = Some(f);
            },
            "header_file" => {
                let f = fs::OpenOptions::new().create(true).write(true).read(false).truncate(true).open(value.trim_start())?;
                request.get_mut().header_file = Some(f);
            },
            "fields" => {
                request.get_mut().output_fields.extend(&mut value.split(';').map(|s| (s.trim().to_lowercase(), None)));
            },
            _ => eprintln!("Ignoring unrecognized option: {option}"),
        }
    } else {
        eprintln!("Ignoring unrecognized option: {option}");
    }
    Ok(())
}
