"use client";

import { useTheme } from "next-themes";

const ASCII_KYLE = [
  "██╗  ██╗ ██╗   ██╗ ██╗      ███████╗",
  "██║ ██╔╝ ╚██╗ ██╔╝ ██║      ██╔════╝",
  "█████╔╝   ╚████╔╝  ██║      █████╗  ",
  "██╔═██╗    ╚██╔╝   ██║      ██╔══╝  ",
  "██║  ██╗    ██║    ███████╗ ███████╗",
  "╚═╝  ╚═╝    ╚═╝    ╚══════╝ ╚══════╝",
].join("\n");

export function Logo() {
  const { resolvedTheme } = useTheme();
  const textFill = resolvedTheme === "dark" ? "#ffffff" : "#000000";

  return (
    <div className="flex items-center gap-2" role="img" aria-label="Kyle logo">
      {/* Terminal icon */}
      <svg viewBox="0 0 44 50" className="h-7 w-auto shrink-0 -translate-y-px">
        <rect
          x="3" y="5" width="38" height="40" rx="7"
          fill="#ffffff"
          stroke="#ff2056"
          strokeWidth="4"
          strokeLinejoin="round"
        />
        <g stroke="#ff2056" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="12,20 17,25 12,30" />
          <line x1="19" y1="32" x2="25" y2="32" />
          <polyline points="32,20 27,25 32,30" />
        </g>
      </svg>
      {/* ASCII KYLE */}
      <pre
        className="font-mono leading-none select-none whitespace-pre"
        style={{ fontSize: "3.5px", color: textFill }}
      >
        {ASCII_KYLE}
      </pre>
    </div>
  );
}
