use std::{path::PathBuf, fs, fmt, process::{Command, Stdio}, time::Duration, error::Error, sync::Arc, io::{BufReader, prelude::BufRead}};
use sysinfo::{System, SystemExt, Pid, Signal, ProcessExt};
use reqwest::{Method, IntoUrl};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::utils::error::{ApplicationError, ApiResultError};

use super::{config::LocalConfig, channel::ChannelManager};

pub struct ServerManager {
    server_path: PathBuf,
    appdata_path: PathBuf,
    channel: String,
    client: Arc<reqwest::Client>,
    access: Access
}

#[derive(Clone)]
struct Access {
    address: Option<String>,
    token: Option<String>
}

impl ServerManager {
    pub fn new(config: &LocalConfig, channel_manager: &ChannelManager) -> ServerManager {
        ServerManager { 
            server_path: config.work_path.server_path.clone(),
            appdata_path: config.work_path.appdata_path.clone(),
            channel: channel_manager.current_channel().to_string(),
            client: Arc::new(reqwest::Client::new()),
            access: Access { address: Option::None, token: Option::None }
        }
    }
    pub async fn status(&mut self) -> ServerStatus {
        let pid_file = self.read_pid_file();
        if pid_file.is_none() {
            return ServerStatus { status: ServerStatusType::Stop, pid: Option::None, port: Option::None, start_time: Option::None }
        }
        let pid_file = pid_file.unwrap();
        if pid_file.port.is_none() || pid_file.token.is_none() {
            return ServerStatus { status: ServerStatusType::Starting, pid: Option::Some(pid_file.pid), port: pid_file.port, start_time: Option::Some(pid_file.start_time) }
        }
        self.set_access(pid_file.port.unwrap(), pid_file.token.unwrap());
        match self.req(Method::GET, "/app/health").await {
            Err(_) => ServerStatus { status: ServerStatusType::Starting, pid: Option::Some(pid_file.pid), port: pid_file.port, start_time: Option::Some(pid_file.start_time) },
            Ok(data) => {
                let d: AppStatusRes = data;
                if d.status != "READY" {
                    ServerStatus { status: ServerStatusType::Loading, pid: Option::Some(pid_file.pid), port: pid_file.port, start_time: Option::Some(pid_file.start_time) }
                }else{
                    ServerStatus { status: ServerStatusType::Running, pid: Option::Some(pid_file.pid), port: pid_file.port, start_time: Option::Some(pid_file.start_time) }
                }
            }
        }
    }
    pub async fn waiting_for_start(&mut self) -> Result<(), Box<dyn Error>> {
        let pid_file = self.read_pid_file();
        if pid_file.is_none() {
            self.start_server();
        }
        if self.check_connection().await {
            self.single_signal(10000).await?;
            Result::Ok(())
        }else{
            Result::Err(Box::new(ApplicationError::new("Check connection failed: timed out.")))
        }
    }
    pub async fn maintaining_for_start(&mut self) -> Result<(), Box<dyn Error>> {
        let pid_file = self.read_pid_file();
        if pid_file.is_none() {
            self.start_server();
        }
        if self.check_connection().await {
            self.single_signal(30000).await?;

            let maintain = MaintainComponent { client: Arc::clone(&self.client), access: self.access.clone() };
            
            tokio::spawn(async move {
                loop {
                    async_std::task::sleep(Duration::from_millis(25000)).await;
                    if let Err(e) = maintain.single_signal(30000).await {
                        eprintln!("Maintaining component cannot send single signal: {}", e);
                    }
                }
            });

            Result::Ok(())
        }else{
            Result::Err(Box::new(ApplicationError::new("Check connection failed: timed out.")))
        }
    }
    pub async fn permanent(&self, enable: bool) {
        let body = serde_json::json!({
            "type": "command-line-application",
            "value": enable
        });
        match self.req_with_body(Method::POST, "/app/lifetime/permanent", body).await {
            Ok(d) => {
                let _: Vec<String> = d;
            },
            Err(e) => panic!("Error occurred when set permanent. {}", e)
        }
    }
    pub fn kill(&self) {
        let pid_file = self.read_pid_file();
        if let Some(pid_file) = pid_file {
            let s = System::new_all();
            if let Some(process) = s.process(Pid::from(pid_file.pid as usize)) {
                if process.kill_with(Signal::Kill).is_none() {
                    panic!("This signal isn't supported on this platform.")
                }
            }
        }
    }
    pub fn log(&self) {
        let log_path = self.appdata_path.join("channel").join(&self.channel).join("server.log");
        match fs::File::open(&log_path) {
            Err(e) => if e.kind() == std::io::ErrorKind::NotFound {
                return
            }else{
                panic!("Read log file {} failed. {}", log_path.to_str().unwrap(), e)
            },
            Ok(b) => {
                let buf = BufReader::new(b);
                for line in buf.lines() {
                    println!("{}", line.unwrap());
                }
            }
        }
    }
    async fn single_signal(&self, interval: i64) -> Result<(), Box<dyn Error>> {
        let body = serde_json::json!({
            "interval": interval,
            "standalone": true
        });
        match self.req_without_res(Method::POST, "/app/lifetime/signal", body).await {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(e)
        }
    }
    fn start_server(&self) {
        let bin_path = self.server_path.join("bin/hedge-v3-server");
        let channel_path = self.appdata_path.join("channel").join(&self.channel);
        let args = ["--channel-path", channel_path.to_str().unwrap()];
        
        let log_path = channel_path.join("server.log");
        let out_file = fs::File::create(&log_path).unwrap();
        let err_file = fs::File::create(&log_path).unwrap();
        let stdout = Stdio::from(out_file);
        let stderr = Stdio::from(err_file);

        Command::new(bin_path)
            .args(args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .unwrap();
    }
    async fn check_connection(&mut self) -> bool {
        for _ in 0..100 {
            async_std::task::sleep(Duration::from_millis(100)).await;

            let pid_file = self.read_pid_file();
            if pid_file.is_none() {
                continue
            }

            let pid_file = pid_file.unwrap();
            if pid_file.port.is_none() || pid_file.token.is_none() {
                continue
            }

            self.set_access(pid_file.port.unwrap(), pid_file.token.unwrap());
            match self.req(Method::GET, "/app/health").await {
                Err(_) => {},
                Ok(data) => {
                    let d: AppStatusRes = data;
                    if d.status == "READY" {
                        return true
                    }
                }
            }
        }
        false
    }
    fn set_access(&mut self, port: i32, token: String) {
        self.access.address = Option::Some(format!("http://{}:{}", "localhost", port));
        self.access.token = Option::Some(token);
    }
    pub async fn req<U, T>(&self, method: Method, path: U) -> Result<T, Box<dyn std::error::Error>> where U: IntoUrl, T: serde::de::DeserializeOwned {
        let url = self.access.address.as_ref().map(|address| format!("{}{}", address, path.as_str())).unwrap_or_else(|| path.as_str().to_string());
        let mut b = self.client.request(method, url);
        if let Some(token) = &self.access.token {
            b = b.header("Authorization", format!("Bearer {}", token));
        }
        let res = b.send().await?;
        if res.status().is_success() || res.status().is_redirection() {
            let text = res.text().await?;
            Result::Ok(serde_json::from_str(&text)?)
        }else{
            let text = res.text().await?;
            let err: ErrorResult = serde_json::from_str(&text)?;
            Result::Err(Box::new(ApiResultError::new(&err.code, &err.message)))
        }
    }
    pub async fn req_with_query<U, T>(&self, method: Method, path: U, query: &Vec<(&str, String)>) -> Result<T, Box<dyn std::error::Error>> where U: IntoUrl, T: serde::de::DeserializeOwned {
        let url = self.access.address.as_ref().map(|address| format!("{}{}", address, path.as_str())).unwrap_or_else(|| path.as_str().to_string());
        let mut b = self.client.request(method, url).query(query);
        if let Some(token) = &self.access.token {
            b = b.header("Authorization", format!("Bearer {}", token));
        }
        let res = b.send().await?;
        if res.status().is_success() || res.status().is_redirection() {
            let text = res.text().await?;
            Result::Ok(serde_json::from_str(&text)?)
        }else{
            let text = res.text().await?;
            let err: ErrorResult = serde_json::from_str(&text)?;
            Result::Err(Box::new(ApiResultError::new(&err.code, &err.message)))
        }
    }
    pub async fn req_with_body<U, T>(&self, method: Method, path: U, body: serde_json::Value) -> Result<T, Box<dyn std::error::Error>> where U: IntoUrl, T: serde::de::DeserializeOwned {
        let url = self.access.address.as_ref().map(|address| format!("{}{}", address, path.as_str())).unwrap_or_else(|| path.as_str().to_string());
        let body = serde_json::to_string(&body)?;
        let mut b = self.client.request(method, url).body(body);
        if let Some(token) = &self.access.token {
            b = b.header("Authorization", format!("Bearer {}", token));
        }
        let res = b.send().await?;
        if res.status().is_success() || res.status().is_redirection() {
            let text = res.text().await?;
            Result::Ok(serde_json::from_str(&text)?)
        }else{
            let text = res.text().await?;
            let err: ErrorResult = serde_json::from_str(&text)?;
            Result::Err(Box::new(ApiResultError::new(&err.code, &err.message)))
        }
    }
    pub async fn req_without_res<U>(&self, method: Method, path: U, body: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> where U: IntoUrl {
        let url = self.access.address.as_ref().map(|address| format!("{}{}", address, path.as_str())).unwrap_or_else(|| path.as_str().to_string());
        let body = serde_json::to_string(&body)?;
        let mut b = self.client.request(method, url).body(body);
        if let Some(token) = &self.access.token {
            b = b.header("Authorization", format!("Bearer {}", token));
        }
        let res = b.send().await?;
        if res.status().is_success() || res.status().is_redirection() {
            Result::Ok(())
        }else{
            let text = res.text().await?;
            let err: ErrorResult = serde_json::from_str(&text)?;
            Result::Err(Box::new(ApiResultError::new(&err.code, &err.message)))
        }
    }
    fn read_pid_file(&self) -> Option<PidFile> {
        let pid_file_path = self.appdata_path.join("channel").join(&self.channel).join("PID");
        match fs::read_to_string(&pid_file_path) {
            Err(e) => if e.kind() == std::io::ErrorKind::NotFound {
                Option::None
            }else{
                panic!("Read pid file {} failed. {}", pid_file_path.to_str().unwrap(), e)
            },
            Ok(s) => match serde_json::from_str(&s) {
                Err(e) => panic!("Pid file {} format error. {}", pid_file_path.to_str().unwrap(), e),
                Ok(d) => Option::Some(d)
            }
        }
    }
}

struct MaintainComponent {
    client: Arc<reqwest::Client>,
    access: Access
}

impl MaintainComponent {
    async fn single_signal(&self, interval: i64) -> Result<(), Box<dyn Error>> {
        let body = serde_json::json!({
            "interval": interval,
            "standalone": true
        });
        match self.req_without_res(Method::POST, "/app/lifetime/signal", body).await {
            Ok(_) => Result::Ok(()),
            Err(e) => Result::Err(e)
        }
    }
    pub async fn req_without_res<U>(&self, method: Method, path: U, body: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> where U: IntoUrl {
        let url = self.access.address.as_ref().map(|address| format!("{}{}", address, path.as_str())).unwrap_or_else(|| path.as_str().to_string());
        let body = serde_json::to_string(&body)?;
        let mut b = self.client.request(method, url).body(body);
        if let Some(token) = &self.access.token {
            b = b.header("Authorization", format!("Bearer {}", token));
        }
        b.send().await?;
        Result::Ok(())
    }
}

#[derive(Deserialize)]
struct PidFile {
    pid: i64,
    port: Option<i32>,
    token: Option<String>,
    #[serde(rename = "startTime")]
    start_time: i64
}

pub struct ServerStatus {
    pub status: ServerStatusType,
    pub pid: Option<i64>,
    pub port: Option<i32>,
    pub start_time: Option<i64>
}

#[derive(PartialEq, Eq, Deserialize)]
pub enum ServerStatusType {
    Stop, Starting, Loading, Running
}

#[derive(Deserialize, Serialize)]
pub struct ListResult<T> {
    pub total: i32,
    pub result: Vec<T>
}

#[derive(Deserialize, Serialize)]
pub struct IdWithWarning {
    pub id: i32,
    pub warnings: Vec<ErrorResult>
}

#[derive(Deserialize, Serialize)]
pub struct IdRes {
    pub id: i32
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResult {
    pub code: String,
    pub message: String
}

#[derive(Deserialize)]
struct AppStatusRes {
    status: String
}

impl fmt::Display for ServerStatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stop => write!(f, "Stop"),
            Self::Starting => write!(f, "Starting"),
            Self::Loading => write!(f, "Loading"),
            Self::Running => write!(f, "Running"),
        }
    }
}
