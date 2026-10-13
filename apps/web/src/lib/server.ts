import "server-only";

/**
 * Access to the openscion-server HTTP API.
 *
 * Every call to the server goes through this module so the base URL is read
 * in exactly one place. It runs only on the server: the browser never talks to
 * openscion-server directly.
 */

/** Base URL used when `OPENSCION_SERVER_URL` is not set. */
export const DEFAULT_SERVER_URL = "http://127.0.0.1:7878";

/** How long a request to the server may take before it counts as unreachable. */
const REQUEST_TIMEOUT_MS = 2000;

/** Response body of `GET /v1/version`. */
export interface VersionInfo {
  /** Semantic version of the running openscion-server, e.g. `"0.2.1"`. */
  server_version: string;
  /** Wire protocol version as `[major, minor]`. */
  protocol_version: [number, number];
}

/** Outcome of probing the server's version endpoint. */
export type VersionProbe =
  | { status: "connected"; url: string; version: VersionInfo }
  | { status: "unreachable"; url: string; reason: string };

/**
 * Returns the configured server base URL without a trailing slash.
 *
 * Read at request time, so one build can be pointed at different servers.
 */
export function serverUrl(): string {
  const configured = process.env.OPENSCION_SERVER_URL?.trim();
  const url = configured ? configured : DEFAULT_SERVER_URL;
  return url.replace(/\/+$/, "");
}

function isProtocolVersion(value: unknown): value is [number, number] {
  return (
    Array.isArray(value) &&
    value.length === 2 &&
    value.every((part) => Number.isInteger(part) && part >= 0)
  );
}

/**
 * Checks that a decoded JSON value has the `/v1/version` response shape.
 */
export function isVersionInfo(value: unknown): value is VersionInfo {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return (
    typeof record.server_version === "string" &&
    record.server_version.length > 0 &&
    isProtocolVersion(record.protocol_version)
  );
}

function describeError(error: unknown): string {
  if (error instanceof Error) {
    if (error.name === "TimeoutError") {
      return `no response within ${REQUEST_TIMEOUT_MS} ms`;
    }
    const cause = (error as Error & { cause?: unknown }).cause;
    if (cause instanceof Error && cause.message) {
      return cause.message;
    }
    return error.message;
  }
  return String(error);
}

/**
 * Fetches `GET /v1/version` from openscion-server.
 *
 * Never throws: network failures, timeouts, non-2xx responses, and malformed
 * bodies all come back as `{ status: "unreachable" }` with a reason.
 */
export async function probeVersion(): Promise<VersionProbe> {
  const url = serverUrl();
  try {
    const response = await fetch(`${url}/v1/version`, {
      cache: "no-store",
      headers: { accept: "application/json" },
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    if (!response.ok) {
      return {
        status: "unreachable",
        url,
        reason: `server answered HTTP ${response.status}`,
      };
    }
    const body: unknown = await response.json();
    if (!isVersionInfo(body)) {
      return {
        status: "unreachable",
        url,
        reason: "server returned an unexpected /v1/version response",
      };
    }
    return { status: "connected", url, version: body };
  } catch (error) {
    return { status: "unreachable", url, reason: describeError(error) };
  }
}
