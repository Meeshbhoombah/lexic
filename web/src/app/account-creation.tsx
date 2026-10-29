"use client";

import { useState } from "react";

type Props = {
  credential: string | null;
  connected: boolean;
};

type CreateAccountResponse = {
  account_id?: string;
  status?: string;
  created_at?: string;
  error?: {
    code?: string;
    message?: string;
  };
};

type AccountState = "idle" | "creating" | "created" | "error";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_LEXIC_API_URL ??
  "http://127.0.0.1:8080";

export default function AccountCreation({
  credential,
  connected,
}: Props) {
  const [state, setState] = useState<AccountState>("idle");
  const [accountId, setAccountId] = useState<string | null>(null);
  const [message, setMessage] = useState("");

  async function createAccount(): Promise<void> {
    if (!connected || !credential || state === "creating") {
      return;
    }

    setState("creating");
    setMessage("");
    setAccountId(null);

    console.info("[Lexic/Account]", {
      event: "account.creation_started",
    });

    try {
      const response = await fetch(
        `${API_BASE_URL}/api/v1/accounts`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ credential }),
        },
      );

      const result =
        (await response.json()) as CreateAccountResponse;

      if (!response.ok || !result.account_id) {
        throw new Error(
          result.error?.message ??
            "The Lexic account could not be created.",
        );
      }

      setAccountId(result.account_id);
      setState("created");
      setMessage("Your Lexic account was created.");

      console.info("[Lexic/Account]", {
        event: "account.creation_succeeded",
        accountId: result.account_id,
        status: result.status,
        createdAt: result.created_at,
      });
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Unable to create your Lexic account.";

      setState("error");
      setMessage(errorMessage);

      console.error("[Lexic/Account]", {
        event: "account.creation_failed",
        errorMessage,
      });
    }
  }

  return (
    <section className="mt-6 border-t border-neutral-200 pt-6">
      <h2 className="text-base font-semibold">
        Create your Lexic account
      </h2>

      <p className="mt-2 text-sm leading-6 text-neutral-600">
        Connect your Lexe wallet, then create a persistent
        Lexic account linked to that authorization.
      </p>

      <button
        type="button"
        onClick={() => void createAccount()}
        disabled={
          !connected ||
          !credential ||
          state === "creating" ||
          state === "created"
        }
        className="mt-4 w-full rounded-lg bg-neutral-950 px-4 py-3 text-sm font-medium text-white transition hover:bg-neutral-800 disabled:cursor-not-allowed disabled:opacity-40"
      >
        {state === "creating" && "Creating account…"}
        {state === "created" && "Account created"}
        {(state === "idle" || state === "error") &&
          (connected
            ? "Create Lexic Account"
            : "Connect your Lexe wallet first")}
      </button>

      {message && (
        <p
          role={state === "error" ? "alert" : "status"}
          className="mt-3 text-sm leading-6 text-neutral-600"
        >
          {message}
        </p>
      )}

      {accountId && (
        <div className="mt-4 rounded-lg bg-neutral-100 p-3">
          <p className="text-xs font-medium uppercase tracking-wide text-neutral-500">
            Your Lexic account ID
          </p>

          <code className="mt-2 block break-all text-sm">
            {accountId}
          </code>
        </div>
      )}
    </section>
  );
}
