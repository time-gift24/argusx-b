import type { Metadata } from "next";
import Script from "next/script";
import { Geist, Geist_Mono, Nunito_Sans } from "next/font/google";
import { cn } from "@/lib/utils";
import "./globals.css";
import { AppLayout } from "@/components/layouts";

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

const themeInitScript = `
(() => {
  const storageKey = "argusx-theme";
  const root = document.documentElement;

  const resolveTheme = () => {
    try {
      const stored = window.localStorage.getItem(storageKey);
      if (stored === "light" || stored === "dark") {
        return stored;
      }
    } catch {}

    if (
      window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches
    ) {
      return "dark";
    }

    return "light";
  };

  const theme = resolveTheme();
  root.classList.toggle("dark", theme === "dark");
  root.style.colorScheme = theme;
})();
`;

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={cn(nunitoSans.variable, geistSans.variable, geistMono.variable)}
      suppressHydrationWarning
    >
      <body className="antialiased">
        <Script id="argusx-theme-init" strategy="beforeInteractive">
          {themeInitScript}
        </Script>
        <AppLayout>{children}</AppLayout>
      </body>
    </html>
  );
}
