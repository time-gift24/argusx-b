import type { Metadata } from "next";
import { Geist, Geist_Mono, Nunito_Sans } from "next/font/google";
import Link from "next/link";
import { Home, MessageCircle } from "lucide-react";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarInset,
} from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import "./globals.css";
import { SidebarToggle } from "@/components/sidebar-toggle";

const nunitoSans = Nunito_Sans({ variable: "--font-sans" });

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "ArgusX",
  description: "ArgusX Desktop Application",
};

const navItems = [
  {
    title: "Chat",
    href: "/chat",
    icon: MessageCircle,
  },
  {
    title: "Home",
    href: "/",
    icon: Home,
  },
];

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={cn(nunitoSans.variable, geistSans.variable, geistMono.variable)}>
      <body className="antialiased">
        <TooltipProvider>
          <SidebarProvider defaultOpen={false}>
            <Sidebar variant="floating" collapsible="offcanvas" className="top-16 h-[calc(100vh-4rem)]">
              <SidebarHeader className="flex flex-row items-center gap-2 py-3">
                <Link href="/" className="text-lg font-semibold hover:opacity-80 transition-opacity">
                  ArgusX
                </Link>
              </SidebarHeader>
              <SidebarContent>
                <SidebarGroup>
                  <SidebarMenu>
                    {navItems.map((item) => (
                      <SidebarMenuItem key={item.href}>
                        <SidebarMenuButton asChild isActive={false}>
                          <Link href={item.href} className="flex items-center gap-2">
                            <item.icon className="h-5 w-5" />
                            <span>{item.title}</span>
                          </Link>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>
                </SidebarGroup>
              </SidebarContent>
            </Sidebar>
            <SidebarInset className="mx-4">
              <header className="flex h-16 shrink-0 items-center gap-2 border-b px-6">
                <SidebarToggle className="flex h-9 w-9 items-center justify-center rounded-md hover:bg-accent" />
                <div className="flex flex-1 flex-col justify-center">
                  <Breadcrumb>
                    <BreadcrumbList>
                      <BreadcrumbItem>
                        <BreadcrumbLink href="/">Home</BreadcrumbLink>
                      </BreadcrumbItem>
                      <BreadcrumbSeparator />
                      <BreadcrumbItem>
                        <BreadcrumbPage>Dashboard</BreadcrumbPage>
                      </BreadcrumbItem>
                    </BreadcrumbList>
                  </Breadcrumb>
                  <div className="text-sm">
                    <span className="font-semibold">Welcome back</span>
                    <span className="text-muted-foreground ml-2">Manage your AI agents and conversations</span>
                  </div>
                </div>
              </header>
              <div className="flex flex-1 flex-col gap-4 p-6">
                {children}
              </div>
            </SidebarInset>
          </SidebarProvider>
        </TooltipProvider>
      </body>
    </html>
  );
}
