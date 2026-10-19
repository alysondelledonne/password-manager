// Pal, the elephant mascot, and the other drawings on the page. All are
// decorative (aria-hidden) and drawn inline, so the strict Content Security
// Policy needs no image sources. Colors come from the CSS variables.

/** The brand mark: Pal's head on a lime tile. Same drawing as public/favicon.svg. */
export function BrandMark({ size = 32 }: { size?: number }) {
  return (
    <svg className="brand-mark" width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
      <rect width="32" height="32" rx="8" fill="var(--lime)" />
      <path d="M17 6.5c5.5-1.8 9.5 2 9 7.4-.5 4.4-3.6 6.6-6.6 5.6" fill="var(--grey-700)" />
      <circle cx="14" cy="13.5" r="7.3" fill="var(--grey-900)" />
      <path
        d="M9 16.5c-1.4 3.4-1.6 6.6.4 8.6 1.3 1.3 3.2.9 3.6-.6"
        fill="none"
        stroke="var(--grey-900)"
        strokeWidth="3.6"
        strokeLinecap="round"
      />
      <circle cx="11.6" cy="12" r="1.3" fill="var(--lime)" />
    </svg>
  );
}

/** Pal, standing, holding up a word tile with its trunk. */
export function Elephant({ word }: { word: string }) {
  return (
    <svg className="elephant" viewBox="0 0 360 290" aria-hidden="true">
      <ellipse cx="212" cy="264" rx="124" ry="9" fill="var(--grey-950)" opacity=".6" />
      {/* far legs and tail */}
      <rect x="246" y="190" width="32" height="72" rx="13" fill="var(--hide-far)" />
      <rect x="168" y="196" width="32" height="66" rx="13" fill="var(--hide-far)" />
      <path d="M320 158q16 14 14 36" fill="none" stroke="var(--hide-shade)" strokeWidth="5" strokeLinecap="round" />
      <path d="M334 190l-7 13 11-4z" fill="var(--grey-500)" />
      {/* body and near legs */}
      <path
        d="M120 150C120 92 178 82 236 86c56 4 90 34 90 82 0 38-26 54-64 56h-92c-32 0-50-28-50-74z"
        fill="var(--hide)"
      />
      <rect x="270" y="196" width="34" height="68" rx="13" fill="var(--hide-near)" />
      <rect x="186" y="200" width="34" height="64" rx="13" fill="var(--hide-near)" />
      <g fill="var(--toenail)">
        <ellipse cx="195" cy="258" rx="5" ry="3.5" />
        <ellipse cx="208" cy="258" rx="5" ry="3.5" />
        <ellipse cx="279" cy="258" rx="5" ry="3.5" />
        <ellipse cx="292" cy="258" rx="5" ry="3.5" />
      </g>
      {/* trunk, raised */}
      <path
        d="M92 130c-8 32-26 50-44 36-14-12-14-38-6-62"
        fill="none"
        stroke="var(--hide)"
        strokeWidth="25"
        strokeLinecap="round"
      />
      <path d="M42 104l4-18" fill="none" stroke="var(--hide)" strokeWidth="19" strokeLinecap="round" />
      <g fill="none" stroke="var(--hide-shade)" strokeWidth="2.4" strokeLinecap="round">
        <path d="M31 120q10 4 20 0" />
        <path d="M31 133q10 4 20 1" />
        <path d="M35 146q9 3 17-2" />
      </g>
      {/* the word tile */}
      <g transform="rotate(-10 46 62)">
        <rect x="4" y="44" width="90" height="34" rx="9" fill="var(--lime)" />
        <text x="49" y="67" textAnchor="middle" className="elephant-word">
          {word}
        </text>
      </g>
      {/* head, ear, face */}
      <circle cx="122" cy="112" r="52" fill="var(--hide)" />
      <path d="M146 70c50-18 76 22 68 62-6 34-32 46-54 36-18-8-22-48-14-98z" fill="var(--hide-shade)" />
      <path d="M158 86c30-8 42 18 38 44-4 20-16 26-26 22-12-6-16-36-12-66z" fill="var(--mustard)" opacity=".35" />
      <path d="M100 92q8-6 16-2" fill="none" stroke="var(--grey-500)" strokeWidth="2.5" strokeLinecap="round" />
      <ellipse cx="108" cy="106" rx="5.5" ry="7" fill="var(--grey-950)" />
      <circle cx="110" cy="103" r="1.8" fill="#fff" />
      <circle cx="124" cy="132" r="8" fill="var(--mustard)" opacity=".22" />
      <path d="M104 150q4 14-8 22" fill="none" stroke="var(--tusk)" strokeWidth="7" strokeLinecap="round" />
    </svg>
  );
}

/** Small drawing for "Easy to remember": Pal's head, as in the brand mark. */
export function MemoryIcon() {
  return (
    <svg className="guide-icon" viewBox="0 0 32 32" aria-hidden="true">
      <path d="M17 6.5c5.5-1.8 9.5 2 9 7.4-.5 4.4-3.6 6.6-6.6 5.6" fill="currentColor" opacity=".55" />
      <circle cx="14" cy="13.5" r="7.3" fill="currentColor" />
      <path
        d="M9 16.5c-1.4 3.4-1.6 6.6.4 8.6 1.3 1.3 3.2.9 3.6-.6"
        fill="none"
        stroke="currentColor"
        strokeWidth="3.6"
        strokeLinecap="round"
      />
      <circle cx="11.6" cy="12" r="1.3" fill="var(--grey-900)" />
    </svg>
  );
}

/** Small line drawing for "Hard to guess": a padlock. */
export function GuessIcon() {
  return (
    <svg className="guide-icon" viewBox="0 0 32 32" aria-hidden="true">
      <path d="M10.5 14V10a5.5 5.5 0 0 1 11 0v4" fill="none" stroke="currentColor" strokeWidth="2" />
      <rect x="7" y="14" width="18" height="13" rx="3" fill="none" stroke="currentColor" strokeWidth="2" />
      <path d="M16 19v3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}
