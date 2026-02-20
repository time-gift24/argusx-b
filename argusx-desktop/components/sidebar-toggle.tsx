"use client";

import { PanelLeft, PanelRight } from "lucide-react";
import { useSidebar } from "@/components/ui/sidebar";

export function SidebarToggle({ className }: { className?: string }) {
  const { state, toggleSidebar } = useSidebar();

  return (
    <button
      onClick={toggleSidebar}
      className={className}
      aria-label={state === "expanded" ? "Collapse sidebar" : "Expand sidebar"}
    >
      {state === "expanded" ? (
        <PanelRight className="h-5 w-5" />
      ) : (
        <PanelLeft className="h-5 w-5" />
      )}
    </button>
  );
}
