import { ClipboardCheck, FolderOpen, FileCheck, ScrollText } from "lucide-react";
import Link from "next/link";

const navItems = [
  { href: "/prompt-lab", label: "Dashboard", icon: ClipboardCheck },
  { href: "/prompt-lab/checklist", label: "Checklist", icon: FolderOpen },
  { href: "/prompt-lab/golden-sets", label: "Golden Sets", icon: FileCheck },
  { href: "/prompt-lab/results", label: "Results", icon: ScrollText },
];

export default function PromptLabLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex gap-6">
      <aside className="w-48 shrink-0">
        <nav className="space-y-1">
          {navItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="flex items-center gap-2 px-3 py-2 text-sm rounded-md hover:bg-accent"
            >
              <item.icon className="h-4 w-4" />
              {item.label}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="flex-1">{children}</main>
    </div>
  );
}
