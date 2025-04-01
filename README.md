# WebCheck - Real-time Website Monitoring Tool

[![Rust CI](https://github.com/user-name/webcheck/actions/workflows/rust.yml/badge.svg)](https://github.com/user-name/webcheck/actions/workflows/rust.yml)
[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC%20BY--NC%204.0-lightgrey.svg)](http://creativecommons.org/licenses/by-nc/4.0/)

WebCheck is a lightweight, real-time web resource monitoring application built with Rust. It provides a simple and efficient way to monitor website availability, response times, and performance metrics through an intuitive web interface.

![WebCheck Screenshot](https://via.placeholder.com/800x450.png?text=WebCheck+Screenshot)

## Features

- **Real-time Website Monitoring**: Track the availability and performance of multiple web resources
- **Status Visualization**: Color-coded status indicators (UP, DOWN, UNKNOWN)
- **Performance Metrics**:
  - HTTP status codes
  - Response time in milliseconds
  - Jitter calculation (response time fluctuations)
  - Time since last check
- **Resource Management**:
  - Add new resources to monitor
  - Remove resources from monitoring
  - Configure check intervals
  - Set auto-refresh intervals
- **Persistent Configuration**: Saves settings between restarts
- **Docker Support**: Easy deployment via Docker/docker-compose
- **Systemd Integration**: Run as a system service on Linux

## Getting Started

### Prerequisites

- Rust 1.77.0 or later
- (Optional) Docker and docker-compose for containerized deployment

### Installation

#### Method 1: Building from Source

1. Clone the repository:
   ```bash
   git clone git@github.com:T-6891/webcheck.git
   cd webcheck
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/webcheck
   ```

4. Access the dashboard:
   ```
   http://localhost:3000
   ```

#### Method 2: Using Docker

1. Clone the repository:
   ```bash
   git clone https://github.com/user-name/webcheck.git
   cd webcheck
   ```

2. Build and start the container:
   ```bash
   docker-compose up -d
   ```

3. Access the dashboard:
   ```
   http://localhost:3000
   ```

#### Method 3: Installing as a System Service (Linux)

1. Build the application:
   ```bash
   cargo build --release
   ```

2. Create application directory:
   ```bash
   sudo mkdir -p /opt/webcheck
   ```

3. Copy executable and templates:
   ```bash
   sudo cp target/release/webcheck /opt/webcheck/
   sudo cp -r templates /opt/webcheck/
   ```

4. Install service file:
   ```bash
   sudo cp webcheck.service /etc/systemd/system/
   ```

5. Enable and start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable webcheck
   sudo systemctl start webcheck
   ```

6. Check service status:
   ```bash
   sudo systemctl status webcheck
   ```

## Usage

### Dashboard Overview

The WebCheck dashboard provides a comprehensive view of all monitored resources with the following information:
- URL
- Status (UP/DOWN/UNKNOWN)
- HTTP Status Code
- Response Time
- Jitter (response time fluctuation)
- Last Checked (in minutes)

### Adding Resources

1. In the "Add Resource" section, enter the complete URL (including http:// or https://)
2. Click "Add Resource"

### Removing Resources

1. In the resources table, click the "Remove" button next to the resource you want to stop monitoring

### Configuring Settings

1. In the "Configuration" section:
   - Set "Check Interval" (in seconds) to control how often resources are checked
   - Set "Page Refresh Interval" (in seconds) to control how often the dashboard updates
2. Click "Update Configuration"

### Refreshing Data

- The page automatically refreshes based on your configured interval
- For immediate updates, click the "Refresh Now" button

## Technical Details

WebCheck is built with modern Rust technologies:

- **Backend Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **Templating**: [Askama](https://github.com/djc/askama)
- **Date/Time**: [Chrono](https://github.com/chronotope/chrono)
- **Serialization**: [Serde](https://serde.rs/)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed information on how to contribute to this project.

## License

<a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc/4.0/88x31.png" /></a><br />This work is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">Creative Commons Attribution-NonCommercial 4.0 International License</a>.

## Roadmap

Future development plans include:

- Additional metrics (average response time, availability percentage)
- Extended monitoring capabilities (content validation, SSL certificate checks)
- Notifications via email/Telegram for resource unavailability
- Authentication system
- Historical data storage and visualization
- API for integrating with other monitoring systems
