"use client";

import { useEffect, useState } from "react";

type Session = {
  account_id: string;
  status: string;
  account_created_at: string;
  expires_at: string;
};

type State =
  | { kind: "loading" }
  | { kind: "ready"; session: Session }
  | { kind: "error"; message: string };

let sessionBootstrapPromise: Promise<Session> | null = null;

function getSession(): Promise<Session> {
  if (!sessionBootstrapPromise) {
    sessionBootstrapPromise = (async () => {
      const response = await fetch("/api/session", {
        method: "POST",
        cache: "no-store",
      });

      const body: unknown = await response.json();

      if (!response.ok) {
        throw new Error("Unable to create your Lexic session.");
      }

      if (
        typeof body !== "object" ||
        body === null ||
        !("account_id" in body) ||
        typeof body.account_id !== "string" ||
        !("status" in body) ||
        typeof body.status !== "string" ||
        !("account_created_at" in body) ||
        typeof body.account_created_at !== "string" ||
        !("expires_at" in body) ||
        typeof body.expires_at !== "string"
      ) {
        throw new Error("Invalid session response.");
      }

      return body as Session;
    })();
  }

  return sessionBootstrapPromise;
}

export default function SessionBootstrap() {
  const [state, setState] = useState<State>({
    kind: "loading",
  });

  useEffect(() => {
    void getSession()
      .then((session) => {
        setState({ kind: "ready", session });
      })
      .catch((error: unknown) => {
        setState({
          kind: "error",
          message:
            error instanceof Error
              ? error.message
              : "Unable to create your Lexic session.",
        });
      });
  }, []);

  return (
    <main className="flex min-h-screen items-center justify-center bg-neutral-50 px-5 py-12 text-neutral-950">
      <section className="w-full max-w-md rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm">
        <p className="text-center text-sm font-semibold tracking-[0.25em] text-neutral-500">
          LEXIC
        </p>

        <h1 className="mt-4 text-center text-3xl font-semibold tracking-tight">
          Your account
        </h1>

        {state.kind === "loading" && (
          <p className="mt-6 text-center text-sm text-neutral-600">
            Creating your session…
          </p>
        )}

        {state.kind === "error" && (
          <p className="mt-6 text-center text-sm text-red-700" role="alert">
            {state.message}
          </p>
        )}

        {state.kind === "ready" && (
          <div className="mt-6 space-y-4">
            <p className="text-center text-sm text-neutral-600">
              Your anonymous Lexic account is ready.
            </p>

            <div className="rounded-xl bg-neutral-100 p-4">
              <p className="text-xs font-medium uppercase tracking-wide text-neutral-500">
                Account ID
              </p>

              <p className="mt-2 break-all font-mono text-sm">
                {state.session.account_id}
              </p>
            </div>
          </div>
        )}
      </section>
    </main>
  );
}
