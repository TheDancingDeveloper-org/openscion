"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** One entry in the sidebar navigation. */
interface NavItem {
  href: string;
  label: string;
}

const NAV_ITEMS: readonly NavItem[] = [{ href: "/", label: "Overview" }];

/** Sidebar navigation that highlights the entry for the current route. */
export function SiteNav() {
  const pathname = usePathname();
  return (
    <nav aria-label="Main" className="flex flex-col gap-1">
      {NAV_ITEMS.map((item) => {
        const active = pathname === item.href;
        return (
          <Link
            key={item.href}
            href={item.href}
            aria-current={active ? "page" : undefined}
            className={cn(
              buttonVariants({ variant: active ? "secondary" : "ghost" }),
              "h-9 justify-start px-3",
            )}
          >
            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}
