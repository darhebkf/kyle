import { getPageMap } from "nextra/page-map";
import { Layout, Navbar } from "nextra-theme-docs";
import type { ReactNode } from "react";
import { ThemeToggle } from "@/components/ThemeToggle";
import { TocFooter } from "@/components/TocFooter";
import { Logo } from "@/components/Logo";

const navbar = (
  <Navbar logo={<Logo />} projectLink="https://github.com/darhebkf/kyle">
    <ThemeToggle />
  </Navbar>
);

const footer = (
  <footer className="py-4 text-center text-xs text-neutral-400 dark:text-white/60">
    &copy; {new Date().getFullYear()} Kyle. All rights reserved.
  </footer>
);

export default async function DocsLayout({
  children,
}: {
  children: ReactNode;
}) {
  return (
    <Layout
      navbar={navbar}
      pageMap={await getPageMap("/docs")}
      docsRepositoryBase="https://github.com/darhebkf/kyle/tree/main/docs"
      footer={footer}
      sidebar={{ toggleButton: false }}
      darkMode={false}
      editLink={null}
      feedback={{ content: null }}
      copyPageButton={false}
      toc={{ backToTop: null, extraContent: <TocFooter /> }}
    >
      {children}
    </Layout>
  );
}
