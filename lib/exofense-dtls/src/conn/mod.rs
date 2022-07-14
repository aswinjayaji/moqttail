#[cfg(test)]
mod conn_test;

use crate::alert::*;
use crate::application_data::*;
use crate::cipher_suite::*;
use crate::config::*;
use crate::connection_db::ConnectionDb;
use crate::content::*;
use crate::curve::named_curve::NamedCurve;
use crate::error::*;
use crate::extension::extension_use_srtp::*;
use crate::flight::flight0::*;
use crate::flight::flight1::*;
use crate::flight::flight5::*;
use crate::flight::flight6::*;
use crate::flight::*;
use crate::fragment_buffer::*;
use crate::handshake::handshake_cache::*;
use crate::handshake::handshake_header::HandshakeHeader;
use crate::handshake::*;
use crate::handshaker::*;
use crate::record_layer::record_layer_header::*;
use crate::record_layer::*;
use crate::signature_hash_algorithm::parse_signature_schemes;
use crate::state::*;

//use util::{replay_detector::*, Conn};
use crate::replay_detector::*;
use crate::udp_conn::*;

use anyhow::Result;
use async_trait::async_trait;
use log::*;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::marker::{Send, Sync};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;

pub(crate) const INITIAL_TICKER_INTERVAL: Duration = Duration::from_secs(1);
pub(crate) const COOKIE_LENGTH: usize = 20;
pub(crate) const DEFAULT_NAMED_CURVE: NamedCurve = NamedCurve::X25519;
pub(crate) const INBOUND_BUFFER_SIZE: usize = 8192;
// Default replay protection window is specified by RFC 6347 Section 4.1.2.6
pub(crate) const DEFAULT_REPLAY_PROTECTION_WINDOW: usize = 64;

lazy_static! {
    pub static ref INVALID_KEYING_LABELS: HashMap<&'static str, bool> = {
        let mut map = HashMap::new();
        map.insert("client finished", true);
        map.insert("server finished", true);
        map.insert("master secret", true);
        map.insert("key expansion", true);
        map
    };
}

type PacketSendRequest = (Vec<Packet>, Option<mpsc::Sender<Result<()>>>);

struct ConnReaderContext {
    is_client: bool,
    replay_protection_window: usize,
    replay_detector: Vec<Box<dyn ReplayDetector + Send>>,
    decrypted_tx: mpsc::Sender<Result<Vec<u8>>>,
    encrypted_packets: Vec<Vec<u8>>,
    fragment_buffer: FragmentBuffer,
    cache: HandshakeCache,
    cipher_suite: Arc<Mutex<Option<Box<dyn CipherSuite + Send + Sync>>>>,
    remote_epoch: Arc<AtomicU16>,
    handshake_tx: mpsc::Sender<mpsc::Sender<()>>,
    handshake_done_rx: mpsc::Receiver<()>,
    packet_tx: Arc<mpsc::Sender<PacketSendRequest>>,
}

// Conn represents a DTLS connection
pub struct DTLSConn {
    conn: Arc<dyn Conn + Send + Sync>,
    pub(crate) cache: HandshakeCache, // caching of handshake messages for verifyData generation
    decrypted_rx: Mutex<mpsc::Receiver<Result<Vec<u8>>>>, //Update // Decrypted Application Data or error, pull by calling `Read`
    pub(crate) state: State,                              // Internal state

    handshake_completed_successfully: Arc<AtomicBool>,
    connection_closed_by_user: bool,
    // closeLock              sync.Mutex
    closed: AtomicBool, //  *closer.Closer
    //handshakeLoopsFinished sync.WaitGroup

    //readDeadline  :deadline.Deadline,
    //writeDeadline :deadline.Deadline,

    //log logging.LeveledLogger
    /*
    reading               chan struct{}
    handshakeRecv         chan chan struct{}
    cancelHandshaker      func()
    cancelHandshakeReader func()
    */
    pub(crate) current_flight: Box<dyn Flight + Send + Sync>,
    pub(crate) flights: Option<Vec<Packet>>,
    pub(crate) cfg: HandshakeConfig,
    pub(crate) retransmit: bool,
    pub(crate) handshake_rx: mpsc::Receiver<mpsc::Sender<()>>,

    pub(crate) packet_tx: Arc<mpsc::Sender<PacketSendRequest>>,
    pub(crate) handle_queue_tx: mpsc::Sender<mpsc::Sender<()>>,
    pub(crate) handshake_done_tx: Option<mpsc::Sender<()>>,

    reader_close_tx: Mutex<Option<mpsc::Sender<()>>>,
}

#[async_trait]
impl Conn for DTLSConn {
    async fn connect(&self, _addr: SocketAddr) -> Result<()> {
        Err(Error::new("Not applicable".to_owned()).into())
    }
    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        self.read(buf, None).await //Update
    }
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        //Update
        if let Some(raddr) = self.conn.remote_addr().await {
            let n = self.read(buf, None).await?;
            Ok((n, raddr))
        } else {
            Err(Error::new("No remote address is provided by underlying Conn".to_owned()).into())
        }
    }
    async fn send(&self, buf: &[u8]) -> Result<usize> {
        self.write(buf, None).await
    }
    async fn send_to(&self, _buf: &[u8], _target: SocketAddr) -> Result<usize> {
        Err(Error::new("Not applicable".to_owned()).into())
    }
    async fn local_addr(&self) -> Result<SocketAddr> {
        self.conn.local_addr().await
    }
    async fn remote_addr(&self) -> Option<SocketAddr> {
        self.conn.remote_addr().await
    }
    async fn close(&self) -> Result<()> {
        self.close().await
    }
}
impl DTLSConn {
    pub async fn new(
        conn: Arc<dyn Conn + Send + Sync>,
        mut config: Config,
        is_client: bool,
        initial_state: Option<State>,
        db: Option<Arc<ConnectionDb>>,
    ) -> Result<Self> {
        // if(!is_client)
        // {
        // let mut db=ConnectionDb::new("DB".to_string()).unwrap();
        // let key = "0.0.0.0:50000".parse::<SocketAddr>().unwrap();
        // let out=db.read(key);
        // log::debug!("The output state is {:?}",out);
        // }
        validate_config(is_client, &config)?;

        let local_cipher_suites: Vec<CipherSuiteId> = parse_cipher_suites(
            &config.cipher_suites,
            config.psk.is_none(),
            config.psk.is_some(),
        )?
        .iter()
        .map(|cs| cs.id())
        .collect();

        let sigs: Vec<u16> = config.signature_schemes.iter().map(|x| *x as u16).collect();
        let local_signature_schemes = parse_signature_schemes(&sigs, config.insecure_hashes)?;

        let retransmit_interval = if config.flight_interval != Duration::from_secs(0) {
            config.flight_interval
        } else {
            INITIAL_TICKER_INTERVAL
        };

        /*
           loggerFactory := config.LoggerFactory
           if loggerFactory == nil {
               loggerFactory = logging.NewDefaultLoggerFactory()
           }

           logger := loggerFactory.NewLogger("dtls")
        */
        let maximum_transmission_unit = if config.mtu == 0 {
            DEFAULT_MTU
        } else {
            config.mtu
        };

        let replay_protection_window = if config.replay_protection_window == 0 {
            DEFAULT_REPLAY_PROTECTION_WINDOW
        } else {
            config.replay_protection_window
        };

        let mut server_name = config.server_name.clone(); //Update
        if is_client && server_name.is_empty() {
            if let Some(remote_addr) = conn.remote_addr().await {
                server_name = remote_addr.ip().to_string();
            } else {
                log::warn!("conn.remote_addr is empty, please set explicitly server_name in Config! Use default \"localhost\" as server_name now");
                server_name = "localhost".to_owned();
            }
        }

        let cfg = HandshakeConfig {
            local_psk_callback: config.psk.take(),
            local_psk_identity_hint: config.psk_identity_hint.take(),
            local_cipher_suites,
            local_signature_schemes,
            extended_master_secret: config.extended_master_secret,
            local_srtp_protection_profiles: config.srtp_protection_profiles.clone(),
            server_name,
            client_auth: config.client_auth,
            local_certificates: config.certificates.clone(),
            insecure_skip_verify: config.insecure_skip_verify,
            verify_peer_certificate: config.verify_peer_certificate.take(),
            roots_cas: config.roots_cas,
            //Update
            client_cert_verifier: if config.client_auth as u8
                >= ClientAuthType::VerifyClientCertIfGiven as u8
            {
                Some(rustls::AllowAnyAuthenticatedClient::new(config.client_cas))
            } else {
                None
            },
            retransmit_interval,
            //log: logger,
            initial_epoch: 0,
            ..Default::default()
        };

        let mut initial_state_is_none = false;
        let (state, flight, initial_fsm_state) = if let Some(state) = initial_state {

            //log::info!("{}", initial_state_is_none);
            let flight = if is_client {
                Box::new(Flight5 {}) as Box<dyn Flight + Send + Sync>
            } else {
                Box::new(Flight6 {}) as Box<dyn Flight + Send + Sync>
            };

            (state, flight, HandshakeState::Finished)
        } else {
            initial_state_is_none = true;
            let flight = if is_client {
                Box::new(Flight1 {}) as Box<dyn Flight + Send + Sync>
            } else {
                Box::new(Flight0 {}) as Box<dyn Flight + Send + Sync>
            };

            (
                State {
                    is_client,
                    //db:Arc::clone(&config.db),
                    ..Default::default()
                },
                flight,
                HandshakeState::Preparing,
            )
        };

        let (decrypted_tx, decrypted_rx) = mpsc::channel(1);
        let (handshake_tx, handshake_rx) = mpsc::channel(1);
        let (handshake_done_tx, handshake_done_rx) = mpsc::channel(1);
        let (packet_tx, mut packet_rx) = mpsc::channel(1);
        let (handle_queue_tx, mut handle_queue_rx) = mpsc::channel(1);
        let (reader_close_tx, mut reader_close_rx) = mpsc::channel(1);

        let packet_tx = Arc::new(packet_tx);
        let packet_tx2 = Arc::clone(&packet_tx);
        let next_conn_rx = Arc::clone(&conn);
        let next_conn_tx = Arc::clone(&conn);
        let cache = HandshakeCache::new();
        let mut cache1 = cache.clone();
        let cache2 = cache.clone();
        let handshake_completed_successfully = Arc::new(AtomicBool::new(false));
        let handshake_completed_successfully2 = Arc::clone(&handshake_completed_successfully);

        let mut c = DTLSConn {
            conn: Arc::clone(&conn),
            cache,
            decrypted_rx: Mutex::new(decrypted_rx),
            state,
            handshake_completed_successfully,
            connection_closed_by_user: false,
            closed: AtomicBool::new(false),

            current_flight: flight,
            flights: None,
            cfg,
            retransmit: false,
            handshake_rx,
            packet_tx,
            handle_queue_tx,
            handshake_done_tx: Some(handshake_done_tx),
            reader_close_tx: Mutex::new(Some(reader_close_tx)),
        };

        let db2 = match db.as_ref() {
            Some(db) => Some(Arc::clone(&db)),
            None => None,
        };
        let (tx1, mut rx1) = mpsc::channel::<State>(1);
        // let (tx2, mut rx2) = mpsc::channel::<u32>(1);
        // let mut f = 0;

        let cipher_suite1 = Arc::clone(&c.state.cipher_suite);
        let sequence_number = Arc::clone(&c.state.local_sequence_number);
        let remote_addr = conn.remote_addr().await;
        //let mut c_state=State::default();
        let mut block = State::default();
        // let mut c_state = c.state.clone().await;
        // tokio::spawn(async move {
        //     block = rx1.recv().await.unwrap();
        //     println!("{:?}", block.serialize().await.unwrap());
        //     f = 1;
        //     tx2.send(f).await.unwrap();
        // });
        //c_state=block;
        let mut received_block = false;
        tokio::spawn(async move {
            loop {
                let rx = packet_rx.recv().await;
                if let Some(r) = rx {
                    let (pkt, result_tx) = r;

                    let sequence_number = Arc::clone(&sequence_number);
                    let cipher_suite1 = Arc::clone(&cipher_suite1);

                    let result = DTLSConn::handle_outgoing_packets(
                        &next_conn_tx,
                        pkt,
                        &mut cache1,
                        is_client,
                        &sequence_number,
                        &cipher_suite1,
                        maximum_transmission_unit,
                    )
                    .await;
                    if let Some(tx) = result_tx {
                        let _ = tx.send(result).await;
                    }
                    if !received_block {
                        // if !rx1.try_recv().is_err() {
                        //     println!("7");
                        //     let block_res = rx1.recv().await;
                        //     println!("8");
                        //     if block_res.is_some() {
                        //         block = block_res.unwrap();
                        //         log::info!("BLOCK {:?}", block.serialize().await);
                        //         println!("3");
                        //         received_block = true;
                        //     }
                        let block_res = rx1.try_recv();
                        if !block_res.is_err() {
                            block = block_res.unwrap();
                            //log::info!("BLOCK {:?}", block.serialize().await);
                            received_block = true;
                        }
                    }

                    if received_block {
                        let sn = Arc::clone(&sequence_number);
                        let db = match db.as_ref() {
                            Some(db) => Some(Arc::clone(&db)),
                            None => None,
                        };
                        block.local_sequence_number = sequence_number;
                        block.cipher_suite = cipher_suite1;
                        // let mut len: usize = 0;
                        // let mut first_ele: u64 = 0;
                        // //println!("{:?}", lsn);
                        // {
                        //     let lsn = sn.lock().await;
                        //     len = lsn.len();
                        //     if len != 0 {
                        //         first_ele = lsn[0];
                        //     }
                        // }
                        if !is_client && db.is_some() {
                            db.unwrap()
                                .update(remote_addr.unwrap(), &block.serialize().await.unwrap());
                            //log::info!("BLOCKSTATE {:?}", block.serialize().await);
                        }
                    }
                } else {
                    trace!("{}: handle_outgoing_packets exit", srv_cli_str(is_client));
                    break;
                }
            }
        });

        let local_epoch = Arc::clone(&c.state.local_epoch);
        let remote_epoch = Arc::clone(&c.state.remote_epoch);
        let cipher_suite2 = Arc::clone(&c.state.cipher_suite);

        tokio::spawn(async move {
            let mut buf = vec![0u8; INBOUND_BUFFER_SIZE];
            let mut ctx = ConnReaderContext {
                is_client,
                replay_protection_window,
                replay_detector: vec![],
                decrypted_tx,
                encrypted_packets: vec![],
                fragment_buffer: FragmentBuffer::new(),
                cache: cache2,
                cipher_suite: cipher_suite2,
                remote_epoch,
                handshake_tx,
                handshake_done_rx,
                packet_tx: packet_tx2,
            };

            //trace!("before enter read_and_buffer: {}] ", srv_cli_str(is_client));
            loop {
                tokio::select! {
                    _ = reader_close_rx.recv() => {
                        trace!(
                                "{}: read_and_buffer exit",
                                srv_cli_str(ctx.is_client),
                            );
                        break;
                    }
                    result = DTLSConn::read_and_buffer(
                                            &mut ctx,
                                            &next_conn_rx,
                                            &mut handle_queue_rx,
                                            &mut buf,
                                            &local_epoch,
                                            &handshake_completed_successfully2,
                                        ) => {
                        if let Err(err) = result {
                            trace!(
                                "{}: read_and_buffer return err: {}",
                                srv_cli_str(is_client),
                                err
                            );
                            if Error::ErrAlertFatalOrClose.equal(&err) {
                                trace!(
                                    "{}: read_and_buffer exit with {}",
                                    srv_cli_str(ctx.is_client),
                                    err
                                );

                                break;
                            }
                        }
                    }
                }
            }
        });

        // Do handshake
        c.handshake(initial_fsm_state).await?;
        trace!("Handshake Completed");
        // let output = rx2.recv().await.unwrap();
        // println!("Output: {:?}", output);
        // Save connection state to database
        //println!("6 {} {}", !is_client, db2.is_some());
        if !is_client && db2.is_some() {
            let mut c2_state = c.state.clone().await;
            //log::info!("{:?}", c2_state.serialize().await.unwrap());
            tx1.send(c2_state).await;
            let raddr = conn.remote_addr().await.unwrap();
            if initial_state_is_none {
                let _ = db2.unwrap().create(

                    //"127.0.0.0:5000".parse().unwrap(),
                    raddr,
                    &c.state.serialize().await.unwrap(),
                );

                //log::info!("SAVED STATE {:?}", c.state.serialize().await);

                log::info!("Saved connection state from client {}", raddr);
            } else {
                log::info!("Restored connection from client {}", raddr);
            }
        }

        Ok(c)
    }

    // pub async fn save_to_db(&self)
    // {
    // self.db.as_ref().create(self.remote_addr().await.unwrap(),&self.state.serialize().await.unwrap()).unwrap().unwrap();
    // }
    // Read reads data from the connection.
    pub async fn read(&self, p: &mut [u8], duration: Option<Duration>) -> Result<usize> {
        //Update
        if !self.is_handshake_completed_successfully() {
            return Err(Error::ErrHandshakeInProgress.into());
        }

        let rx = {
            let mut decrypted_rx = self.decrypted_rx.lock().await;
            if let Some(d) = duration {
                let timer = tokio::time::sleep(d);
                tokio::pin!(timer);

                tokio::select! {
                    r = decrypted_rx.recv() => r,
                    _ = timer.as_mut() => return Err(Error::ErrDeadlineExceeded.into()),
                }
            } else {
                decrypted_rx.recv().await
            }
        };

        if let Some(out) = rx {
            match out {
                Ok(val) => {
                    let n = val.len(); //Update
                    if p.len() < n {
                        return Err(Error::ErrBufferTooSmall.into());
                    }
                    p[..n].copy_from_slice(&val);
                    Ok(n) //Update
                }
                Err(err) => Err(err),
            }
        } else {
            Err(Error::ErrAlertFatalOrClose.into())
        }
    }

    // Write writes len(p) bytes from p to the DTLS connection
    pub async fn write(&self, p: &[u8], duration: Option<Duration>) -> Result<usize> {
        if self.is_connection_closed() {
            return Err(Error::ErrConnClosed.into());
        }

        if !self.is_handshake_completed_successfully() {
            return Err(Error::ErrHandshakeInProgress.into());
        }

        let pkts = vec![Packet {
            record: RecordLayer::new(
                PROTOCOL_VERSION1_2,
                self.get_local_epoch(),
                Content::ApplicationData(ApplicationData { data: p.to_vec() }),
            ),
            should_encrypt: true,
            reset_local_sequence_number: false,
        }];

        if let Some(d) = duration {
            let timer = tokio::time::sleep(d);
            tokio::pin!(timer);

            tokio::select! {
                result = self.write_packets(pkts) => {
                    if let Err(err) = result {
                        return Err(err);
                    }
                }
                _ = timer.as_mut() => return Err(Error::ErrDeadlineExceeded.into()),
            }
        } else {
            self.write_packets(pkts).await?;
        }

        Ok(p.len())
    }

    // Close closes the connection.
    pub async fn close(&self) -> Result<()> {
        if !self.closed.load(Ordering::SeqCst) {
            self.closed.store(true, Ordering::SeqCst);

            // Discard error from notify() to return non-error on the first user call of Close()
            // even if the underlying connection is already closed.
            self.notify(AlertLevel::Warning, AlertDescription::CloseNotify)
                .await?;

            {
                let mut reader_close_tx = self.reader_close_tx.lock().await;
                reader_close_tx.take();
            }
            self.conn.close().await?;
        }

        Ok(())
    }

    /// connection_state returns basic DTLS details about the connection.
    /// Note that this replaced the `Export` function of v1.
    pub async fn connection_state(&self) -> State {
        self.state.clone().await
    }

    /// selected_srtpprotection_profile returns the selected SRTPProtectionProfile
    pub fn selected_srtpprotection_profile(&self) -> SrtpProtectionProfile {
        self.state.srtp_protection_profile
    }

    pub(crate) async fn notify(&self, level: AlertLevel, desc: AlertDescription) -> Result<()> {
        self.write_packets(vec![Packet {
            record: RecordLayer::new(
                PROTOCOL_VERSION1_2,
                self.get_local_epoch(),
                Content::Alert(Alert {
                    alert_level: level,
                    alert_description: desc,
                }),
            ),
            should_encrypt: self.is_handshake_completed_successfully(),
            reset_local_sequence_number: false,
        }])
        .await
    }

    pub(crate) async fn write_packets(&self, pkts: Vec<Packet>) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        self.packet_tx.send((pkts, Some(tx))).await?;

        if let Some(result) = rx.recv().await {
            result
        } else {
            Ok(())
        }
    }

    async fn handle_outgoing_packets(
        next_conn: &Arc<dyn Conn + Send + Sync>,
        mut pkts: Vec<Packet>,
        cache: &mut HandshakeCache,
        is_client: bool,
        local_sequence_number: &Arc<Mutex<Vec<u64>>>,
        cipher_suite: &Arc<Mutex<Option<Box<dyn CipherSuite + Send + Sync>>>>,
        maximum_transmission_unit: usize,
    ) -> Result<()> {
        let mut raw_packets = vec![];
        for p in &mut pkts {
            if let Content::Handshake(h) = &p.record.content {
                let mut handshake_raw = vec![];
                {
                    let mut writer = BufWriter::<&mut Vec<u8>>::new(handshake_raw.as_mut());
                    p.record.marshal(&mut writer)?;
                }
                trace!(
                    "Send [handshake:{}] -> {} (epoch: {}, seq: {})",
                    srv_cli_str(is_client),
                    h.handshake_header.handshake_type.to_string(),
                    p.record.record_layer_header.epoch,
                    h.handshake_header.message_sequence
                );
                cache
                    .push(
                        handshake_raw[RECORD_LAYER_HEADER_SIZE..].to_vec(),
                        p.record.record_layer_header.epoch,
                        h.handshake_header.message_sequence,
                        h.handshake_header.handshake_type,
                        is_client,
                    )
                    .await;

                let raw_handshake_packets = DTLSConn::process_handshake_packet(
                    local_sequence_number, //Update
                    cipher_suite,
                    maximum_transmission_unit,
                    p,
                    h,
                )
                .await?;
                raw_packets.extend_from_slice(&raw_handshake_packets);
            } else {
                /*if let Content::Alert(a) = &p.record.content {
                    if a.alert_description == AlertDescription::CloseNotify {
                        closed = true;
                    }
                }*/

                let raw_packet =
                    DTLSConn::process_packet(local_sequence_number, cipher_suite, p).await?;
                raw_packets.push(raw_packet);
            }
        }

        if !raw_packets.is_empty() {
            let compacted_raw_packets =
                compact_raw_packets(&raw_packets, maximum_transmission_unit);

            for compacted_raw_packets in &compacted_raw_packets {
                next_conn.send(compacted_raw_packets).await?;
            }
        }

        Ok(())
    }

    async fn process_packet(
        local_sequence_number: &Arc<Mutex<Vec<u64>>>,
        cipher_suite: &Arc<Mutex<Option<Box<dyn CipherSuite + Send + Sync>>>>,
        p: &mut Packet,
    ) -> Result<Vec<u8>> {
        let epoch = p.record.record_layer_header.epoch as usize;
        let seq = {
            let mut lsn = local_sequence_number.lock().await;
            while lsn.len() <= epoch {
                lsn.push(0);
            }

            lsn[epoch] += 1;
            lsn[epoch] - 1
        };
        //trace!("{}: seq = {}", srv_cli_str(is_client), seq);

        if seq > MAX_SEQUENCE_NUMBER {
            // RFC 6347 Section 4.1.0
            // The implementation must either abandon an association or rehandshake
            // prior to allowing the sequence number to wrap.
            return Err(Error::ErrSequenceNumberOverflow.into());
        }
        p.record.record_layer_header.sequence_number = seq;

        let mut raw_packet = vec![];
        {
            let mut writer = BufWriter::<&mut Vec<u8>>::new(raw_packet.as_mut());
            p.record.marshal(&mut writer)?;
        }

        if p.should_encrypt {
            let cipher_suite = cipher_suite.lock().await;
            if let Some(cipher_suite) = &*cipher_suite {
                raw_packet = cipher_suite.encrypt(&p.record.record_layer_header, &raw_packet)?;
            }
        }

        Ok(raw_packet)
    }

    async fn process_handshake_packet(
        local_sequence_number: &Arc<Mutex<Vec<u64>>>,
        cipher_suite: &Arc<Mutex<Option<Box<dyn CipherSuite + Send + Sync>>>>,
        maximum_transmission_unit: usize,
        p: &Packet,
        h: &Handshake,
    ) -> Result<Vec<Vec<u8>>> {
        let mut raw_packets = vec![];

        let handshake_fragments = DTLSConn::fragment_handshake(maximum_transmission_unit, h)?;

        let epoch = p.record.record_layer_header.epoch as usize;

        let mut lsn = local_sequence_number.lock().await;
        while lsn.len() <= epoch {
            lsn.push(0);
        }

        for handshake_fragment in &handshake_fragments {
            let seq = {
                lsn[epoch] += 1;
                lsn[epoch] - 1
            };
            //trace!("seq = {}", seq);
            if seq > MAX_SEQUENCE_NUMBER {
                return Err(Error::ErrSequenceNumberOverflow.into());
            }

            let record_layer_header = RecordLayerHeader {
                protocol_version: p.record.record_layer_header.protocol_version,
                content_type: p.record.record_layer_header.content_type,
                content_len: handshake_fragment.len() as u16,
                epoch: p.record.record_layer_header.epoch,
                sequence_number: seq,
            };

            let mut record_layer_header_bytes = vec![];
            {
                let mut writer = BufWriter::<&mut Vec<u8>>::new(record_layer_header_bytes.as_mut());
                record_layer_header.marshal(&mut writer)?;
            }

            //p.record.record_layer_header = record_layer_header;

            let mut raw_packet = vec![];
            raw_packet.extend_from_slice(&record_layer_header_bytes);
            raw_packet.extend_from_slice(handshake_fragment); //Update
            if p.should_encrypt {
                let cipher_suite = cipher_suite.lock().await;
                if let Some(cipher_suite) = &*cipher_suite {
                    raw_packet = cipher_suite.encrypt(&record_layer_header, &raw_packet)?;
                }
            }

            raw_packets.push(raw_packet);
        }

        Ok(raw_packets)
    }

    fn fragment_handshake(maximum_transmission_unit: usize, h: &Handshake) -> Result<Vec<Vec<u8>>> {
        let mut content = vec![];
        {
            let mut writer = BufWriter::<&mut Vec<u8>>::new(content.as_mut());
            h.handshake_message.marshal(&mut writer)?;
        }

        let mut fragmented_handshakes = vec![];

        let mut content_fragments = split_bytes(&content, maximum_transmission_unit);
        if content_fragments.is_empty() {
            content_fragments = vec![vec![]];
        }

        let mut offset = 0;
        for content_fragment in &content_fragments {
            let content_fragment_len = content_fragment.len();

            let handshake_header_fragment = HandshakeHeader {
                handshake_type: h.handshake_header.handshake_type,
                length: h.handshake_header.length,
                message_sequence: h.handshake_header.message_sequence,
                fragment_offset: offset as u32,
                fragment_length: content_fragment_len as u32,
            };

            offset += content_fragment_len;

            let mut handshake_header_fragment_raw = vec![];
            {
                let mut writer =
                    BufWriter::<&mut Vec<u8>>::new(handshake_header_fragment_raw.as_mut());
                handshake_header_fragment.marshal(&mut writer)?;
            }

            let mut fragmented_handshake = vec![];
            fragmented_handshake.extend_from_slice(&handshake_header_fragment_raw);
            fragmented_handshake.extend_from_slice(content_fragment); //Update

            fragmented_handshakes.push(fragmented_handshake);
        }

        Ok(fragmented_handshakes)
    }

    pub(crate) fn set_handshake_completed_successfully(&mut self) {
        self.handshake_completed_successfully
            .store(true, Ordering::SeqCst); //Update
    }

    pub(crate) fn is_handshake_completed_successfully(&self) -> bool {
        self.handshake_completed_successfully.load(Ordering::SeqCst)
    }

    async fn read_and_buffer(
        ctx: &mut ConnReaderContext,
        next_conn: &Arc<dyn Conn + Send + Sync>,
        handle_queue_rx: &mut mpsc::Receiver<mpsc::Sender<()>>,
        buf: &mut [u8],
        local_epoch: &Arc<AtomicU16>,
        handshake_completed_successfully: &Arc<AtomicBool>,
    ) -> Result<()> {
        let n = next_conn.recv(buf).await?; //Update
        let pkts = unpack_datagram(&buf[..n])?;
        let mut has_handshake = false;
        for pkt in pkts {
            let (hs, alert, mut err) = DTLSConn::handle_incoming_packet(ctx, pkt, true).await; //Update
            if let Some(alert) = alert {
                let alert_err = ctx
                    .packet_tx
                    .send((
                        vec![Packet {
                            record: RecordLayer::new(
                                PROTOCOL_VERSION1_2,
                                local_epoch.load(Ordering::SeqCst),
                                Content::Alert(Alert {
                                    alert_level: alert.alert_level,
                                    alert_description: alert.alert_description,
                                }),
                            ),
                            should_encrypt: handshake_completed_successfully.load(Ordering::SeqCst),
                            reset_local_sequence_number: false,
                        }],
                        None,
                    ))
                    .await;

                if let Err(alert_err) = alert_err {
                    if err.is_none() {
                        err = Some(Error::new(alert_err.to_string()).into());
                    }
                }

                if alert.alert_level == AlertLevel::Fatal
                    || alert.alert_description == AlertDescription::CloseNotify
                {
                    return Err(Error::ErrAlertFatalOrClose.into());
                }
            }

            if let Some(err) = err {
                return Err(err);
            }

            if hs {
                has_handshake = true
            }
        }

        if has_handshake {
            let (done_tx, mut done_rx) = mpsc::channel(1);

            tokio::select! {
                _ = ctx.handshake_tx.send(done_tx) => {
                    let mut wait_done_rx = true;
                    while wait_done_rx{
                        tokio::select!{
                            _ = done_rx.recv() => {
                                // If the other party may retransmit the flight,
                                // we should respond even if it not a new message.
                                wait_done_rx = false;
                            }
                            done = handle_queue_rx.recv() => {
                                //trace!("recv handle_queue: {} ", srv_cli_str(ctx.is_client));

                                let pkts = ctx.encrypted_packets.drain(..).collect();
                                DTLSConn::handle_queued_packets(ctx, local_epoch, handshake_completed_successfully, pkts).await?; //Update

                                drop(done);
                            }
                        }
                    }
                }
                _ = ctx.handshake_done_rx.recv() => {}
            }
        }

        Ok(())
    }

    async fn handle_queued_packets(
        ctx: &mut ConnReaderContext,
        local_epoch: &Arc<AtomicU16>,
        handshake_completed_successfully: &Arc<AtomicBool>,
        pkts: Vec<Vec<u8>>,
        //raddr: SocketAddr,
    ) -> Result<()> {
        for p in pkts {
            let (_, alert, mut err) = DTLSConn::handle_incoming_packet(ctx, p, false).await; //Update // don't re-enqueue
            if let Some(alert) = alert {
                let alert_err = ctx
                    .packet_tx
                    .send((
                        vec![Packet {
                            record: RecordLayer::new(
                                PROTOCOL_VERSION1_2,
                                local_epoch.load(Ordering::SeqCst),
                                Content::Alert(Alert {
                                    alert_level: alert.alert_level,
                                    alert_description: alert.alert_description,
                                }),
                            ),
                            should_encrypt: handshake_completed_successfully.load(Ordering::SeqCst),
                            reset_local_sequence_number: false,
                        }],
                        None,
                    ))
                    .await;

                if let Err(alert_err) = alert_err {
                    if err.is_none() {
                        err = Some(Error::new(alert_err.to_string()).into());
                    }
                }
                if alert.alert_level == AlertLevel::Fatal
                    || alert.alert_description == AlertDescription::CloseNotify
                {
                    return Err(Error::ErrAlertFatalOrClose.into());
                }
            }

            if let Some(err) = err {
                return Err(err);
            }
        }

        Ok(())
    }

    async fn handle_incoming_packet(
        ctx: &mut ConnReaderContext,
        mut pkt: Vec<u8>,
        //raddr: SocketAddr,
        enqueue: bool,
    ) -> (bool, Option<Alert>, Option<anyhow::Error>) {
        let mut reader = BufReader::new(pkt.as_slice());
        let h = match RecordLayerHeader::unmarshal(&mut reader) {
            Ok(h) => h,
            Err(err) => {
                // Decode error must be silently discarded
                // [RFC6347 Section-4.1.2.7]
                debug!(
                    "{}: discarded broken packet: {}",
                    srv_cli_str(ctx.is_client),
                    err
                );
                return (false, None, None);
            }
        };

        // Validate epoch
        let epoch = ctx.remote_epoch.load(Ordering::SeqCst);
        if h.epoch > epoch {
            if h.epoch > epoch + 1 {
                debug!(
                    "{}: discarded future packet (epoch: {}, seq: {})",
                    srv_cli_str(ctx.is_client),
                    h.epoch,
                    h.sequence_number,
                );
                return (false, None, None);
            }
            if enqueue {
                debug!(
                    "{}: received packet of next epoch, queuing packet",
                    srv_cli_str(ctx.is_client)
                );
                ctx.encrypted_packets.push(pkt);
            }
            return (false, None, None);
        }

        // Anti-replay protection
        while ctx.replay_detector.len() <= h.epoch as usize {
            ctx.replay_detector
                .push(Box::new(SlidingWindowDetector::new(
                    ctx.replay_protection_window,
                    MAX_SEQUENCE_NUMBER,
                )));
        }

        let ok = ctx.replay_detector[h.epoch as usize].check(h.sequence_number);
        if !ok {
            debug!(
                "{}: discarded duplicated packet (epoch: {}, seq: {})",
                srv_cli_str(ctx.is_client),
                h.epoch,
                h.sequence_number,
            );
            return (false, None, None);
        }

        // Decrypt
        if h.epoch != 0 {
            let invalid_cipher_suite = {
                let cipher_suite = ctx.cipher_suite.lock().await;
                if cipher_suite.is_none() {
                    true
                } else if let Some(cipher_suite) = &*cipher_suite {
                    !cipher_suite.is_initialized()
                } else {
                    false
                }
            };
            if invalid_cipher_suite {
                if enqueue {
                    debug!(
                        "{}: handshake not finished, queuing packet",
                        srv_cli_str(ctx.is_client)
                    );
                    ctx.encrypted_packets.push(pkt);
                }
                return (false, None, None);
            }

            let cipher_suite = ctx.cipher_suite.lock().await;
            if let Some(cipher_suite) = &*cipher_suite {
                pkt = match cipher_suite.decrypt(&pkt) {
                    Ok(pkt) => pkt,
                    Err(err) => {
                        debug!("{}: decrypt failed: {}", srv_cli_str(ctx.is_client), err);
                        return (false, None, None);
                    }
                };
            }
        }

        let is_handshake = match ctx.fragment_buffer.push(&pkt) {
            Ok(is_handshake) => is_handshake,
            Err(err) => {
                // Decode error must be silently discarded
                // [RFC6347 Section-4.1.2.7]
                debug!("{}: defragment failed: {}", srv_cli_str(ctx.is_client), err);
                return (false, None, None);
            }
        };
        if is_handshake {
            ctx.replay_detector[h.epoch as usize].accept();
            while let Ok((out, epoch)) = ctx.fragment_buffer.pop() {
                //log::debug!("Extension Debug: out.len()={}", out.len());
                let mut reader = BufReader::new(out.as_slice());
                let raw_handshake = match Handshake::unmarshal(&mut reader) {
                    Ok(rh) => {
                        trace!(
                            "Recv [handshake:{}] -> {} (epoch: {}, seq: {})",
                            srv_cli_str(ctx.is_client),
                            rh.handshake_header.handshake_type.to_string(),
                            h.epoch,
                            rh.handshake_header.message_sequence
                        );
                        rh
                    }
                    Err(err) => {
                        debug!(
                            "{}: handshake parse failed: {}",
                            srv_cli_str(ctx.is_client),
                            err
                        );
                        continue;
                    }
                };

                ctx.cache
                    .push(
                        out,
                        epoch,
                        raw_handshake.handshake_header.message_sequence,
                        raw_handshake.handshake_header.handshake_type,
                        !ctx.is_client,
                    )
                    .await;
            }

            return (true, None, None);
        }

        let mut reader = BufReader::new(pkt.as_slice());
        let r = match RecordLayer::unmarshal(&mut reader) {
            Ok(r) => r,
            Err(err) => {
                return (
                    false,
                    Some(Alert {
                        alert_level: AlertLevel::Fatal,
                        alert_description: AlertDescription::DecodeError,
                    }),
                    Some(err),
                );
            }
        };

        match r.content {
            Content::Alert(mut a) => {
                trace!("{}: <- {}", srv_cli_str(ctx.is_client), a.to_string());
                if a.alert_description == AlertDescription::CloseNotify {
                    // Respond with a close_notify [RFC5246 Section 7.2.1]
                    a = Alert {
                        alert_level: AlertLevel::Warning,
                        alert_description: AlertDescription::CloseNotify,
                    };
                }
                ctx.replay_detector[h.epoch as usize].accept();
                return (
                    false,
                    Some(a),
                    Some(Error::new(format!("Error of Alert {}", a.to_string())).into()),
                );
            }
            Content::ChangeCipherSpec(_) => {
                let invalid_cipher_suite = {
                    let cipher_suite = ctx.cipher_suite.lock().await;
                    if cipher_suite.is_none() {
                        true
                    } else if let Some(cipher_suite) = &*cipher_suite {
                        !cipher_suite.is_initialized()
                    } else {
                        false
                    }
                };

                if invalid_cipher_suite {
                    if enqueue {
                        debug!(
                            "{}: CipherSuite not initialized, queuing packet",
                            srv_cli_str(ctx.is_client)
                        );
                        ctx.encrypted_packets.push(pkt);
                    }
                    return (false, None, None);
                }

                let new_remote_epoch = h.epoch + 1;
                trace!(
                    "{}: <- ChangeCipherSpec (epoch: {})",
                    srv_cli_str(ctx.is_client),
                    new_remote_epoch
                );

                if epoch + 1 == new_remote_epoch {
                    ctx.remote_epoch.store(new_remote_epoch, Ordering::SeqCst);
                    ctx.replay_detector[h.epoch as usize].accept();
                }
            }
            Content::ApplicationData(a) => {
                if h.epoch == 0 {
                    return (
                        false,
                        Some(Alert {
                            alert_level: AlertLevel::Fatal,
                            alert_description: AlertDescription::UnexpectedMessage,
                        }),
                        Some(Error::ErrApplicationDataEpochZero.into()),
                    );
                }

                ctx.replay_detector[h.epoch as usize].accept();

                let _ = ctx.decrypted_tx.send(Ok(a.data)).await; //Update
                                                                 //TODO
                                                                 /*select {
                                                                     case self.decrypted < - content.data:
                                                                     case < -c.closed.Done():
                                                                 }*/
            }
            _ => {
                return (
                    false,
                    Some(Alert {
                        alert_level: AlertLevel::Fatal,
                        alert_description: AlertDescription::UnexpectedMessage,
                    }),
                    Some(Error::ErrUnhandledContextType.into()),
                );
            }
        };

        (false, None, None)
    }

    fn is_connection_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub(crate) fn set_local_epoch(&mut self, epoch: u16) {
        self.state.local_epoch.store(epoch, Ordering::SeqCst);
    }

    pub(crate) fn get_local_epoch(&self) -> u16 {
        self.state.local_epoch.load(Ordering::SeqCst)
    }
}

fn compact_raw_packets(raw_packets: &[Vec<u8>], maximum_transmission_unit: usize) -> Vec<Vec<u8>> {
    let mut combined_raw_packets = vec![];
    let mut current_combined_raw_packet = vec![];

    for raw_packet in raw_packets {
        if !current_combined_raw_packet.is_empty()
            && current_combined_raw_packet.len() + raw_packet.len() >= maximum_transmission_unit
        {
            combined_raw_packets.push(current_combined_raw_packet);
            current_combined_raw_packet = vec![];
        }
        current_combined_raw_packet.extend_from_slice(raw_packet);
    }

    combined_raw_packets.push(current_combined_raw_packet);

    combined_raw_packets
}

fn split_bytes(bytes: &[u8], split_len: usize) -> Vec<Vec<u8>> {
    let mut splits = vec![];
    let num_bytes = bytes.len();
    for i in (0..num_bytes).step_by(split_len) {
        let mut j = i + split_len;
        if j > num_bytes {
            j = num_bytes;
        }

        splits.push(bytes[i..j].to_vec());
    }

    splits
}
