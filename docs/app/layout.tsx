import localFont from "next/font/local";
import { Head } from "nextra/components";
import "nextra-theme-docs/style.css";
import "./globals.css";
import type { ReactNode } from "react";

const fsex = localFont({
  src: "../public/fonts/FSEX300.ttf",
  variable: "--font-fsex",
});

const cascadiaCode = localFont({
  src: "../public/fonts/CascadiaCode-Bold.ttf",
  variable: "--font-cascadia",
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
      className={`${fsex.variable} ${cascadiaCode.variable}`}
    >
      <Head />
      <body className="font-sans">{children}</body>
    </html>
  );
}
