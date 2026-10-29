"use client";

import { useEffect, useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import {
  CipherSuite,
  DhkemX25519HkdfSha256,
  HkdfSha256,
} from "@hpke/core";
import { Chacha20Poly1305 } from "@hpke/chacha20poly1305";
import AccountCreation from "./account-creation";


const CONNECT_URL = "https://lexe.app/connect";
const MAILBOX_URL = "https://lexe.app/mailbox";
const DOMAIN_SEPARATOR = "LexeConnect-v1";
const REQUEST_LIFETIME_MS = 5 * 60 * 1000;
const POLL_INTERVAL_MS = 1500;

type ConnectionResponse = {
  credential?: string;
  error?: string;
  error_message?: string;
  one_time_secret?: string;
  scopes?: string[];
  permissions?: string[];
  expires_at?: number;
};

type ConnectionState =
  | "preparing"
  | "waiting"
  | "connected"
  | "rejected"
  | "expired"
  | "error";

type LogDetails = Record<
  string,
  string | number | boolean | null | string[] | undefined
>;

function logEvent(event: string, details: LogDetails = {}): void {
  console.info("[Lexic/LexeConnect]", {
    timestamp: new Date().toISOString(),
    event,
    ...details,
  });
}

function logFailure(event: string, error: unknown): void {
  const errorMessage =
    error instanceof Error ? error.message : "Unknown error";

  console.error("[Lexic/LexeConnect]", {
    timestamp: new Date().toISOString(),
    event,
    errorMessage,
  });
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
}

function concatenate(
  first: Uint8Array,
  second: Uint8Array,
): Uint8Array {
  const result = new Uint8Array(first.length + second.length);
  result.set(first);
  result.set(second, first.length);
  return result;
}

function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });
}

export default function LexeConnect() {
  const [requestUrl, setRequestUrl] = useState("");
  const [status, setStatus] = useState<ConnectionState>("preparing");
  const [message, setMessage] = useState("");
  const [credential, setCredential] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    let cancelled = false;

    async function startConnection(): Promise<void> {
      setRequestUrl("");
      setMessage("");
      setCredential(null);
      setStatus("preparing");

      logEvent("connection.initializing", {
        protocol: "LexeConnect",
        version: 1,
        deliveryMethod: "mailbox",
      });

      try {
        if (!globalThis.crypto?.subtle) {
          throw new Error(
            "Secure browser cryptography is unavailable. Open Lexic over HTTPS or localhost.",
          );
        }

        logEvent("crypto.initializing", {
          kem: "DHKEM(X25519, HKDF-SHA256)",
          kdf: "HKDF-SHA256",
          aead: "ChaCha20-Poly1305",
        });

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
          throw new Error(
            `Invalid HPKE public key encoding: expected 64 hex characters, got ${publicKeyHex.length}.`,
          );
        }

        logEvent("crypto.keypair_generated", {
          algorithm: "X25519",
          publicKeyHexLength: publicKeyHex.length,
          privateKeyLogged: false,
        });

        const secretBytes = crypto.getRandomValues(
          new Uint8Array(16),
        );
        const oneTimeSecret = toHex(secretBytes);

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

        logEvent("mailbox.address_derived", {
          endpoint: MAILBOX_URL,
          addressLength: mailboxAddress.length,
          addressLogged: false,
        });

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
        connectionUrl.searchParams.set("label", "Lexic");

        if (cancelled) {
          return;
        }

        setRequestUrl(connectionUrl.toString());

        logEvent("connection.qr_ready", {
          connectHost: connectionUrl.hostname,
          protocolVersion: 1,
          requestedScopes: ["read_info"],
          qrUrlLogged: false,
        });

        setStatus("waiting");

        logEvent("mailbox.polling_started", {
          endpoint: MAILBOX_URL,
          intervalMs: POLL_INTERVAL_MS,
          timeoutMs: REQUEST_LIFETIME_MS,
        });

        const deadline = Date.now() + REQUEST_LIFETIME_MS;
        let pollCount = 0;
        let notFoundCount = 0;

        while (!cancelled && Date.now() < deadline) {
          const mailboxUrl = new URL(MAILBOX_URL);
          mailboxUrl.searchParams.set("address", mailboxAddress);

          pollCount += 1;

          const response = await fetch(mailboxUrl.toString(), {
            method: "GET",
            cache: "no-store",
          });

          if (cancelled) {
            return;
          }

          if (response.status === 404) {
            notFoundCount += 1;

            if (notFoundCount === 1 || notFoundCount % 5 === 0) {
              logEvent("mailbox.response_not_ready", {
                httpStatus: 404,
                totalPolls: pollCount,
                notFoundResponses: notFoundCount,
                elapsedMs:
                  REQUEST_LIFETIME_MS - (deadline - Date.now()),
              });
            }

            await wait(POLL_INTERVAL_MS);
            continue;
          }

          if (!response.ok) {
            logEvent("mailbox.http_error", {
              httpStatus: response.status,
              totalPolls: pollCount,
            });

            throw new Error(
              `Lexe mailbox returned HTTP ${response.status}.`,
            );
          }

          logEvent("mailbox.response_received", {
            httpStatus: response.status,
            totalPolls: pollCount,
            previousNotFoundResponses: notFoundCount,
          });

          const blob = new Uint8Array(await response.arrayBuffer());

          logEvent("mailbox.payload_received", {
            encryptedPayloadBytes: blob.length,
            minimumExpectedBytes: 33,
          });

          if (blob.length <= 32) {
            throw new Error(
              "The Lexe mailbox response was too short.",
            );
          }

          logEvent("crypto.decryption_started", {
            algorithm: "HPKE",
            encapsulatedKeyBytes: 32,
            ciphertextBytes: blob.length - 32,
          });

          const recipient = await suite.createRecipientContext({
            recipientKey: keyPair.privateKey,
            enc: blob.slice(0, 32),
            info,
          });

          const plaintext = await recipient.open(blob.slice(32));

          logEvent("crypto.decryption_succeeded", {
            plaintextBytes: plaintext.length,
            plaintextLogged: false,
          });

          let result: ConnectionResponse;

          try {
            result = JSON.parse(
              new TextDecoder().decode(plaintext),
            ) as ConnectionResponse;
          } catch {
            throw new Error(
              "The decrypted Lexe response was not valid JSON.",
            );
          }

          logEvent("response.json_parsed", {
            credentialPresent: Boolean(result.credential),
            errorPresent: Boolean(result.error),
            oneTimeSecretPresent: Boolean(result.one_time_secret),
            scopesPresent: Array.isArray(result.scopes),
            responseLogged: false,
          });

          if (result.one_time_secret !== oneTimeSecret) {
            logEvent("response.validation_failed", {
              check: "one_time_secret_match",
              passed: false,
            });

            throw new Error(
              "The Lexe response did not match this connection request.",
            );
          }

          logEvent("response.validation_succeeded", {
            check: "one_time_secret_match",
            passed: true,
          });

          if (cancelled) {
            return;
          }

          if (result.error) {
            logEvent("connection.rejected", {
              errorCode: result.error,
              errorMessagePresent: Boolean(result.error_message),
            });

            setStatus("rejected");
            setMessage(
              result.error_message ??
                "The connection was not approved.",
            );
            return;
          }

          if (!result.credential) {
            throw new Error(
              "Lexe returned no client credential.",
            );
          }

          if (
            result.scopes &&
            (!Array.isArray(result.scopes) ||
              result.scopes.some(
                (scope) => typeof scope !== "string",
              ))
          ) {
            throw new Error(
              "Lexe returned an invalid credential scopes field.",
            );
          }

          setCredential(result.credential);
          setStatus("connected");
          setMessage(
            "Your Lexe wallet is connected for this browser session.",
          );

          logEvent("connection.success", {
            credentialReceived: true,
            credentialLogged: false,
            grantedScopes: result.scopes ?? [],
            grantedPermissions: result.permissions ?? [],
            expirationProvided: result.expires_at !== undefined,
            oneTimeSecretMatched: true,
            hpkeDecryptionSucceeded: true,
          });

          return;
        }

        if (!cancelled) {
          logEvent("connection.expired", {
            totalPolls: pollCount,
            notFoundResponses: notFoundCount,
            timeoutMs: REQUEST_LIFETIME_MS,
          });

          setStatus("expired");
          setMessage(
            "This QR code expired. Generate a new one to try again.",
          );
        }
      } catch (error: unknown) {
        if (cancelled) {
          return;
        }

        logFailure("connection.failed", error);

        setStatus("error");
        setMessage(
          error instanceof Error
            ? error.message
            : "Unable to initialize LexeConnect.",
        );
      }
    }

    void startConnection();

    return () => {
      cancelled = true;
      logEvent("connection.effect_cleanup", {
        reason: "component_unmounted_or_request_restarted",
      });
    };
  }, [refreshKey]);

  return (
    <main className="flex min-h-screen items-center justify-center bg-neutral-50 px-5 py-12 text-neutral-950">
      <div className="w-full max-w-md">
        <header className="mb-8 text-center">
          <p className="text-sm font-semibold tracking-[0.25em] text-neutral-500">
            LEXIC
          </p>

          <h1 className="mt-4 text-3xl font-semibold tracking-tight">
            Connect your wallet
          </h1>

          <p className="mx-auto mt-3 max-w-sm text-sm leading-6 text-neutral-600">
            Scan this code with the Lexe app to connect your
            self-custodial wallet to Lexic.
          </p>
        </header>

        <section className="rounded-2xl border border-neutral-200 bg-white p-6 shadow-sm">
          <div className="mx-auto flex min-h-64 w-full max-w-64 items-center justify-center rounded-xl bg-white p-3">
            {requestUrl ? (
              <QRCodeSVG
                value={requestUrl}
                size={240}
                level="M"
                marginSize={2}
                title="Lexic LexeConnect QR code"
                className="h-auto w-full"
              />
            ) : (
              <div
                className="flex h-60 w-60 items-center justify-center rounded-lg bg-neutral-100 text-center text-sm text-neutral-500"
                aria-live="polite"
              >
                Preparing secure connection…
              </div>
            )}
          </div>

          <div className="mt-5 text-center" aria-live="polite">
            <p className="text-sm font-medium">
              {status === "preparing" && "Preparing connection"}
              {status === "waiting" && "Waiting for Lexe approval"}
              {status === "connected" && "Wallet connected"}
              {status === "rejected" && "Connection declined"}
              {status === "expired" && "QR code expired"}
              {status === "error" && "Connection error"}
            </p>

            {message && (
              <p className="mt-2 text-sm leading-6 text-neutral-600">
                {message}
              </p>
            )}

            {status === "waiting" && (
              <p className="mt-2 text-xs text-neutral-500">
                Approve the request in Lexe. This request asks for
                read-only wallet information.
              </p>
            )}

            {credential && status === "connected" && (
              <p className="mt-2 text-xs text-neutral-500">
                A scoped credential was received and is held in memory.
              </p>
            )}
          </div>

          {(status === "expired" ||
            status === "rejected" ||
            status === "error") && (
            <button
              type="button"
              onClick={() => setRefreshKey((key) => key + 1)}
              className="mt-5 w-full rounded-lg border border-neutral-300 px-4 py-3 text-sm font-medium hover:bg-neutral-50"
            >
              Generate a new QR code
            </button>
          )}

          <AccountCreation
            credential={credential}
            connected={status === "connected"}
          />

        </section>

        <footer className="mt-6 text-center text-xs leading-5 text-neutral-500">
          Lexe manages your wallet. Lexic never asks for your recovery
          phrase.
        </footer>
      </div>
    </main>
  );
}

