**Rustorrent**
-
Rustorrent is a work-in-progress BitTorrent client written in Rust. This project aims to implement a functional BitTorrent client with various features, utilizing Rust's concurrency and asynchronous capabilities.

**Project Overview**
This BitTorrent client is built in Rust and offers the following functionalities:

- **Torrent Info Input:** Allows users to provide information via three methods:

  - Torrent file
  - Magnet URL
  - Info Hash
  
- **Fetching Tracker URL:** Currently, the logic for fetching the tracker URL is not implemented. This functionality is a work in progress.

- **Peer Interaction:** The client fetches peers from the tracker URL and initiates downloading pieces from these peers using the BitTorrent protocol.

**Usage**
*Building and Running the Client:*

  - To build and run the Rustorrent BitTorrent client:
  
    - Clone the repository:
      
           *git clone https://github.com/your-username/rustorrent.git*

    - Navigate to the project directory:

           *cd rustorrent*

    - Compile and run the project:

           *cargo run*

**Usage Flow:**

- Upon execution, the client prompts the user to select the type of torrent info to provide (torrent file, magnet URL, or info hash). Based on the input, the client then fetches the tracker URL and proceeds with fetching peers and initiating the download process.

**Contribution and Development**
This project is a work in progress. Contributions are welcome! If you're interested in contributing to Rustorrent, feel free to fork the repository, make changes, and submit pull requests. Below are some areas that require attention:

**Implement Fetching Tracker URL:** Logic for fetching the tracker URL from torrent files, magnet links, or info hashes needs implementation.

**Testing:** The codebase requires thorough testing to ensure functionality and stability.

**Code Segmentation:** The code currently resides in a single main.rs file. Consider segmenting the code into smaller, manageable modules.

**License**
This project is licensed under the MIT License - see the LICENSE file for details.

**Acknowledgments**
-
