# Getting Started
This guide explains how to connect a Lexe wallet to Lexic using LexeConnect, test the end-to-end connection flow, and optionally create a Lexic account.

## 1. Overview

Lexic implements LexeConnect in the Next.js browser client. The browser generates a connection request, displays it as a QR code, and waits for the user to approve it in the Lexe app. Lexe returns a scoped client credential through an encrypted mailbox response.

The current implementation uses:

- **Frontend:** Next.js, React, TypeScript, and Bun
- **Connection protocol:** LexeConnect v1
- **Response encryption:** HPKE using X25519, HKDF-SHA256, and ChaCha20-Poly1305
- **Response delivery:** Lexe mailbox at `https://lexe.app/mailbox`
- **Requested scope:** `read_info`
- **Optional backend:** Rust, Axum, SQLx, and PostgreSQL

LexeConnect authorizes Lexic to access the approved wallet capabilities. It does not expose the wallet's root seed or, by itself, establish a persistent Lexic login. See the [official LexeConnect documentation](https://docs.lexe.tech/lexeconnect/).

## 2. Prerequisites

Install the following tools:

- [Git](https://git-scm.com/)
- [Bun](https://bun.sh/)
- [Rust](https://www.rust-lang.org/tools/install) and Cargo, if running the backend
- [PostgreSQL](https://www.postgresql.org/download/), if creating Lexic accounts through the backend
- The [Lexe app](https://docs.lexe.app/getting-started/) with a configured wallet

Check the frontend tools:

```bash
bun --version
git --version
```

If you intend to create a Lexic account, also check Rust and PostgreSQL:

```bash
rustc --version
cargo --version
psql --version
```

## 3. Clone the repository

Clone Lexic and enter the repository:

```bash
git clone https://github.com/Meeshbhoombah/lexic.git
cd lexic
```

The LexeConnect frontend is located in `browser/`. Its main implementation is:

```text
browser/
└── src/
    └── app/
        ├── page.tsx
        ├── lexe.tsx
        └── account-creation.tsx
```

- `page.tsx` renders the LexeConnect experience.
- `lexe.tsx` creates the connection request, renders the QR code, polls the mailbox, decrypts the response, and retains the returned credential in browser memory.
- `account-creation.tsx` optionally submits the credential to the Rust API to create a Lexic account.

## 4. Install and start the browser client

Navigate to the frontend and install its dependencies:

```bash
cd browser
bun install
```

Start the development server:

```bash
bun --bun run dev
```

Open [http://localhost:3000](http://localhost:3000).

The page should display a **Connect your wallet** screen and a QR code once the connection request has been prepared.

Use `localhost` or HTTPS. The browser requires secure-context cryptography for the Web Crypto APIs used by the implementation.

## 5. Connect a Lexe wallet

1. Open the Lexic page at `http://localhost:3000` on your development computer.
2. Open the Lexe app on your phone.
3. Scan the QR code using the Lexe app's connection flow.
4. Review the requested permissions in Lexe.
5. Approve the request.
6. Return to the Lexic browser page and wait for the response to be received and verified.

Lexic currently requests the `read_info` scope and labels the request **Lexic**. This is intended for read-only access to basic wallet and node information, not payment spending.

### What happens under the hood

When the page initializes, `lexe.tsx`:

1. Generates an ephemeral X25519 key pair for encrypting the response.
2. Generates a random one-time secret.
3. Derives a mailbox address using the `LexeConnect-v1` domain separator and the one-time secret.
4. Constructs a LexeConnect v1 URL containing the mailbox endpoint, ephemeral public key, one-time secret, requested scope, and application label.
5. Renders that URL as a QR code.
6. Polls the mailbox every 1.5 seconds for up to five minutes.
7. Decrypts a received response using HPKE.
8. Parses the response and checks that its one-time secret matches the request.
9. Stores the returned client credential in React state and marks the connection as successful.

The credential is held in browser memory for the current page session. The frontend does not intentionally log the credential, private key, or complete QR URL.

## 6. Verify that the connection succeeded

The browser page should display **Wallet connected** and indicate that a scoped credential was received.

For additional verification, open the browser developer tools and select **Console**. Look for structured events prefixed with:

```text
[Lexic/LexeConnect]
```

A successful connection should produce events corresponding to:

```text
connection.initializing
crypto.keypair_generated
connection.qr_ready
mailbox.polling_started
mailbox.response_received
crypto.decryption_succeeded
response.validation_succeeded
connection.success
```

The exact sequence can vary depending on how quickly the request is approved.

The important success conditions are:

- The mailbox returns an encrypted response.
- HPKE decryption succeeds.
- The decrypted response is valid JSON.
- The returned one-time secret matches the request.
- A client credential is present.

A QR code being displayed, or the Lexe app showing an approval screen, is not sufficient by itself to establish that the handshake completed.

## Troubleshooting

### The page never displays a QR code

Check the browser console for `connection.failed`.

Confirm that:

- The frontend dependencies installed successfully.
- You opened the page on `localhost` or HTTPS.
- Browser cryptography APIs are available.
- There are no JavaScript or dependency errors.

### The Lexe app opens, but the browser never connects

Look for `mailbox.response_not_ready` events.

The implementation treats HTTP `404` from the mailbox as a response that is not ready yet and continues polling. Some `404` responses while waiting are expected. If they continue until the five-minute timeout:

1. Confirm that you approved the request in the Lexe app.
2. Generate a fresh QR code and approve that new request.
3. Check whether the mailbox returns a non-404 HTTP error.
4. Check the LexeConnect protocol and mailbox behavior against the [official documentation](https://docs.lexe.tech/lexeconnect/).

Do not share the complete QR URL or mailbox address in debugging reports.

### The browser reports a decryption or validation error

Inspect the event immediately preceding `connection.failed`.

- A decryption failure can indicate a mismatch in the HPKE parameters, encapsulated key, or response payload.
- Invalid JSON indicates that decryption completed but the plaintext was not a valid response object.
- A one-time-secret mismatch means the response did not match the current connection request.
- A missing credential means the response did not contain the expected authorization result.

Generate a new QR code before retrying, and do not bypass the one-time-secret validation.

### The wallet connects, but account creation fails

Check that the Rust API is running and the database is reachable:

```bash
curl http://127.0.0.1:8080/health
pg_isready -h 127.0.0.1 -p 5432
```

Then inspect the Rust server's logs. The current backend verifies the credential by calling Lexe's `node_info()` API; a valid-looking credential is not enough if Lexe rejects it or the network request fails.

Confirm that the requested `read_info` scope is present and that the local database migrations completed successfully.

### The account is created, but the credential is not saved

This is expected in the current implementation. Lexic stores an account and its Lexe connection record, but does not store the credential itself. The browser currently retains the credential only in memory.

## Security considerations

- **Request minimal scopes.** The current `read_info` scope is appropriate for verifying a basic connection. Only request payment or spending permissions when the feature requires them.
- **Never request or handle the root seed phrase.** LexeConnect is designed to provide scoped client credentials without exposing the wallet's root seed.
- **Treat client credentials as secrets.** Do not print them to logs, include them in URLs, commit them, or expose them in error messages.
- **Keep the ephemeral private key private.** The browser uses it to decrypt the mailbox response.
- **Use HTTPS in production.** Localhost is suitable for development; production deployment requires secure transport and an appropriate origin/CORS configuration.
- **Plan credential revocation and reconnection.** Lexe credentials are scoped and revocable. The application should handle expired or revoked credentials and allow users to reconnect.
- **Do not treat LexeConnect as persistent Lexic authentication.** A separate session and identity design is required for secure returning-user login.

## 10. Reference links

- [LexeConnect overview and integration guide](https://docs.lexe.tech/lexeconnect/)
- [Lexe client credentials](https://docs.lexe.tech/authentication/client-credentials/)
- [Lexe Rust SDK quickstart](https://docs.lexe.tech/rust/quickstart/)
- [Lexe public source repository](https://github.com/lexe-app/lexe-public)
- [Lexic repository](https://github.com/Meeshbhoombah/lexic)


