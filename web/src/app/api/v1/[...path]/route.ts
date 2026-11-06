import { NextRequest, NextResponse } from "next/server";

async function proxy(request: NextRequest, context: { params: Promise<{ path: string[] }> }) {
  const apiUrl = process.env.LEXIC_API_URL;
  if (!apiUrl) return NextResponse.json({ error: "LEXIC_API_URL is not configured." }, { status: 500 });
  const { path } = await context.params;
  const url = new URL(`/api/v1/${path.map(encodeURIComponent).join("/")}${request.nextUrl.search}`, apiUrl);
  const headers = new Headers();
  const cookie = request.headers.get("cookie");
  const contentType = request.headers.get("content-type");
  if (cookie) headers.set("cookie", cookie);
  if (contentType) headers.set("content-type", contentType);
  try {
    const upstream = await fetch(url, { method: request.method, headers, body: request.method === "GET" || request.method === "HEAD" ? undefined : await request.arrayBuffer(), cache: "no-store" });
    const response = new NextResponse(upstream.body, { status: upstream.status });
    const setCookie = upstream.headers.get("set-cookie");
    if (setCookie) response.headers.set("set-cookie", setCookie);
    response.headers.set("cache-control", "no-store");
    return response;
  } catch {
    return NextResponse.json({ error: "The Lexic API could not be reached." }, { status: 502 });
  }
}

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
