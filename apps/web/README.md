# openscion-web

The OpenScion web interface. It is a Next.js (App Router) application that
talks to `openscion-server` over the server's HTTP API. All requests to the
server are made server-side from React Server Components, so the browser only
ever talks to this app, never to `openscion-server` directly.

Today it has one page, **Overview**, which calls `GET /v1/version` on every
request and shows whether the server is reachable, the server version, the
protocol version, and the URL it used.

## Running it

Requires Node.js 20 or newer and pnpm (the exact version is pinned in
`package.json` under `packageManager`; `corepack enable` picks it up).

```sh
cd apps/web
pnpm install
pnpm dev
```

Then open <http://localhost:3000>. Start `openscion-server` first, or the
Overview page reports the server as unreachable and says why.

## Configuration

| Variable               | Default                 | Purpose                                  |
| ---------------------- | ----------------------- | ---------------------------------------- |
| `OPENSCION_SERVER_URL` | `http://127.0.0.1:7878` | Base URL of the openscion-server HTTP API |

Copy `.env.example` to `.env.local` to override it in development. The value
is read at request time, so a single production build can be pointed at a
different server through the environment of `pnpm start`.

## Scripts

| Script           | What it does                                              |
| ---------------- | --------------------------------------------------------- |
| `pnpm dev`       | Development server with hot reload                        |
| `pnpm build`     | Production build                                          |
| `pnpm start`     | Serve the production build                                |
| `pnpm lint`      | ESLint with the Next.js rules                             |
| `pnpm typecheck` | Generate Next.js route types, then `tsc --noEmit`         |

## Layout

```text
src/
├── app/                # routes: root layout and the Overview page
├── components/
│   ├── site-nav.tsx    # sidebar navigation
│   └── ui/             # shadcn/ui components (button, card, badge, separator)
└── lib/
    ├── server.ts       # the only module that talks to openscion-server
    └── utils.ts        # class name helper used by the shadcn/ui components
```

Every call to `openscion-server` goes through `src/lib/server.ts`, which reads
`OPENSCION_SERVER_URL`, applies a two-second timeout, and validates response
shapes by hand before the UI uses them.

UI components come from [shadcn/ui](https://ui.shadcn.com) (configured in
`components.json`). Add more with `pnpm dlx shadcn@latest add <component>`.
Light and dark themes follow the operating system's `prefers-color-scheme`.
