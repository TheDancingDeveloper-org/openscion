import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import { SiteNav } from "@/components/site-nav";
import { Separator } from "@/components/ui/separator";

import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "OpenScion",
  description: "Web interface for an OpenScion source control server.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="flex min-h-full flex-col">
        <header className="flex h-14 shrink-0 items-center px-6">
          <span className="font-heading text-base font-semibold tracking-tight">
            OpenScion
          </span>
        </header>
        <Separator />
        <div className="flex flex-1 flex-col md:flex-row">
          <aside className="shrink-0 border-b p-4 md:w-56 md:border-r md:border-b-0">
            <SiteNav />
          </aside>
          <main className="flex-1 p-6">{children}</main>
        </div>
      </body>
    </html>
  );
}
