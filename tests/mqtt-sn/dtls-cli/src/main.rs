use anyhow::Result;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::UdpSocket;
use webrtc_dtls::{
    config::Config, conn::DTLSConn, crypto::Certificate, dtls_cli::DtlsCli,
    extension::extension_use_srtp::SrtpProtectionProfile,
};
use webrtc_util::Conn;

#[derive(StructOpt)]
struct Cli {
    //pattern: String,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

// #[derive(Debug)]
// struct DtlsCli1 {
//     flight: u8,
//     target: SocketAddr,
//     times: usize,
// }

#[tokio::main]
async fn main() {
    //let cookie:Vec<u8> = "ff 20 14 ed 1f".split(" ").map(|x| hex_to_u8(x).unwrap()).collect::<Vec<u8>>();
    //println!("{:?}",cookie);
    let args = Cli::from_args();
    let content = std::fs::read_to_string(&args.path).expect("Could not read the file");

    let data: Vec<&str> = content.lines().collect();

    let mut cli = DtlsCli {
        ..Default::default()
    };

    let mut count = 0;
    // let mut start:u8=data[count].parse().unwrap();
    // println!("{:?}", data);
    // println!("{}", data.len());
    // println!("{}", data[count]);
    // let st: u8 = data[count].parse().unwrap();
    // println!("{}", st);

    let target = data[count].parse().unwrap();
    count += 1;

    cli.target = target;

    while count < data.len() {
        let start: u8 = data[count].parse().unwrap();
        if start == 1 {
            cli.flight1 = true;
            cli.times1 = Some(data[count + 1].parse().unwrap());
            count = count + 2;
            //println!("{:?}", cli);
        } else if start == 3 {
            cli.flight3 = true;
            cli.times3 = Some(data[count + 1].parse().unwrap());
            cli.cookie = parse_cookie(data[count + 2]);
            count = count + 3;
            //println!("{:?}", cli);
        } else if start == 5 {
            cli.flight5 = true;
            cli.times5 = Some(data[count + 1].parse().unwrap());
            count = count + 2;
        }
    }
    //println!("{:?}", cli);

    let conn = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
    conn.connect(cli.target).await.unwrap();

    let cfg = Config {
        dtls_cli: cli,
        srtp_protection_profiles: vec![SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80],
        ..Default::default()
    };

    let conn_successful = match create_client(conn, cfg, true).await {
        Ok(_) => true,
        Err(e) => {
            println!("{:?}", e);
            false
        }
    };

    if conn_successful {
        println!("DTLS connection successful");
    }
}

async fn create_client(
    ca: Arc<dyn Conn + Send + Sync>,
    mut cfg: Config,
    generate_certificate: bool,
) -> Result<impl Conn> {
    if generate_certificate {
        let client_cert = Certificate::generate_self_signed(vec!["localhost".to_owned()])?;
        cfg.certificates = vec![client_cert];
    }

    cfg.insecure_skip_verify = true;
    DTLSConn::new(ca, cfg, true, None).await
}

fn hex_to_u8(x: &str) -> Option<u8> {
    return if x.len() > 2 {
        None
    } else {
        let mut num: usize = 0;
        for (i, c) in x.chars().enumerate() {
            let val: usize = match c {
                'a' => 10,
                'b' => 11,
                'c' => 12,
                'd' => 13,
                'e' => 14,
                'f' => 15,
                n @ '0'..='9' => (n as usize) - 48,
                _ => return None,
            };
            num += val * (16 as usize).pow((x.len() - i - 1) as u32);
        }
        Some(num as u8)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_to_u8() {
        assert_eq!(hex_to_u8("14"), Some(20));
        assert_eq!(hex_to_u8("25"), Some(37));
        assert_eq!(hex_to_u8("a1"), Some(161));
        assert_eq!(hex_to_u8("b9"), Some(185));
        assert_eq!(hex_to_u8("ff"), Some(255));
        assert_eq!(hex_to_u8("00"), Some(0));
        assert_eq!(hex_to_u8("ce"), Some(206));
        assert_eq!(hex_to_u8("d9"), Some(217));
        assert_eq!(hex_to_u8("ax"), None);
        assert_eq!(hex_to_u8("g3"), None);
        assert_eq!(hex_to_u8("100"), None);
        assert_eq!(hex_to_u8("59a8"), None);
    }

    #[test]
    fn test_parse_cookie() {
        assert_eq!(
            parse_cookie("aabbccddee1a2b3c2312"),
            vec![170, 187, 204, 221, 238, 26, 43, 60, 35, 18]
        );
        assert_eq!(
            parse_cookie("aabbccddee1a2b3c2311"),
            vec![170, 187, 204, 221, 238, 26, 43, 60, 35, 17]
        );
        assert_eq!(
            parse_cookie("aa bb cc dd ee 1a 2b 3c 23 11"),
            vec![170, 187, 204, 221, 238, 26, 43, 60, 35, 17]
        );
        assert_eq!(parse_cookie("default"), vec![]);
        //assert_eq!(parse_cookie("aa bb cc dd ee 1a 2b 3c 23 11"),vec![170,187,204,221,228,26,43,60,35,17]);
    }

    #[test]
    #[should_panic]
    fn test_parse_cookie_panic() {
        parse_cookie("aa gg ff ee 1a 2b 3c 4d 5e 6f");
    }
}

fn parse_cookie(x: &str) -> Vec<u8> {
    let mut vec_cookie: Vec<u8> = Vec::new();

    if x == "default" {
        return vec_cookie;
    }

    let mut i = 0;
    while i + 1 < x.len() {
        if x.chars().nth(i).unwrap() == ' ' {
            i += 1;
            continue;
        }
        vec_cookie.push(hex_to_u8(&x[i..=i + 1]).unwrap());
        i += 2;
    }
    vec_cookie
}
