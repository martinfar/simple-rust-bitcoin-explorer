# Bitcoin Block Explorer API Documentation

## Overview

This API provides access to Bitcoin blockchain data, including block information, transaction details, and the latest blocks. It interfaces with a Bitcoin node through RPC calls and exposes the data via HTTP endpoints.

## Base URL

The base URL for the API is configured in the `config.yaml` file. By default, it uses the host and port specified in the `ServerConfig` struct.

## Endpoints

### 1. Get Block Information

Retrieves detailed information about a specific block.

- **URL:** `/block/{hash}`
- **Method:** GET
- **URL Params:**
    - `hash`: The hash of the block to retrieve

#### Success Response

- **Code:** 200 OK
- **Content:** JSON object containing block information

#### Error Response

- **Code:** 400 Bad Request
    - **Content:** `"Invalid block hash"`
- **Code:** 500 Internal Server Error
    - **Content:** `"Failed to retrieve block information"`

### 2. Get Transaction Information

Retrieves detailed information about a specific transaction.

- **URL:** `/tx/{txid}`
- **Method:** GET
- **URL Params:**
    - `txid`: The ID of the transaction to retrieve

#### Success Response

- **Code:** 200 OK
- **Content:** JSON object containing transaction information

#### Error Response

- **Code:** 500 Internal Server Error
    - **Content:** `"Failed to retrieve transaction information"`

### 3. Get Latest Blocks

Retrieves information about the 10 most recent blocks.

- **URL:** `/latest_blocks`
- **Method:** GET

#### Success Response

- **Code:** 200 OK
- **Content:** JSON array containing information about the 10 most recent blocks

#### Error Response

- **Code:** 500 Internal Server Error
    - **Content:** `"Failed to retrieve latest blocks"`

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of requests. In case of errors, a brief error message is returned in the response body.

## Authentication

This API does not implement authentication for client requests. However, it uses basic authentication for RPC calls to the Bitcoin node, configured in the `config.yaml` file.

## Rate Limiting

This API does not implement rate limiting. Consider adding rate limiting in a production environment to prevent abuse.

## Logging

The API uses the `log` crate for logging. Log levels can be configured using the `RUST_LOG` environment variable.

## Configuration

The API is configured using a `config.yaml` file, which should contain the following structure:

```yaml
rpc:
  url: "http://localhost:8332"
  user: "rpcuser"
  pass: "rpcpassword"
server:
  host: "127.0.0.1"
  port: 8080
```

Adjust these values according to your Bitcoin node configuration and desired API server settings.

## Running the API

To run the API, ensure you have Rust installed and then execute:

```
cargo run
```

Make sure the `config.yaml` file is in the same directory as the binary.

## Dependencies

This API uses several Rust crates, including:
- `actix-web` for the web server
- `bitcoin` for Bitcoin-specific types
- `serde` and `serde_json` for JSON serialization/deserialization
- `reqwest` for making HTTP requests to the Bitcoin RPC
- `log` and `env_logger` for logging

Ensure all dependencies are properly listed in your `Cargo.toml` file.