"use client";

import { useTheme } from "next-themes";

export function Logo() {
  const { resolvedTheme } = useTheme();
  const textFill = resolvedTheme === "dark" ? "#ffffff" : "#000000";

  return (
    <svg
      viewBox="0 0 160 50"
      className="h-7 w-auto"
      role="img"
      aria-label="Kyle logo"
    >
      {/* Terminal body */}
      <rect
        x="3" y="5" width="38" height="40" rx="7"
        fill="#ffffff"
        stroke="#ff2056"
        strokeWidth="4"
        strokeLinejoin="round"
      />
      {/* >_< Face */}
      <g stroke="#ff2056" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="12,20 17,25 12,30" />
        <line x1="19" y1="32" x2="25" y2="32" />
        <polyline points="32,20 27,25 32,30" />
      </g>

      {/* Text */}
      <text
        x="52" y="38"
        fontSize="36"
        fill={textFill}
        style={{ fontFamily: "var(--font-cascadia), monospace" }}
      >
        KYLE
      </text>
    </svg>
  );
}
