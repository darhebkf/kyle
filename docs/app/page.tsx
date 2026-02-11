"use client";

import { motion } from "motion/react";
import { useEffect, useState } from "react";
import FaultyTerminal from "@/components/FaultyTerminal";

const GRID_MUL: [number, number] = [2, 1];

const ASCII_KYLE = [
  "██╗  ██╗ ██╗   ██╗ ██╗      ███████╗",
  "██║ ██╔╝ ╚██╗ ██╔╝ ██║      ██╔════╝",
  "█████╔╝   ╚████╔╝  ██║      █████╗  ",
  "██╔═██╗    ╚██╔╝   ██║      ██╔══╝  ",
  "██║  ██╗    ██║    ███████╗ ███████╗",
  "╚═╝  ╚═╝    ╚═╝    ╚══════╝ ╚══════╝",
].join("\n");

function InstallCommand() {
  const [copied, setCopied] = useState(false);
  const [isWindows, setIsWindows] = useState(false);

  useEffect(() => {
    setIsWindows(navigator.platform.toLowerCase().includes("win"));
  }, []);

  const command = isWindows
    ? "irm https://kylefile.dev/install.ps1 | iex"
    : "curl -fsSL https://kylefile.dev/install.sh | sh";

  const handleCopy = () => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-col items-center gap-3">
      <button
        type="button"
        onClick={handleCopy}
        className="group flex items-center gap-3 bg-black/40 border border-white/10 rounded-lg px-5 py-3 hover:bg-black/60 hover:border-white/20 transition-all cursor-pointer backdrop-blur-sm"
      >
        <span className="text-white/60 text-sm font-mono">$</span>
        <code className="text-[#86EFAC] font-mono text-sm">{command}</code>
        <svg
          className="w-4 h-4 ml-2 text-white/30 group-hover:text-white/50 transition-colors"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-label={copied ? "Copied" : "Copy to clipboard"}
          role="img"
        >
          {copied ? (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M5 13l4 4L19 7"
            />
          ) : (
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
            />
          )}
        </svg>
      </button>
      <button
        type="button"
        onClick={() => setIsWindows(!isWindows)}
        className="text-white/60 text-xs hover:text-white/80 transition-colors cursor-pointer"
      >
        {isWindows ? "Show Linux/macOS command" : "Show Windows command"}
      </button>
    </div>
  );
}

export default function LandingPage() {
  const [showContent, setShowContent] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setShowContent(true), 2200);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="min-h-screen w-screen overflow-hidden relative bg-black">
      <div className="absolute inset-0 z-0">
        <FaultyTerminal
          scale={1.5}
          gridMul={GRID_MUL}
          digitSize={1.2}
          timeScale={0.5}
          scanlineIntensity={0.5}
          glitchAmount={1}
          flickerAmount={1}
          noiseAmp={1}
          chromaticAberration={0}
          dither={0}
          curvature={0}
          tint="#ff2056"
          mouseReact={false}
          pageLoadAnimation
          brightness={0.6}
        />
      </div>
      {showContent && (
        <div className="relative z-10 min-h-screen flex flex-col items-center justify-center px-6">
          <motion.pre
            className="text-white/80 font-mono text-[0.35rem] sm:text-[0.5rem] md:text-xs lg:text-sm leading-none select-none whitespace-pre"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
          >
            {ASCII_KYLE}
          </motion.pre>

          <motion.p
            className="text-white/60 text-lg sm:text-xl mt-6 text-center"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.2 }}
          >
            A fast, polyglot and customizable project manager.
          </motion.p>
          <motion.div
            className="mt-8"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.4 }}
          >
            <InstallCommand />
          </motion.div>
        </div>
      )}
      {showContent && (
        <motion.div
          className="absolute bottom-4 left-0 right-0 text-center text-xs text-white/60 z-20"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.6 }}
        >
          © {new Date().getFullYear()} Kyle. All rights reserved.
        </motion.div>
      )}
    </div>
  );
}
