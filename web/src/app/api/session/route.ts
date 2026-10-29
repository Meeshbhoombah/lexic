import { NextRequest, NextResponse } from "next/server";


export async function POST(request: NextRequest) {
  const apiUrl = process.env.LEXIC_API_URL;

  if (!apiUrl) {
    return NextResponse.json(
      { error: "LEXIC_API_URL is not configured." },
      { status: 500 },
    );
  }

  let upstream: Response;

  try {
    upstream = await fetch(
      `${apiUrl.replace(/\/$/, "")}/api/v1/session`,
      {
        method: "POST",
        headers: {
          cookie: request.headers.get("cookie") ?? "",
        },
        cache: "no-store",
      },
    );
  } catch {
    return NextResponse.json(
      { error: "The Lexic API could not be reached." },
      { status: 502 },
    );
  }

  const body: unknown = await upstream.json().catch(() => ({
    error: "The Lexic API returned an invalid response.",
  }));

  const response = NextResponse.json(body, {
    status: upstream.status,
  });

  const setCookie = upstream.headers.get("set-cookie");

  if (setCookie) {
    response.headers.set("set-cookie", setCookie);
  }

  response.headers.set("cache-control", "no-store");

  return response;
}
