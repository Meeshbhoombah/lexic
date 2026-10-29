# Getting Started

This guide walks through integrating [LexeConnect](https://docs.lexe.tech/lexeconnect/) into a web application. By the end, your application will be able to request access to a user's Lexe wallet, display a QR code, receive an encrypted authorization response, and obtain a scoped Lexe client credential.

The implementation follows the browser-based mailbox flow demonstrated in [Lexic](https://github.com/Meeshbhoombah/lexic). The examples use TypeScript, React, and the Web Crypto API alongside HPKE libraries.

> **Implementation note:** Lexe now provides an official [`lexe-connect` Rust crate](https://docs.rs/lexe-connect/latest/lexe_connect/), which is the recommended reference implementation for constructing requests and validating responses. The TypeScript examples below illustrate the integration mechanics used in Lexic; review the [protocol specification](https://github.com/lexe-app/lexe-connect) before using a custom implementation in production.

## 1. Understand the connection flow

LexeConnect allows an application to request a scoped, revocable client credential without receiving the user's wallet root seed. The user approves the request in the Lexe app, and the credential is delivered using one of the protocol's supported delivery methods.

This tutorial implements the **mailbox delivery** method, which works well when a web application displays a QR code on one device and the user approves it on another.

The flow is:

1. Generate an ephemeral encryption key pair and a random one-time secret.
2. Construct a LexeConnect connection URL.
3. Display that URL as a QR code.
4. Let the user scan the code and approve the request in Lexe.
5. Poll the mailbox for the encrypted response.
6. Decrypt and validate the response.
7. Use the resulting credential with the Lexe SDK or another supported Lexe interface.

The mailbox response is encrypted end to end. The mailbox relays the response but does not need access to the credential.

## 2. Create the frontend project

For this example, use a React application with TypeScript. If you already have a React application, use its existing setup and skip the project creation step.

Create a Next.js application:

```bash
bun create next-app lexeconnect-demo
cd lexeconnect-demo
```

Install the libraries used by the integration:

```bash
bun add @hpke/core @hpke/chacha20poly1305 qrcode.react
```

These packages provide:

- `@hpke/core`: HPKE key generation and response decryption.
- `@hpke/chacha20poly1305`: the ChaCha20-Poly1305 authenticated-encryption implementation.
- `qrcode.react`: QR-code rendering.

The example assumes a modern browser with Web Crypto support and a secure context, such as `localhost` during development or HTTPS in production.

## 3. Define the connection parameters

LexeConnect requests are connection URLs built from `https://lexe.app/connect` and URL query parameters.

For this implementation, define the following constants:

```typescript
const CONNECT_URL = "https://lexe.app/connect";
const MAILBOX_URL = "https://lexe.app/mailbox";

const DOMAIN_SEPARATOR = "LexeConnect-v1";
const REQUEST_LIFETIME_MS = 5 * 60 * 1000;
const POLL_INTERVAL_MS = 1500;
```

The domain separator is used when deriving the mailbox address and HPKE context information in this implementation. The request lifetime is an application-side timeout; it does not automatically set the lifetime of the credential granted by Lexe.

The protocol requires a fresh ephemeral public key and one-time secret for each request. The request's one-time secret is echoed in the response so the application can verify that it belongs to the outstanding request.

## 4. Generate the ephemeral key pair

The requester creates a new HPKE key pair for each connection attempt. Its public key is included in the connection URL; its private key must remain private and available until the response is decrypted or the request expires.

Create a helper to encode byte arrays as lowercase hexadecimal strings:

```typescript
function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
}
```

Then initialize HPKE:

```typescript
import {
  CipherSuite,
  DhkemX25519HkdfSha256,
  HkdfSha256,
} from "@hpke/core";

import { Chacha20Poly1305 } from "@hpke/chacha20poly1305";

const suite = new CipherSuite({
  kem: new DhkemX25519HkdfSha256(),
  kdf: new HkdfSha256(),
  aead: new Chacha20Poly1305(),
});

const keyPair = await suite.kem.generateKeyPair();

const publicKey = new Uint8Array(
  await suite.kem.serializePublicKey(keyPair.publicKey),
);

const publicKeyHex = toHex(publicKey);

if (publicKeyHex.length !== 64) {
  throw new Error("Expected a 32-byte X25519 public key.");
}
```

The X25519 public key is encoded as 64 lowercase hexadecimal characters. Do not send the private key to Lexe, expose it to other application components unnecessarily, or log it.

## 5. Generate the one-time secret and mailbox address

Generate a cryptographically secure 16-byte random secret. The protocol representation is 32 lowercase hexadecimal characters.

```typescript
const secretBytes = crypto.getRandomValues(
  new Uint8Array(16),
);

const oneTimeSecret = toHex(secretBytes);
```

Create a helper for concatenating byte arrays:

```typescript
function concatenate(
  first: Uint8Array,
  second: Uint8Array,
): Uint8Array {
  const result = new Uint8Array(
    first.length + second.length,
  );

  result.set(first);
  result.set(second, first.length);

  return result;
}
```

Derive the HPKE context information and mailbox address:

```typescript
const info = concatenate(
  new TextEncoder().encode(DOMAIN_SEPARATOR),
  secretBytes,
);

const addressDigest = await crypto.subtle.digest(
  "SHA-256",
  info,
);

const mailboxAddress = toHex(
  new Uint8Array(addressDigest),
);
```

The mailbox address tells the requester where to poll for the response. The private key, one-time secret, and request context must remain associated with this specific outstanding request.

**Important:** These derivation steps reflect the Lexic integration example. For a new production integration, use the official `lexe-connect` implementation or verify each derivation against the current protocol specification and its test vectors rather than independently reimplementing cryptographic protocol details.

## 6. Construct the LexeConnect URL

Build the connection URL using the public key, mailbox endpoint, one-time secret, and requested scope:

```typescript
const connectionUrl = new URL(CONNECT_URL);

connectionUrl.searchParams.set("v", "1");
connectionUrl.searchParams.set("mailbox_url", MAILBOX_URL);
connectionUrl.searchParams.set(
  "ephemeral_hpke_pubkey",
  publicKeyHex,
);
connectionUrl.searchParams.set(
  "one_time_secret",
  oneTimeSecret,
);
connectionUrl.searchParams.set("scopes", "read_info");
connectionUrl.searchParams.set("label", "My App");
```

The important parameters are:

| Parameter | Purpose |
|---|---|
| `v` | Selects protocol version 1. |
| `mailbox_url` | Specifies where the encrypted response will be delivered. |
| `ephemeral_hpke_pubkey` | Supplies the public key used to encrypt the response. |
| `one_time_secret` | Binds the response to this connection attempt. |
| `scopes` | Specifies the wallet capabilities requested by the application. |
| `label` | Suggests a label for the resulting credential. |

The example requests `read_info`, a read-only scope suitable for accessing basic wallet information. Choose scopes based on the actual functionality you need. For example, an application that receives payments may need `receive`; an application that initiates payments may need `spend`. Consult [Lexe's credential scope documentation](https://docs.lexe.tech/authentication/credential-scopes/) and request only the minimum required access.

Use the URL API rather than concatenating query strings manually so parameter values are encoded correctly.

## 7. Display the connection URL as a QR code

The Lexe app can open the connection URL when the user scans the QR code. In React, render the URL using `QRCodeSVG`:

```tsx
"use client";

import { QRCodeSVG } from "qrcode.react";

type Props = {
  connectionUrl: string;
};

export function LexeConnectQRCode({
  connectionUrl,
}: Props) {
  return (
    <section>
      <h2>Connect your Lexe wallet</h2>

      <p>
        Scan this code with the Lexe app to authorize
        the requested wallet access.
      </p>

      <QRCodeSVG
        value={connectionUrl}
        size={240}
        level="M"
        title="LexeConnect QR code"
      />
    </section>
  );
}
```

Generate a fresh URL for each attempt. Do not reuse the same one-time secret or ephemeral key pair for subsequent requests.

For an app-to-app experience on a supported mobile device, the application can open the connection URL directly instead of displaying a QR code. See the [LexeConnect specification](https://github.com/lexe-app/lexe-connect) for the other supported delivery methods.

## 8. Poll the mailbox for the response

After constructing the request, the application must wait for the user to approve or reject it.

The mailbox is queried using the address derived in Step 5:

```typescript
async function pollMailbox(
  mailboxAddress: string,
  deadline: number,
): Promise<Uint8Array> {
  while (Date.now() < deadline) {
    const url = new URL(MAILBOX_URL);
    url.searchParams.set("address", mailboxAddress);

    const response = await fetch(url.toString(), {
      method: "GET",
      cache: "no-store",
    });

    if (response.status === 404) {
      await new Promise<void>((resolve) => {
        setTimeout(resolve, POLL_INTERVAL_MS);
      });

      continue;
    }

    if (!response.ok) {
      throw new Error(
        `Mailbox returned HTTP ${response.status}.`,
      );
    }

    return new Uint8Array(
      await response.arrayBuffer(),
    );
  }

  throw new Error("LexeConnect request expired.");
}
```

Start polling with a deadline:

```typescript
const deadline =
  Date.now() + REQUEST_LIFETIME_MS;

const encryptedResponse = await pollMailbox(
  mailboxAddress,
  deadline,
);
```

This is a minimal illustration. A production polling loop should also support cancellation when a component unmounts, network timeouts, backoff or sensible polling intervals, and a way for the user to restart an expired request.

An HTTP `404` is treated here as a response that has not arrived yet. Other HTTP errors should not be silently treated as a pending authorization.

## 9. Decrypt the response

In the mailbox flow, the response blob contains the 32-byte HPKE encapsulated key followed by the encrypted payload.

Check the payload length before attempting decryption:

```typescript
if (encryptedResponse.length <= 32) {
  throw new Error("Mailbox response is too short.");
}
```

Create the recipient context using the private key generated for this request:

```typescript
const recipient = await suite.createRecipientContext({
  recipientKey: keyPair.privateKey,
  enc: encryptedResponse.slice(0, 32),
  info,
});

const plaintext = await recipient.open(
  encryptedResponse.slice(32),
);
```

The `info` value must match the context used when the response was encrypted. If decryption fails, reject the response; do not attempt to interpret the ciphertext as JSON or bypass authentication checks.

Parse the decrypted payload:

```typescript
type LexeConnectResponse = {
  credential?: string;
  error?: string;
  error_message?: string;
  one_time_secret?: string;
  account?: string;
  metadata?: string;
  scopes?: string[];
  permissions?: string[];
  expires_at?: number;
};

const result = JSON.parse(
  new TextDecoder().decode(plaintext),
) as LexeConnectResponse;
```

This type describes the fields relevant to the example. It is not a substitute for runtime schema validation.

## 10. Validate the response before accepting the credential

Never accept a credential merely because the response decrypted successfully. Validate that the response corresponds to the outstanding request and represents a successful grant.

First, check the one-time secret:

```typescript
if (result.one_time_secret !== oneTimeSecret) {
  throw new Error(
    "Response does not match the connection request.",
  );
}
```

If the request included an `account` or `metadata` parameter, verify the corresponding echoed response fields as required by the protocol.

Next, handle rejection and success separately:

```typescript
if (result.error) {
  throw new Error(
    result.error_message ??
      "The user rejected the connection request.",
  );
}

if (
  typeof result.credential !== "string" ||
  result.credential.length === 0
) {
  throw new Error(
    "Lexe returned no client credential.",
  );
}

const credential = result.credential;
```

If your application requested specific scopes or permissions, validate the returned grant against those requested values. The protocol requires granted exact parameters to match the request; use the official implementation to perform protocol-level validation where possible.

Once accepted, mark the connection as successful and make the credential available only to the components that need it.

## 11. Use the credential with the Lexe SDK

A successful LexeConnect flow returns a Lexe client credential. It is not a root seed, and it should be treated as a secret because it authorizes the capabilities granted by the user.

For a Rust backend, add the [Lexe Rust SDK](https://docs.lexe.tech/rust/quickstart/) to your project and parse the credential using the SDK's credential types:

```rust
use lexe::{
    config::WalletEnvConfig,
    types::auth::{ClientCredentials, CredentialsRef},
    wallet::LexeWallet,
};

async fn verify_credential(
    credential: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client_credentials =
        ClientCredentials::from_string(credential)?;

    let credentials =
        CredentialsRef::from(&client_credentials);

    let wallet = LexeWallet::load_or_fresh(
        WalletEnvConfig::mainnet(),
        credentials,
        None,
    )?;

    let node_info = wallet.node_info().await?;

    println!("Successfully connected to Lexe.");

    // Use node_info as needed by your application.
    let _ = node_info;

    Ok(())
}
```

This demonstrates parsing the credential and verifying that it can be used to request basic wallet information. The credential must have the scope required by the operation; the example's `read_info` scope is not sufficient for spending funds.

In a web application, avoid exposing a long-lived credential to unrelated frontend code. If a backend needs to use the credential, transmit it over HTTPS, authenticate the application session, and protect it in transit and at rest. Never log the credential.

For SDK setup and error handling, follow the current [Rust quickstart](https://docs.lexe.tech/rust/quickstart/).

## 12. Handle connection states and failures

A robust integration should represent the connection as an explicit state machine. At a minimum, account for:

- **Preparing:** generating keys and constructing the request.
- **Waiting:** displaying the QR code and awaiting approval.
- **Connected:** the response is decrypted and validated, and a credential is available.
- **Rejected:** the user declined the request.
- **Expired:** no response arrived within the request lifetime.
- **Error:** the request could not be initialized, received, decrypted, or validated.

For example, in React:

```typescript
type ConnectionState =
  | "preparing"
  | "waiting"
  | "connected"
  | "rejected"
  | "expired"
  | "error";
```

Keep the private key, one-time secret, and request context associated with the current request. When the request expires, is cancelled, or is replaced, discard its state and generate new values for the next attempt.

Useful diagnostics include event names, HTTP status codes, elapsed time, poll counts, and encrypted payload length. Do not log the credential, private key, one-time secret, complete connection URL, mailbox address, or decrypted response.

## 13. Production checklist

Before shipping the integration:

- [ ] Verify connection construction, mailbox derivation, encryption, and response validation against the current [LexeConnect specification](https://github.com/lexe-app/lexe-connect).
- [ ] Prefer the official [`lexe-connect` crate](https://docs.rs/lexe-connect/latest/lexe_connect/) for protocol-sensitive operations when using Rust.
- [ ] Generate fresh ephemeral keys and a one-time secret for every request.
- [ ] Request only the scopes and permissions your application needs.
- [ ] Validate the one-time secret and all applicable echoed request fields.
- [ ] Reject malformed, expired, rejected, or mismatched responses.
- [ ] Implement cancellation, timeouts, and safe retry behavior.
- [ ] Use HTTPS in production and protect credentials as secrets.
- [ ] Design credential storage, revocation, and reconnection deliberately.
- [ ] Test the complete flow with a real Lexe wallet before release.

## Further reading

- [LexeConnect overview](https://docs.lexe.tech/lexeconnect/)
- [LexeConnect protocol specification](https://github.com/lexe-app/lexe-connect)
- [Official `lexe-connect` Rust crate](https://docs.rs/lexe-connect/latest/lexe_connect/)
- [Lexe client credentials](https://docs.lexe.tech/authentication/client-credentials/)
- [Lexe credential scopes](https://docs.lexe.tech/authentication/credential-scopes/)
- [Lexe Rust SDK quickstart](https://docs.lexe.tech/rust/quickstart/)
- [Lexic's web client implementation](https://github.com/Meeshbhoombah/lexic/tree/main/web/src/app)

