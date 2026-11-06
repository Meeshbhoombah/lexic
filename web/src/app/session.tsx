"use client";

import { useCallback, useEffect, useRef, useState } from "react";

type Session = { account_id: string; status: string; account_created_at: string; expires_at: string };
type Conversation = { id: string; kind: string; created_at: string; participant_count: number; last_message_at: string | null };
type Message = { id: string; conversation_id: string; sender_account_id: string; client_message_id: string; protocol_version: number; nonce: string; ciphertext: string; created_at: string; text?: string };
type KeyResponse = { account_id: string; public_key: string; key_version: number };
const enc = new TextEncoder();
const dec = new TextDecoder();
const b64 = (bytes: Uint8Array) => btoa(Array.from(bytes, b => String.fromCharCode(b)).join("")).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
const unb64 = (value: string) => Uint8Array.from(atob(value.replace(/-/g, "+").replace(/_/g, "/") + "===".slice((value.length + 3) % 4)), c => c.charCodeAt(0));
const uuid = () => crypto.randomUUID();
let sessionBootstrapPromise: Promise<Session> | null = null;
function getSession(): Promise<Session> {
  if (!sessionBootstrapPromise) sessionBootstrapPromise = api<Session>("/api/session", { method: "POST" });
  return sessionBootstrapPromise;
}
const api = async <T,>(url: string, init?: RequestInit): Promise<T> => {
  const response = await fetch(url, { ...init, cache: "no-store", headers: { "content-type": "application/json", ...init?.headers } });
  if (!response.ok) throw new Error(`Request failed (${response.status}).`);
  return response.json() as Promise<T>;
};

async function loadKeyPair(accountId: string): Promise<CryptoKeyPair> {
  const db = await new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open("lexic-e2ee", 1);
    request.onupgradeneeded = () => request.result.createObjectStore("keys");
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
  const keyName = `v1:${accountId}`;
  const existing = await new Promise<CryptoKeyPair | undefined>((resolve, reject) => {
    const request = db.transaction("keys", "readonly").objectStore("keys").get(keyName);
    request.onsuccess = () => resolve(request.result as CryptoKeyPair | undefined);
    request.onerror = () => reject(request.error);
  });
  const pair = existing ?? await crypto.subtle.generateKey({ name: "ECDH", namedCurve: "P-256" }, false, ["deriveBits"]);
  if (!existing) await new Promise<void>((resolve, reject) => {
    const tx = db.transaction("keys", "readwrite");
    tx.objectStore("keys").put(pair, keyName);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
  db.close();
  const rawPublic = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
  await api<KeyResponse>("/api/v1/accounts/me/encryption-key", { method: "PUT", body: JSON.stringify({ public_key: b64(rawPublic) }) });
  return pair;
}

async function trustedPeerKey(accountId: string): Promise<KeyResponse> {
  const remote = await api<KeyResponse>(`/api/v1/accounts/${accountId}/encryption-key`);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", unb64(remote.public_key)));
  const fingerprint = b64(digest);
  const storageKey = `lexic-peer-key-v1:${accountId}`;
  const pinned = localStorage.getItem(storageKey);
  if (pinned && pinned !== fingerprint) throw new Error("This account’s encryption key changed. Verify the account identity with your contact before continuing.");
  if (!pinned) localStorage.setItem(storageKey, fingerprint);
  return remote;
}

async function messageKey(privateKey: CryptoKey, peerPublic: string, conversationId: string): Promise<CryptoKey> {
  const pub = await crypto.subtle.importKey("raw", unb64(peerPublic), { name: "ECDH", namedCurve: "P-256" }, false, []);
  const shared = await crypto.subtle.deriveBits({ name: "ECDH", public: pub }, privateKey, 256);
  const material = await crypto.subtle.importKey("raw", shared, "HKDF", false, ["deriveKey"]);
  return crypto.subtle.deriveKey({ name: "HKDF", hash: "SHA-256", salt: enc.encode(conversationId), info: enc.encode("Lexic-E2EE-message-v1") }, material, { name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"]);
}

export default function SessionBootstrap() {
  const [session, setSession] = useState<Session | null>(null);
  const [privateKey, setPrivateKey] = useState<CryptoKey | null>(null);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [displayMessages, setDisplayMessages] = useState<Message[]>([]);
  const [peerId, setPeerId] = useState("");
  const [draft, setDraft] = useState("");
  const [error, setError] = useState("");
  const busy = useRef(false);

  const refresh = useCallback(async () => {
    if (busy.current) return;
    busy.current = true;
    try {
      const list = await api<Conversation[]>("/api/v1/conversations");
      setConversations(list);
      if (selected) {
        const raw = await api<{ messages: Message[] }>(`/api/v1/conversations/${selected}/messages?limit=100`);
        setMessages(raw.messages);
      }
    } catch (e) { setError(e instanceof Error ? e.message : "Unable to refresh messages."); }
    finally { busy.current = false; }
  }, [selected]);

  useEffect(() => {
    let alive = true;
    void (async () => {
      try {
        const s = await getSession();
        if (!alive) return;
        setSession(s);
        const keys = await loadKeyPair(s.account_id);
        if (alive) setPrivateKey(keys.privateKey);
      } catch (e) { if (alive) setError(e instanceof Error ? e.message : "Unable to initialize your account."); }
    })();
    return () => { alive = false; };
  }, []);

  useEffect(() => { if (!session) return; const initial = window.setTimeout(() => void refresh(), 0); const timer = setInterval(() => void refresh(), 1200); return () => { clearTimeout(initial); clearInterval(timer); }; }, [session, refresh]);

  useEffect(() => {
    if (!selected || !session || !privateKey) return;
    let cancelled = false;
    void (async () => {
      try {
        const participants = await api<{ account_id: string }[]>(`/api/v1/conversations/${selected}/participants`);
        const peer = participants.find(p => p.account_id !== session.account_id);
        if (!peer) return;
        const keyData = await trustedPeerKey(peer.account_id);
        const key = await messageKey(privateKey, keyData.public_key, selected);
        const decrypted = await Promise.all(messages.map(async m => {
          try {
            const nonce = unb64(m.nonce);
            const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce, additionalData: enc.encode(selected) }, key, unb64(m.ciphertext));
            return { ...m, text: dec.decode(plain) };
          } catch { return { ...m, text: "[Unable to decrypt this message]" }; }
        }));
        if (!cancelled) setDisplayMessages(decrypted);
      } catch { /* peer may not have registered a key yet; keep ciphertext hidden */ }
    })();
    return () => { cancelled = true; };
  }, [selected, session, privateKey, messages]);

  async function startConversation() {
    setError("");
    try {
      const result = await api<{ conversation_id: string }>("/api/v1/conversations", { method: "POST", body: JSON.stringify({ other_account_id: peerId.trim() }) });
      setSelected(result.conversation_id); setPeerId(""); await refresh();
    } catch (e) { setError(e instanceof Error ? e.message : "Could not start conversation."); }
  }

  async function sendMessage(event: React.FormEvent) {
    event.preventDefault();
    if (!selected || !session || !privateKey || !draft.trim()) return;
    setError("");
    try {
      const participants = await api<{ account_id: string }[]>(`/api/v1/conversations/${selected}/participants`);
      const peer = participants.find(p => p.account_id !== session.account_id);
      if (!peer) throw new Error("A direct conversation needs another participant.");
      const publicKey = await trustedPeerKey(peer.account_id);
      const key = await messageKey(privateKey, publicKey.public_key, selected);
      const nonce = crypto.getRandomValues(new Uint8Array(12));
      const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce, additionalData: enc.encode(selected) }, key, enc.encode(draft));
      await api(`/api/v1/conversations/${selected}/messages`, { method: "POST", body: JSON.stringify({ client_message_id: uuid(), protocol_version: 1, nonce: b64(nonce), ciphertext: b64(new Uint8Array(ciphertext)) }) });
      setDraft(""); await refresh();
    } catch (e) { setError(e instanceof Error ? e.message : "Unable to send message."); }
  }

  return <main className="min-h-screen bg-neutral-100 p-5 text-neutral-950 md:p-10">
    <section className="mx-auto grid min-h-[80vh] max-w-5xl overflow-hidden rounded-2xl border border-neutral-200 bg-white shadow-sm md:grid-cols-[300px_1fr]">
      <aside className="border-b border-neutral-200 p-5 md:border-b-0 md:border-r"><p className="text-xs font-semibold tracking-[.25em] text-neutral-500">LEXIC</p><h1 className="mt-3 text-xl font-semibold">Secure messages</h1>
        {session && <div className="mt-5 rounded-lg bg-neutral-100 p-3"><div className="text-xs text-neutral-500">Your account ID</div><div className="mt-1 break-all font-mono text-xs">{session.account_id}</div></div>}
        <form className="mt-5 space-y-2" onSubmit={e => { e.preventDefault(); void startConversation(); }}><label className="text-sm font-medium" htmlFor="peer">Start a conversation</label><input id="peer" value={peerId} onChange={e => setPeerId(e.target.value)} placeholder="Other account ID" className="w-full rounded-lg border border-neutral-300 px-3 py-2 text-sm"/><button className="w-full rounded-lg bg-neutral-900 px-3 py-2 text-sm font-medium text-white">Open conversation</button></form>
        <div className="mt-6 space-y-1">{conversations.map(c => <button key={c.id} onClick={() => setSelected(c.id)} className={`w-full rounded-lg p-3 text-left text-sm ${selected === c.id ? "bg-neutral-900 text-white" : "hover:bg-neutral-100"}`}><span className="block font-medium">{c.kind === "direct" ? "Direct conversation" : "Group conversation"}</span><span className="mt-1 block truncate font-mono text-[10px] opacity-70">{c.id}</span></button>)}</div>
      </aside>
      <section className="flex min-h-[60vh] flex-col"><header className="border-b border-neutral-200 p-5"><h2 className="font-semibold">{selected ? "End-to-end encrypted" : "Choose a conversation"}</h2><p className="mt-1 text-xs text-neutral-500">Messages encrypt in your browser. Updates refresh automatically.</p></header>
        {error && <p role="alert" className="m-4 rounded-lg bg-red-50 p-3 text-sm text-red-800">{error}</p>}
        <div className="flex-1 space-y-3 overflow-auto p-5">{displayMessages.map(m => <article key={m.id} className={`max-w-[85%] rounded-xl p-3 text-sm ${m.sender_account_id === session?.account_id ? "ml-auto bg-neutral-900 text-white" : "bg-neutral-100"}`}><p className="whitespace-pre-wrap break-words">{m.text ?? "Decrypting…"}</p><time className="mt-2 block text-[10px] opacity-60">{new Date(m.created_at).toLocaleTimeString()}</time></article>)}</div>
        <form onSubmit={sendMessage} className="flex gap-2 border-t border-neutral-200 p-4"><input aria-label="Message" value={draft} onChange={e => setDraft(e.target.value)} disabled={!selected} placeholder="Write an encrypted message" className="min-w-0 flex-1 rounded-lg border border-neutral-300 px-3 py-2 text-sm"/><button disabled={!selected || !draft.trim()} className="rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-40">Send</button></form>
      </section>
    </section>
    <p className="mx-auto mt-4 max-w-5xl text-xs leading-5 text-neutral-500">Your non-exportable encryption private key stays in this browser. Peer keys are pinned on first use; verify your contact’s account ID through a separate trusted channel. A changed key blocks sending and decryption.</p>
  </main>;
}
