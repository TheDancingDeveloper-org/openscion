import { Suspense } from "react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { probeVersion, type VersionProbe } from "@/lib/server";

/** One labelled value in the connection card. */
function Field({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1 sm:flex-row sm:items-baseline sm:justify-between sm:gap-4">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="font-mono break-all">{value}</dd>
    </div>
  );
}

function ConnectionDetails({ probe }: { probe: VersionProbe }) {
  if (probe.status === "connected") {
    const [major, minor] = probe.version.protocol_version;
    return (
      <dl className="flex flex-col gap-3">
        <Field label="Server version" value={probe.version.server_version} />
        <Separator />
        <Field label="Protocol version" value={`${major}.${minor}`} />
        <Separator />
        <Field label="Server URL" value={probe.url} />
      </dl>
    );
  }
  return (
    <div className="flex flex-col gap-4">
      <div role="alert" className="flex flex-col gap-1">
        <p className="font-medium">OpenScion could not reach the server.</p>
        <p className="text-muted-foreground">
          Start <code className="font-mono text-foreground">openscion-server</code>{" "}
          or set <code className="font-mono text-foreground">OPENSCION_SERVER_URL</code>{" "}
          to the address it listens on, then reload this page.
        </p>
      </div>
      <Separator />
      <dl className="flex flex-col gap-3">
        <Field label="Server URL" value={probe.url} />
        <Separator />
        <Field label="Reason" value={probe.reason} />
      </dl>
    </div>
  );
}

/** Probes the server at request time and renders the result. */
async function ServerStatus() {
  const probe = await probeVersion();
  const connected = probe.status === "connected";
  return (
    <Card>
      <CardHeader>
        <CardTitle>Server</CardTitle>
        <CardDescription>The openscion-server this interface talks to.</CardDescription>
        <CardAction>
          <Badge variant={connected ? "default" : "destructive"}>
            {connected ? "Connected" : "Unreachable"}
          </Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <ConnectionDetails probe={probe} />
      </CardContent>
    </Card>
  );
}

function ServerStatusPending() {
  return (
    <Card aria-busy="true">
      <CardHeader>
        <CardTitle>Server</CardTitle>
        <CardDescription>Checking the connection to openscion-server.</CardDescription>
      </CardHeader>
    </Card>
  );
}

export default function OverviewPage() {
  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h1 className="font-heading text-2xl font-semibold tracking-tight">Overview</h1>
        <p className="text-muted-foreground">
          Connection status of the OpenScion server.
        </p>
      </div>
      <Suspense fallback={<ServerStatusPending />}>
        <ServerStatus />
      </Suspense>
    </div>
  );
}
