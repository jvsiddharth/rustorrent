use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::process;
use std::time::Instant;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rand::seq::SliceRandom;

struct PieceManager {
    total_pieces: u32,
    downloaded_pieces: HashSet<u32>,
    requested_pieces: HashSet<u32>,
    pieces_data: Vec<Vec<Vec<u8>>>, // New field to store piece data
}

use rand::Rng;

fn generate_peer_id() -> String {
    const CLIENT_CODE: &str = "TR";
    const VERSION_NUMBER: &str = "2820";
    const RANDOM_STRING_LENGTH: usize = 12;

    let mut rng = rand::thread_rng();
    let random_string: String = (0..RANDOM_STRING_LENGTH)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char) // Convert each u8 to a char
        .collect();

    format!("-{}{}-{}", CLIENT_CODE, VERSION_NUMBER, random_string)
}

fn construct_handshake() -> Vec<u8> {
    const PROTOCOL_STRING: &str = "BitTorrent protocol";
    const PROTOCOL_STRING_LEN: u8 = PROTOCOL_STRING.len() as u8;

    let info_hash: [u8; 20] = [0xAB; 20]; // Replace with actual info hash
    let peer_id: String = generate_peer_id();

    let mut handshake: Vec<u8> = Vec::new();
    handshake.push(PROTOCOL_STRING_LEN);
    handshake.extend(PROTOCOL_STRING.as_bytes());
    handshake.extend(&[0; 8]); // Reserved bytes
    handshake.extend(&info_hash);
    handshake.extend(peer_id.as_bytes());

    handshake
}

fn parse_handshake(handshake: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if handshake.len() < 68 {
        return None;
    }

    let protocol_len = handshake[0] as usize;

    if &handshake[1..=protocol_len] != b"BitTorrent protocol" {
        return None;
    }

    let info_hash_start = protocol_len + 9;
    let info_hash_end = info_hash_start + 20;

    let peer_id_start = info_hash_end;
    let peer_id_end = peer_id_start + 20;

    if handshake.len() < peer_id_end {
        return None;
    }

    let info_hash = handshake[info_hash_start..info_hash_end].to_vec();
    let peer_id = handshake[peer_id_start..peer_id_end].to_vec();

    Some((info_hash, peer_id))
}

fn assemble_file(piece_manager: &PieceManager) -> Result<(), Box<dyn std::error::Error>> {
    // Assuming PieceManager has a pieces_data field storing downloaded pieces
    // Assemble the file from downloaded pieces

    // Ensure all pieces are downloaded before assembling the file
    if !piece_manager.all_pieces_downloaded() {
        return Err("Not all pieces downloaded yet.".into());
    }

    // Assuming the file content is stored in a Vec<u8>
    let mut file_content: Vec<u8> = Vec::new();

    // Concatenate pieces and blocks to assemble the file content
    for piece in &piece_manager.pieces_data {
        for block in piece {
            file_content.extend_from_slice(block);
        }
    }

    // Perform operations with the assembled file content (e.g., write to disk)
    // For example:
    // std::fs::write("assembled_file.txt", &file_content)?;

    Ok(())
}
async fn process_message(
    message: &[u8],
    piece_manager: &mut PieceManager,
    _socket: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    // Existing message handling code
    match message[0] {
        // Existing cases...
        7 => {
            if message.len() < 9 {
                return Ok(());
            }
            let piece_index = u32::from_be_bytes([message[1], message[2], message[3], message[4]]);
            let block_offset = u32::from_be_bytes([message[5], message[6], message[7], message[8]]);
            let block_data = &message[9..];

            println!(
                "Received piece message for piece index: {}, offset: {}, data length: {}",
                piece_index,
                block_offset,
                block_data.len()
            );

            // Store the received piece data
            piece_manager.store_piece(piece_index, block_offset, block_data);

            // Check if all pieces are downloaded
            if piece_manager.all_pieces_downloaded() {
                println!("All pieces downloaded. Assembling file...");
                if let Err(err) = assemble_file(piece_manager) {
                    eprintln!("Error assembling file: {}", err);
                } else {
                    println!("File assembled successfully.");
                }
            }
        }
        // Existing cases...
        _ => println!("Received unknown message type"),
    }

    Ok(())
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "BitTorrent Client", about = "A BitTorrent client")]
struct Cli {
    #[structopt(help = "Magnet link")]
    magnet_link: String,
}

use indicatif::{ProgressBar, ProgressStyle};
use url::Url;

fn fetch_tracker_url(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let tracker_url = if input.starts_with("magnet:") {
        // Parse Magnet link
        let parsed_url = Url::parse(input)?;

        if let Some(host) = parsed_url.host_str() {
            format!("https://{}/tracker", host)
        } else {
            return Err(format!("Invalid magnet link: no host found in '{}'", input).into());
        }
    } else if input.starts_with("urn:btih:") {
        // Handle torrent info hash (replace with your logic)
        // Example: Generate tracker URL from info hash
        // let info_hash = parse_info_hash(input)?;
        // let tracker_url = generate_tracker_url_from_info_hash(info_hash);
        unimplemented!("Handling torrent info hash is not implemented yet")
    } else if input.ends_with(".torrent") {
        // Handle loading .torrent file (replace with your logic)
        // Example: Load .torrent file and extract tracker URL
        // let tracker_url = extract_tracker_url_from_torrent_file(input)?;
        unimplemented!("Loading .torrent file and extracting tracker URL is not implemented yet")
    } else {
        return Err("Unsupported input format".into());
    };

    Ok(tracker_url)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Select the type of torrent info:");
    println!("1. Torrent File");
    println!("2. Magnet URL");
    println!("3. Info Hash");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: u32 = input.trim().parse().expect("Please enter a valid number");
    let mut tracker_url = String::new();

    match choice {
        1 => {
            println!("Enter the path to the torrent file:");
            let mut torrent_file_path = String::new();
            io::stdin().read_line(&mut torrent_file_path)?;
            tracker_url = process_torrent_file(&torrent_file_path.trim()).await?;
        }
        2 => {
            println!("Enter the magnet URL:");
            let mut magnet_url = String::new();
            io::stdin().read_line(&mut magnet_url)?;
            tracker_url = fetch_tracker_url_from_magnet(&magnet_url.trim())?;
        }
        3 => {
            println!("Enter the info hash:");
            let mut info_hash = String::new();
            io::stdin().read_line(&mut info_hash)?;
            tracker_url = process_info_hash(&info_hash.trim()).await?;
        }
        _ => {
            println!("Invalid choice!");
            return Ok(());
        }
    }

    let peers = fetch_peers(&tracker_url).await?;
    for peer_addr in peers {
        handle_peer_download(&peer_addr).await?;
    }

    let peers = fetch_peers(&tracker_url).await?;
    for peer_addr in peers {
        handle_peer_download(&peer_addr).await?;
    }


    let cli_args = Cli::from_args();
    let magnet_link = cli_args.magnet_link;
    let env_args: Vec<String> = env::args().collect();
    let tracker_url = fetch_tracker_url(&magnet_link)?;
    let pb = ProgressBar::new(100);
    let style = match ProgressStyle::default_spinner().tick_chars("█▉▊▋▌▍▎▏ ").template("{spinner:.green} {msg}") {
        Ok(s) => s,
        Err(_) => ProgressStyle::default_spinner(), // Use default style if there's an error
    };

    pb.set_style(style);
    
    // Start the progress bar
    pb.enable_steady_tick(Duration::from_millis(100)); // Use Duration here

    // Simulate progress
    for _ in 0..100 {
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    pb.finish_with_message("Download complete");

    let peers: Vec<String> = fetch_peers(&tracker_url).await?; // Intentionally unused, prefixed with an underscore

    // Start downloading files
    for peer_addr in peers {
        let mut socket = TcpStream::connect(peer_addr).await?;
        let handshake_msg = construct_handshake();
        socket.write_all(&handshake_msg).await?;
        // ... Handle peer connection and messages as per the BitTorrent protocol
        // You might want to implement downloading logic here
    }

    // Call fetch_peers with both arguments
    
    // Start the progress bar
    pb.enable_steady_tick(Duration::from_millis(100)); // Use Duration here
    
    // Simulate progress
    for _ in 0..100 {
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    pb.finish_with_message("Download complete");

    let _peers = fetch_peers(&tracker_url).await?; // Use `await` here
    
    pb.finish_with_message("Download complete");

    if env_args.len() != 2 {
        println!("Usage: bittorrent-client <port>");
        process::exit(1);
    }

    let port = match env_args[1].parse::<u16>() {
        Ok(p) => p,
        Err(_) => {
            println!("Invalid port number");
            process::exit(1);
        }
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Server running on 127.0.0.1:{}", port);

    let piece_manager = Arc::new(Mutex::new(PieceManager::new(100))); // Example total pieces
    let downloaded_count = Arc::new(Mutex::new(0));

    let (tx, rx) = mpsc::channel::<u32>();
    let _piece_manager_clone = Arc::clone(&piece_manager);
    let downloaded_count_clone = Arc::clone(&downloaded_count);

    let peers = fetch_peers("MAGNET_LINK").await?; // Replace "MAGNET_LINK" with an actual magnet link

    for peer_addr in peers {
        let mut socket = TcpStream::connect(peer_addr).await?;
        // Perform handshake and communication with the peer (as in the existing handle_connection function)
        // For example:
        let handshake_msg = construct_handshake();
        socket.write_all(&handshake_msg).await?;
        // ... Handle peer connection and messages as per the BitTorrent protocol
    }

    thread::spawn(move || {
        let start = Instant::now();
        let mut prev_downloaded = 0;
        let mut downloaded = 0;
        let mut file_progress: Vec<String> = Vec::new();

        loop {
            match rx.try_recv() {
                Ok(piece) => {
                    downloaded += 1;
                    *downloaded_count_clone.lock().unwrap() = downloaded;

                    if downloaded == 1 {
                        println!("Download in progress...");
                    }

                    if !file_progress.contains(&piece.to_string()) {
                        file_progress.push(piece.to_string());
                        println!("Downloading piece: {}", piece);
                    }

                    if downloaded == 100 {
                        println!("All pieces downloaded!");
                        break;
                    }
                }
                Err(_) => {}
            }

            if downloaded != prev_downloaded {
                let elapsed = start.elapsed().as_secs();
                let speed = downloaded as f64 / elapsed as f64;
                let remaining_pieces = 100 - downloaded;
                let eta = remaining_pieces as f64 / speed;

                println!(
                    "Download Speed: {:.2} pieces/sec | Remaining Pieces: {} | ETA: {:.2} sec",
                    speed, remaining_pieces, eta
                );

                prev_downloaded = downloaded;
            }

            thread::sleep(Duration::from_secs(1));
        }
    });

    while let Ok((socket, _)) = listener.accept().await {
        let _tx_clone = tx.clone();
        let _piece_manager_clone = Arc::clone(&piece_manager);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}


use reqwest;

async fn process_torrent_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Process the torrent file and extract tracker URL
    // Example logic: Read the torrent file and parse tracker URL
    // Placeholder return for demonstration purposes
    Ok("tracker_url_from_torrent_file".to_string())
}

async fn process_info_hash(info_hash: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Process the info hash and generate the tracker URL
    // Example logic: Generate tracker URL based on the info hash
    // Placeholder return for demonstration purposes
    Ok("tracker_url_from_info_hash".to_string())
}

// Other async functions remain unchanged...

async fn fetch_tracker_url_from_magnet(magnet_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Implement fetching tracker URL from the magnet URL
    Ok("tracker_url_from_magnet".to_string())
}


async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let handshake_msg = construct_handshake();
    socket.write_all(&handshake_msg).await?;

    let mut peer_handshake = [0; 68];
    socket.read_exact(&mut peer_handshake).await?;

    let (info_hash, peer_id) = parse_handshake(&peer_handshake).unwrap_or_default();
    println!("Peer Info Hash: {:?}, Peer ID: {:?}", info_hash, peer_id);

    let mut piece_manager = PieceManager::new(100); // Example total pieces

  
    let mut buffer = [0; 1024];
    let bytes_read = socket.read(&mut buffer).await?;
    let message = &buffer[..bytes_read];

    process_message(message, &mut piece_manager, &mut socket).await?;

    loop {
        print!("Enter a command (request/quit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim();

        match command {
            "request" => {
                if let Some(requested_piece) = piece_manager.generate_request() {
                    println!("Requesting piece: {}", requested_piece);
                } else {
                    println!("No more pieces to request");
                }
            }
            "quit" => {
                println!("Quitting...");
                break;
            }
            _ => {
                println!("Invalid command");
            }
        }
    }

    Ok(())
}


impl PieceManager {
    fn new(total_pieces: u32) -> Self {
        PieceManager {
            total_pieces,
            downloaded_pieces: HashSet::new(),
            requested_pieces: HashSet::new(),
            pieces_data: Vec::new(), // Initialize pieces_data as an empty vector
        }
    }

    fn _mark_piece_as_downloaded(&mut self, piece_index: u32) {
        self.downloaded_pieces.insert(piece_index);
        self.requested_pieces.remove(&piece_index);
    }

    fn mark_piece_as_requested(&mut self, piece_index: u32) {
        if !self.downloaded_pieces.contains(&piece_index) {
            self.requested_pieces.insert(piece_index);
        }
    }

    fn generate_request(&mut self) -> Option<u32> {
        let available_pieces: Vec<u32> = (0..self.total_pieces)
            .filter(|&p| !self.downloaded_pieces.contains(&p) && !self.requested_pieces.contains(&p))
            .collect();

        if let Some(&piece_to_request) = available_pieces.choose(&mut rand::thread_rng()) {
            self.mark_piece_as_requested(piece_to_request);
            Some(piece_to_request)
        } else {
            None
        }
    }

    fn store_piece(&mut self, piece_index: u32, block_offset: u32, block_data: &[u8]) {
        // Assuming PieceManager has a Vec<Vec<u8>> to store pieces' data
        // The outer Vec represents pieces indexed by piece_index
        // The inner Vec stores blocks of data for a piece indexed by block_offset
    
        // Ensure the outer Vec has enough space for piece_index
        while self.pieces_data.len() <= piece_index as usize {
            self.pieces_data.push(Vec::new());
        }
    
        // Ensure the inner Vec has enough space for block_offset
        while self.pieces_data[piece_index as usize].len() <= block_offset as usize {
            self.pieces_data[piece_index as usize].push(Vec::new());
        }
    
        // Store block_data at the appropriate position in the pieces_data structure
        self.pieces_data[piece_index as usize][block_offset as usize].extend_from_slice(block_data);
    }
    
    fn all_pieces_downloaded(&self) -> bool {
        // Check if the number of downloaded pieces matches the total number of pieces
        self.downloaded_pieces.len() == self.total_pieces as usize
    }
}    


// Function to handle peer download
async fn handle_peer_download(peer_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(peer_addr).await?;
    let handshake_msg = construct_handshake(); // Implement the handshake construction logic

    // Send handshake message
    socket.write_all(&handshake_msg).await?;

    // Read response handshake from peer
    let mut response = [0; 68];
    socket.read_exact(&mut response).await?;

    // Process the response handshake
    // ...

    // Implement logic for handling the BitTorrent protocol communication with the peer
    // This involves sending and receiving messages as per the protocol

    Ok(())
}

