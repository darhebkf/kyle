import localFont from "next/font/local";
import { Head } from "nextra/components";
import "nextra-theme-docs/style.css";
import "./globals.css";
import type { ReactNode } from "react";

const fsex = localFont({
  src: "../public/fonts/FSEX300.ttf",
  variable: "--font-fsex",
});

export const metadata = {
  title: "Kyle",
  description: "A fast, polyglot and customizable project manager runner",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html
      lang="en"
      dir="ltr"
      suppressHydrationWarning
      className={fsex.variable}
    >
      <Head />
      <body className="font-sans">{children}</body>
    </html>
  );
}
